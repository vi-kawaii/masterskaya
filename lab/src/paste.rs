use arboard::Clipboard;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Показывается пользователю. Без raw-строк, без вложенных маркеров,
/// которые могли бы сломать парсер.
fn prompt_text() -> String {
    let mut s = String::new();
    s.push_str("Отвечай СТРОГО в таком формате, без пояснений вне блоков:\n");
    s.push('\n');
    s.push_str("FILE путь/к/файлу\n");
    s.push_str("====\n");
    s.push_str("<полное содержимое файла>\n");
    s.push_str("====\n");
    s.push_str("EOF\n");
    s.push('\n');
    s.push_str("Правила:\n");
    s.push_str("- Путь относительно корня проекта (например, src/main.rs).\n");
    s.push_str("- Содержимое полное, без сокращений и без \"...\".\n");
    s.push_str("- Без markdown-обёрток вокруг блоков.\n");
    s.push_str("- Если файл не менялся — не включай его.\n");
    s.push_str("- Если файл нужно удалить: DELETE путь/к/файлу\n");
    s
}

pub fn run(cwd: &Path, restart_after: bool) -> Result<(), String> {
    let prompt = prompt_text();

    println!("──────────────────────────────────────────────");
    println!("ПРОМПТ (скопируй и вставь в DeepSeek):");
    println!("──────────────────────────────────────────────");
    println!("{prompt}");
    println!("──────────────────────────────────────────────");
    println!();
    if restart_after {
        println!("🔄 Режим --restart: после применения файлов проект перезапустится.");
        println!();
    }

    let mut clipboard = Clipboard::new().map_err(|e| format!("нет доступа к буферу: {e}"))?;

    println!();
    println!("Слушаю буфер обмена... Нажмите Ctrl+C для выхода.");
    println!("Скопируй ответ DeepSeek с блоками FILE ... EOF");
    println!();

    // Текущее содержимое буфера считаем «уже виденным».
    let mut last_text = clipboard.get_text().unwrap_or_default();

    loop {
        if let Ok(text) = clipboard.get_text() {
            if text != last_text {
                last_text = text.clone();

                let has_file_block = text.contains("FILE ") || text.contains("DELETE ");
                let has_end_marker = text
                    .lines()
                    .any(|l| l.trim() == "EOF" || l.trim() == "END FILE");

                if has_file_block && has_end_marker {
                    match unpack_files(&text, cwd) {
                        Ok((w, d, s)) => {
                            println!();
                            println!("══════════════ РЕЗУЛЬТАТ ══════════════");
                            if !w.is_empty() {
                                println!("✅ Записано ({}):", w.len());
                                for f in &w {
                                    println!("   • {f}");
                                }
                            }
                            if !d.is_empty() {
                                println!("🗑 Удалено ({}):", d.len());
                                for f in &d {
                                    println!("   • {f}");
                                }
                            }
                            if !s.is_empty() {
                                println!("⚠️ Пропущено ({}):", s.len());
                                for f in &s {
                                    println!("   • {f}");
                                }
                            }
                            println!("═══════════════════════════════════════");
                            println!();

                            if restart_after && (!w.is_empty() || !d.is_empty()) {
                                println!("🔄 Перезапуск проекта...");
                                println!();
                                // once=true: запускаем проект один раз
                                // и возвращаемся к слушанию буфера —
                                // без цикла выбора.
                                if let Err(e) = crate::run::run_with_options(
                                    cwd,
                                    crate::run::RunOptions {
                                        package: None,
                                        once: true,
                                    },
                                ) {
                                    eprintln!("❌ Ошибка перезапуска: {e}");
                                }
                                println!();
                                println!("👀 Слушаю буфер обмена дальше...");
                                println!();
                            }
                        }
                        Err(e) => {
                            println!();
                            println!("❌ Ошибка распаковки: {e}");
                            println!();
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(300));
    }
}

fn is_end_marker(line: &str) -> bool {
    let t = line.trim();
    t == "EOF" || t == "END FILE"
}

fn unpack_files(
    text: &str,
    cwd: &Path,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
    let mut written = Vec::new();
    let mut deleted = Vec::new();
    let mut skipped = Vec::new();

    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if let Some(path) = line.strip_prefix("DELETE ") {
            let path = path.trim().to_string();
            let p = cwd.join(&path);
            if p.exists() {
                match fs::remove_file(&p) {
                    Ok(_) => deleted.push(path),
                    Err(e) => skipped.push(format!("{path} (ошибка удаления: {e})")),
                }
            } else {
                skipped.push(format!("{path} (не существует)"));
            }
            i += 1;
            continue;
        }

        if let Some(path) = line.strip_prefix("FILE ") {
            let path = path.trim().to_string();
            i += 1;

            if i < lines.len() && lines[i].trim() == "====" {
                i += 1;
            }

            let mut content = String::new();
            while i < lines.len() && lines[i].trim() != "====" {
                if is_end_marker(lines[i]) {
                    break;
                }
                content.push_str(lines[i]);
                content.push('\n');
                i += 1;
            }
            if i < lines.len() && lines[i].trim() == "====" {
                i += 1;
            }
            if i < lines.len() && is_end_marker(lines[i]) {
                i += 1;
            }

            if path.is_empty() || path.contains("..") {
                skipped.push(format!("{path} (подозрительный путь)"));
                continue;
            }

            let file_path = cwd.join(&path);
            if let Some(parent) = file_path.parent() {
                if !parent.as_os_str().is_empty() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        skipped.push(format!(
                            "{path} (не могу создать папку {}: {e})",
                            parent.display()
                        ));
                        continue;
                    }
                }
            }
            match fs::write(&file_path, content) {
                Ok(_) => written.push(path),
                Err(e) => skipped.push(format!("{path} (ошибка записи: {e})")),
            }
            continue;
        }

        i += 1;
    }

    if written.is_empty() && deleted.is_empty() {
        return Err("Не найдено ни одного блока FILE или DELETE".to_string());
    }
    Ok((written, deleted, skipped))
}
