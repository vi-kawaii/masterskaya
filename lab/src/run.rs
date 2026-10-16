//! `lab run` — выбрать и запустить проект из мастерской.

use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const RUN_CACHE_PATH: &str = ".lab_run_cache";

struct RunnableProject {
    package: String,
    dir: PathBuf,
    display: String,
}

pub struct RunOptions {
    pub package: Option<String>,
    pub once: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self { package: None, once: false }
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

    if projects.len() == 1 {
        if opts.once {
            return cargo_run(&projects[0], cwd);
        }
        return run_loop(cwd, &projects, None);
    }

    if opts.once {
        let last_run_package = read_run_cache(cwd);
        let choice = pick(&projects, last_run_package.as_deref())?;
        return cargo_run(&projects[choice], cwd);
    }

    run_loop(cwd, &projects, None)
}

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
            let name = forced_package.as_deref().unwrap();
            projects
                .iter()
                .position(|p| p.package == name || p.display == name)
                .ok_or_else(|| format!("проект '{name}' не найден"))?
        } else {
            pick(projects, last_ref)?
        };

        first = false;
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

fn scan_runnable(cwd: &Path) -> Vec<RunnableProject> {
    const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", ".zed", ".idea"];

    let mut out = Vec::new();

    let walker = WalkDir::new(cwd)
        .into_iter()
        .filter_entry(|e| {
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

        if dir.join("Cargo.toml").exists() {
            if let Some(p) = project_at(dir) {
                out.push(p);
            }
        }
    }

    out.sort_by(|a, b| a.display.cmp(&b.display));
    out
}

fn project_at(dir: &Path) -> Option<RunnableProject> {
    let toml_path = dir.join("Cargo.toml");
    let toml = fs::read_to_string(&toml_path).ok()?;

    let package = extract_package_name(&toml)?;

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

fn read_run_cache(cwd: &Path) -> Option<String> {
    let cache_path = cwd.join(RUN_CACHE_PATH);
    fs::read_to_string(cache_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn write_run_cache(cwd: &Path, package: &str) -> Result<(), String> {
    let cache_path = cwd.join(RUN_CACHE_PATH);
    fs::write(cache_path, package).map_err(|e| format!("не могу записать кэш запуска: {e}"))
}

fn pick(projects: &[RunnableProject], last_run: Option<&str>) -> Result<usize, String> {
    let last_run_original_index = last_run
        .and_then(|last| projects.iter().position(|p| p.package == last));

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

    Ok(order[chosen_pos])
}

fn cargo_run(p: &RunnableProject, workspace_root: &Path) -> Result<(), String> {
    println!();
    println!("▶ cargo run -p {}", p.package);
    println!("  (cwd: {})", p.dir.display());
    println!();

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

/// Публичный скан запускаемых пакетов — для `lab serve`.
pub fn scan_packages_for_serve(cwd: &Path) -> Result<Vec<(String, String)>, String> {
    let projects = scan_runnable(cwd);
    Ok(projects
        .into_iter()
        .map(|p| (p.package, p.display))
        .collect())
}
