//! `lab run` — выбрать и запустить проект из мастерской.
//!
//! Обходит дерево от cwd, ищет все Cargo.toml с признаками запускаемого
//! проекта (src/main.rs или [[bin]]). Показывает меню через dialoguer.
//!
//! Помнит последний запущенный пакет в `.lab_run_cache` и предлагает его
//! первым при следующем запуске.
//!
//! После завершения проекта (успешного, с ошибкой или падения) снова
//! показывает выбор — чтобы не перезапускать таску вручную.

use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const RUN_CACHE_PATH: &str = ".lab_run_cache";

/// Найденный запускаемый проект.
struct RunnableProject {
    /// Имя пакета из Cargo.toml (для `cargo run -p`).
    package: String,
    /// Папка, откуда запускать (где лежит Cargo.toml).
    dir: PathBuf,
    /// Человекочитаемое имя для меню.
    display: String,
}

/// Опции запуска: аргумент командной строки (если был) и режим «один раз».
pub struct RunOptions {
    /// Если Some — запустить только этот проект.
    pub package: Option<String>,
    /// Если true — после завершения проекта не показывать меню, выйти.
    pub once: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            package: None,
            once: false,
        }
    }
}

pub fn run(cwd: &Path, arg: Option<&str>) -> Result<(), String> {
    run_with_options(
        cwd,
        RunOptions {
            package: arg.map(|s| s.to_string()),
            once: false,
        },
    )
}

pub fn run_with_options(cwd: &Path, opts: RunOptions) -> Result<(), String> {
    let projects = scan_runnable(cwd);

    if projects.is_empty() {
        return Err(format!(
            "не нашёл ни одного запускаемого проекта в {}",
            cwd.display()
        ));
    }

    // Явно указан аргумент — ищем его среди найденных.
    if let Some(ref name) = opts.package {
        let p = projects
            .iter()
            .find(|p| p.package == *name || p.display == *name)
            .ok_or_else(|| {
                format!(
                    "проект '{name}' не найден. Доступно: {}",
                    projects
                        .iter()
                        .map(|p| p.display.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        if opts.once {
            return cargo_run(p, cwd);
        }
        return run_loop(cwd, &projects, Some(&p.package));
    }

    // Один проект — без вопроса.
    if projects.len() == 1 {
        if opts.once {
            return cargo_run(&projects[0], cwd);
        }
        return run_loop(cwd, &projects, None);
    }

    if opts.once {
        // Без цикла: выбрать один раз и выйти.
        let last_run_package = read_run_cache(cwd);
        let choice = pick(&projects, last_run_package.as_deref())?;
        return cargo_run(&projects[choice], cwd);
    }

    run_loop(cwd, &projects, None)
}

/// Цикл: выбор → запуск → завершение → снова выбор.
/// `initial_package` — если Some, первый запуск без вопроса.
fn run_loop(
    cwd: &Path,
    projects: &[RunnableProject],
    initial_package: Option<&str>,
) -> Result<(), String> {
    let mut first = true;
    let mut forced_package = initial_package.map(|s| s.to_string());

    loop {
        let last_run_package = read_run_cache(cwd);
        let last_ref = forced_package.as_deref().or(last_run_package.as_deref());

        let choice = if first && forced_package.is_some() {
            // Первый запуск с явно указанным пакетом — без вопроса.
            let name = forced_package.as_deref().unwrap();
            projects
                .iter()
                .position(|p| p.package == name || p.display == name)
                .ok_or_else(|| format!("проект '{name}' не найден"))?
        } else {
            pick(projects, last_ref)?
        };

        first = false;
        // После первого запуска больше не форсируем.
        forced_package = None;

        match cargo_run(&projects[choice], cwd) {
            Ok(()) => {
                println!();
                println!("═══════════════════════════════════════");
                println!("✅ Проект завершился нормально");
                println!("═══════════════════════════════════════");
            }
            Err(e) => {
                println!();
                println!("═══════════════════════════════════════");
                println!("⚠️  Проект завершился с ошибкой:");
                println!("   {e}");
                println!("═══════════════════════════════════════");
            }
        }

        println!();
        println!("Снова показываю меню выбора проекта. Ctrl+C — выйти.");
        println!();
    }
}

/// Обходит дерево от cwd, находит все папки с Cargo.toml.
/// В найденный проект глубже не заходит.
fn scan_runnable(cwd: &Path) -> Vec<RunnableProject> {
    const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", ".zed", ".idea"];

    let mut out = Vec::new();

    let walker = WalkDir::new(cwd)
        .into_iter()
        .filter_entry(|e| {
            // Скипаем мусорные папки на входе.
            if !e.file_type().is_dir() {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref())
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            continue;
        }
        let dir = entry.path();

        // Есть Cargo.toml — это кандидат.
        if dir.join("Cargo.toml").exists() {
            if let Some(p) = project_at(dir) {
                out.push(p);
            }
        }
    }

    out.sort_by(|a, b| a.display.cmp(&b.display));
    out
}

/// Читает Cargo.toml и проверяет, можно ли проект запустить.
fn project_at(dir: &Path) -> Option<RunnableProject> {
    let toml_path = dir.join("Cargo.toml");
    let toml = fs::read_to_string(&toml_path).ok()?;

    let package = extract_package_name(&toml)?;

    // Запускаемо, если есть src/main.rs или [[bin]].
    let has_main = dir.join("src/main.rs").exists();
    let has_bin = toml.contains("[[bin]]");
    if !has_main && !has_bin {
        return None;
    }

    Some(RunnableProject {
        package,
        dir: dir.to_path_buf(),
        display: dir.file_name()?.to_string_lossy().to_string(),
    })
}

/// Достаёт `name = "..."` из секции [package].
/// Возвращает None для workspace-манифеста (там нет [package]).
fn extract_package_name(toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in toml.lines() {
        let line = line.trim();
        if line == "[package]" {
            in_package = true;
            continue;
        }
        if line.starts_with('[') && line != "[package]" {
            in_package = false;
        }
        if in_package && line.starts_with("name") {
            if let Some(v) = line.split('=').nth(1) {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Читает имя последнего запущенного пакета из кэша.
fn read_run_cache(cwd: &Path) -> Option<String> {
    let cache_path = cwd.join(RUN_CACHE_PATH);
    fs::read_to_string(cache_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Пишет имя запущенного пакета в кэш.
fn write_run_cache(cwd: &Path, package: &str) -> Result<(), String> {
    let cache_path = cwd.join(RUN_CACHE_PATH);
    fs::write(cache_path, package).map_err(|e| format!("не могу записать кэш запуска: {e}"))
}

/// Интерактивный выбор через dialoguer.
/// Если есть `last_run` и он найден среди проектов — он идёт первым в списке
/// и выбран по умолчанию.
fn pick(projects: &[RunnableProject], last_run: Option<&str>) -> Result<usize, String> {
    // Индекс «последнего запущенного» в исходном массиве.
    let last_run_original_index = last_run
        .and_then(|last| projects.iter().position(|p| p.package == last));

    // Порядок отображения: last_run первым (если есть), остальные — в исходном порядке.
    let mut order: Vec<usize> = (0..projects.len()).collect();
    if let Some(idx) = last_run_original_index {
        order.retain(|&i| i != idx);
        order.insert(0, idx);
    }

    let items: Vec<String> = order
        .iter()
        .map(|&i| {
            let p = &projects[i];
            format!("{}  ({})", p.display, p.package)
        })
        .collect();

    let default_pos = 0;

    let chosen_pos = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Что запустить?")
        .items(&items)
        .default(default_pos)
        .interact()
        .map_err(|e| format!("не удалось получить выбор: {e}"))?;

    // Переводим позицию в отображаемом списке в индекс в исходном массиве.
    Ok(order[chosen_pos])
}

/// Запускает `cargo run -p <package>` в папке проекта.
/// Перед запуском пишет package в кэш.
fn cargo_run(p: &RunnableProject, workspace_root: &Path) -> Result<(), String> {
    println!();
    println!("▶ cargo run -p {}", p.package);
    println!("  (cwd: {})", p.dir.display());
    println!();

    // Записываем в кэш до запуска, чтобы при следующем вызове
    // этот проект шёл первым.
    if let Err(e) = write_run_cache(workspace_root, &p.package) {
        eprintln!("⚠️  Не удалось сохранить кэш запуска: {e}");
    }

    let status = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg(&p.package)
        .current_dir(&p.dir)
        .status()
        .map_err(|e| format!("не удалось запустить cargo: {e}"))?;

    if !status.success() {
        return Err(format!("cargo run завершился с кодом {:?}", status.code()));
    }
    Ok(())
}
