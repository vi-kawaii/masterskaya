//! `lab commit` — умный коммит.
//!
//! Если запущен в корне мастерской (есть `.git/` и есть подпапки с `.git/`):
//!   1. коммитит корень мастерской;
//!   2. обходит подпапки на один уровень вглубь и коммитит каждую,
//!      где есть `.git/` и есть изменения.
//!
//! Если запущен внутри проекта (есть `.git/`, подпапок с `.git/` нет):
//!   коммитит только `cwd`.
//!
//! На закрытый редактор без сообщения — пропускает проект и идёт дальше,
//! в конце печатает сводку.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(cwd: &Path) -> Result<(), String> {
    let has_git_here = cwd.join(".git").is_dir();
    let subprojects = scan_subprojects(cwd);

    // Случай 1: корень мастерской.
    // Признак: есть хотя бы одна подпапка с `.git/`.
    // (Если сам cwd без .git, но подпапки есть — тоже мастерская,
    //  но коммитить корень нечего, пропустим его.)
    if !subprojects.is_empty() {
        return run_workspace(cwd, has_git_here, &subprojects);
    }

    // Случай 2: одиночный проект.
    if has_git_here {
        return commit_one(cwd);
    }

    Err("здесь нечего коммитить: нет .git/ и нет подпапок с .git/".into())
}

/// Коммит мастерской: сначала корень (если есть .git), потом все подпапки.
fn run_workspace(
    cwd: &Path,
    has_git_here: bool,
    subprojects: &[PathBuf],
) -> Result<(), String> {
    let mut committed = 0usize;
    let mut clean = 0usize;
    let mut skipped = 0usize;

    println!("Мастерская: {}", cwd.display());

    // Корень.
    if has_git_here {
        match has_changes(cwd) {
            Ok(true) => {
                println!();
                println!("── корень мастерской ──");
                match commit_one(cwd) {
                    Ok(()) => committed += 1,
                    Err(e) => {
                        eprintln!("⚠️  корень: {e}");
                        skipped += 1;
                    }
                }
            }
            Ok(false) => {
                println!("  ○ корень: чисто");
                clean += 1;
            }
            Err(e) => {
                eprintln!("⚠️  корень: не могу проверить ({e})");
                skipped += 1;
            }
        }
    }

    // Подпапки.
    for p in subprojects {
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| p.display().to_string());

        match has_changes(p) {
            Ok(true) => {
                println!();
                println!("── {name} ──");
                match commit_one(p) {
                    Ok(()) => committed += 1,
                    Err(e) => {
                        eprintln!("⚠️  {name}: {e}");
                        skipped += 1;
                    }
                }
            }
            Ok(false) => {
                println!("  ○ {name}: чисто");
                clean += 1;
            }
            Err(e) => {
                eprintln!("⚠️  {name}: не могу проверить ({e})");
                skipped += 1;
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════");
    println!("Готово. Закоммичено: {committed}, чисто: {clean}, пропущено: {skipped}");
    println!("═══════════════════════════════════════");

    Ok(())
}

/// Все прямые подпапки `cwd`, в которых есть `.git/`.
/// На один уровень вглубь. Скрытые папки (начинаются с `.`) пропускаем.
fn scan_subprojects(cwd: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();

    let Ok(entries) = fs::read_dir(cwd) else {
        return out;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        if path.join(".git").is_dir() {
            out.push(path);
        }
    }

    // Стабильный порядок для читаемого вывода.
    out.sort();
    out
}

/// Есть ли незакоммиченные изменения в репозитории.
/// `git status --porcelain` возвращает пустой вывод, если всё чисто.
fn has_changes(path: &Path) -> Result<bool, String> {
    let out = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(path)
        .output()
        .map_err(|e| format!("не удалось запустить git: {e}"))?;

    if !out.status.success() {
        return Err(format!(
            "git status завершился с кодом {:?}",
            out.status.code()
        ));
    }

    Ok(!out.stdout.is_empty())
}

/// Коммит и пуш одного репозитория.
/// `git commit` без `-m` — git сам откроет core.editor и дождётся закрытия.
fn commit_one(cwd: &Path) -> Result<(), String> {
    run_cmd(Command::new("git").arg("add").arg("-A").current_dir(cwd))
        .map_err(|e| format!("git add: {e}"))?;

    run_cmd(Command::new("git").arg("commit").current_dir(cwd)).map_err(|e| {
        format!("git commit: {e}\n   (вероятно, пустое сообщение или нечего коммитить — пропускаю)")
    })?;

    run_cmd(Command::new("git").arg("push").current_dir(cwd))
        .map_err(|e| format!("git push: {e}\n   (коммит создан локально, пуш не удался)"))?;

    println!("  ✅ закоммичено и запушено");
    Ok(())
}

fn run_cmd(cmd: &mut Command) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("не удалось запустить: {e}"))?;
    if !status.success() {
        return Err(format!("команда завершилась с кодом {:?}", status.code()));
    }
    Ok(())
}
