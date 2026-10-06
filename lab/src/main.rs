mod commit;
mod dump;
mod paste;

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
            eprintln!("usage: lab <dump|paste|commit>");
            return ExitCode::FAILURE;
        }
    };

    let result = match cmd.as_str() {
        "dump" => dump::run(&cwd),
        "paste" => paste::run(&cwd),
        "commit" => commit::run(&cwd),
        other => {
            eprintln!("❌ lab: неизвестная команда '{other}'");
            eprintln!("   Доступно: dump, paste, commit");
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
