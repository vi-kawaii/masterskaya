mod commit;
mod dump;
mod paste;
mod run;
mod serve;

use std::process::ExitCode;

fn main() -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ lab: не могу получить текущую папку: {e}");
            return ExitCode::FAILURE;
        }
    };

    let cmd = match std::env::args().nth(1) {
        Some(c) => c,
        None => {
            eprintln!("usage: lab <dump|paste|commit|run|serve>");
            eprintln!("  dump                     — собрать контекст мастерской в буфер");
            eprintln!("  paste [-r|--restart]     — применить файлы и (опц.) перезапустить проект");
            eprintln!("  commit                   — умный коммит по мастерской");
            eprintln!("  run [--once] [<package>] — запустить проект; --once без цикла");
            eprintln!("  serve                    — поднять локальный сервер для браузера");
            return ExitCode::FAILURE;
        }
    };

    let args: Vec<String> = std::env::args().skip(2).collect();
    let mut run_once = false;
    let mut run_package: Option<String> = None;
    let mut paste_restart = false;

    match cmd.as_str() {
        "paste" => {
            for a in &args {
                match a.as_str() {
                    "-r" | "--restart" => paste_restart = true,
                    other => {
                        eprintln!("❌ lab paste: неизвестный флаг '{other}'");
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
        "run" => {
            for a in &args {
                match a.as_str() {
                    "--once" => run_once = true,
                    other if other.starts_with('-') => {
                        eprintln!("❌ lab run: неизвестный флаг '{other}'");
                        return ExitCode::FAILURE;
                    }
                    other => {
                        if run_package.is_some() {
                            eprintln!("❌ lab run: можно указать только один пакет");
                            return ExitCode::FAILURE;
                        }
                        run_package = Some(other.to_string());
                    }
                }
            }
        }
        _ => {}
    }

    let result = match cmd.as_str() {
        "dump" => dump::run(&cwd),
        "paste" => paste::run(&cwd, paste_restart),
        "commit" => commit::run(&cwd),
        "run" => run::run_with_options(
            &cwd,
            run::RunOptions {
                package: run_package,
                once: run_once,
            },
        ),
        "serve" => serve::run(&cwd),
        other => {
            eprintln!("❌ lab: неизвестная команда '{other}'");
            eprintln!("   Доступно: dump, paste, commit, run, serve");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {e}");
            ExitCode::FAILURE
        }
    }
}
