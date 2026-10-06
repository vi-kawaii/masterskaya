use arboard::Clipboard;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

const PROMPT: &str = r##"Отвечай СТРОГО в таком формате, без пояснений вне блоков:

FILE путь/к/файлу
====
<полное содержимое файла>
====
END FILE

Правила:
- Путь указывай относительно корня проекта (например, src/main.rs).
- Содержимое — полное, без сокращений и без "...".
- Никаких markdown-обёрток вокруг блоков.
- Если файл не менялся — не включай его.
- Если файл нужно удалить — напиши: DELETE путь/к/файлу
"##;

pub fn run(cwd: &Path) -> Result<(), String> {
    println!("──────────────────────────────────────────────");
    println!("ПРОМПТ (скопируй и вставь в DeepSeek):");
    println!("──────────────────────────────────────────────");
    println!("{PROMPT}");
    println!("──────────────────────────────────────────────");
    println!();

    let mut clipboard = Clipboard::new().map_err(|e| format!("нет доступа к буферу: {e}"))?;

    if clipboard.set_text(PROMPT.to_string()).is_ok() {
        println!("✅ Промпт уже в буфере обмена — просто вставь в чат.");
    }

    println!();
    println!("Слушаю буфер обмена... Нажмите Ctrl+C для выхода.");
    println!("Скопируй ответ DeepSeek с блоками FILE ... END FILE");
    println!();

    let mut last_text = PROMPT.to_string();

    loop {
        if let Ok(text) = clipboard.get_text() {
            if text != last_text
                && (text.contains("FILE ") || text.contains("DELETE "))
                && text.contains("END FILE")
            {
                last_text = text.clone();
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
                    }
                    Err(e) => {
                        println!();
                        println!("❌ Ошибка распаковки: {e}");
                        println!();
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(300));
    }
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
                content.push_str(lines[i]);
                content.push('\n');
                i += 1;
            }
            if i < lines.len() && lines[i].trim() == "====" {
                i += 1;
            }
            if i < lines.len() && lines[i].trim() == "END FILE" {
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
