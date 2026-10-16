//! `lab serve` — локальный HTTP-сервер для userscript.
//!
//! Слушает 127.0.0.1:8765, принимает запросы только от chat.deepseek.com
//! с правильным токеном.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Response, Server, StatusCode};

const PORT: u16 = 8765;
const ALLOWED_ORIGIN: &str = "https://chat.deepseek.com";
const TOKEN_FILE: &str = ".lab_serve_config";

struct RunningProc {
    #[allow(dead_code)]
    package: String,
    pid: u32,
    log_path: PathBuf,
    child: Child,
}

#[derive(Default)]
struct State {
    running: HashMap<u32, RunningProc>,
    last_package: Option<String>,
}

type SharedState = Arc<Mutex<State>>;

#[derive(Serialize, Deserialize)]
struct ServeConfig {
    token: String,
}

fn load_or_create_config(root: &Path) -> Result<ServeConfig, String> {
    let path = root.join(TOKEN_FILE);
    if path.exists() {
        let s = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        return serde_json::from_str(&s)
            .map_err(|e| format!("parse {}: {e}", path.display()));
    }

    let token: String = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        (0..32)
            .map(|_| {
                let idx: usize = rng.gen_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect()
    };

    let cfg = ServeConfig { token: token.clone() };
    let s = serde_json::to_string_pretty(&cfg).unwrap();
    fs::write(&path, s).map_err(|e| format!("write {}: {e}", path.display()))?;

    println!();
    println!("════════════════════════════════════════════════");
    println!("  Создан новый токен: {}", path.display());
    println!("  Токен: {token}");
    println!("  Вставь его в userscript (см. README).");
    println!("════════════════════════════════════════════════");
    println!();

    Ok(cfg)
}

#[derive(Deserialize)]
struct ApplyBody {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct PackageBody {
    package: String,
}

#[derive(Deserialize)]
struct StopBody {
    pid: u32,
}

#[derive(Serialize)]
struct ContextResp {
    text: String,
}

#[derive(Serialize)]
struct ApplyResp {
    ok: bool,
    path: String,
}

#[derive(Serialize)]
struct TreeResp {
    files: Vec<String>,
}

#[derive(Serialize)]
struct PackageInfo {
    package: String,
    display: String,
}

#[derive(Serialize)]
struct PackagesResp {
    packages: Vec<PackageInfo>,
    last: Option<String>,
}

#[derive(Serialize)]
struct BuildResp {
    ok: bool,
    exit_code: i32,
    errors: Vec<String>,
    raw: String,
}

#[derive(Serialize)]
struct RunResp {
    ok: bool,
    pid: u32,
    log_path: String,
}

#[derive(Serialize)]
struct RunStatusResp {
    running: bool,
    exit_code: Option<i32>,
    tail: String,
}

#[derive(Serialize)]
struct CommitResp {
    ok: bool,
    message: String,
}

#[derive(Serialize)]
struct ErrorResp {
    error: String,
}

pub fn run(root: &Path) -> Result<(), String> {
    let config = load_or_create_config(root)?;
    let state: SharedState = Arc::new(Mutex::new(State::default()));

    let addr = format!("127.0.0.1:{PORT}");
    let server = Server::http(&addr)
        .map_err(|e| format!("не могу занять {addr}: {e}"))?;

    println!("[serve] http://{addr}");
    println!("[serve] cwd: {}", root.display());
    println!("[serve] Origin разрешён только: {ALLOWED_ORIGIN}");
    println!("[serve] Ctrl+C — выход.");
    println!();

    for mut req in server.incoming_requests() {
        let origin_ok = check_origin(&req);
        let token_ok = check_token(&req, &config.token);

        // OPTIONS preflight не несёт X-Lab-Token, но Origin несёт.
        if req.method() == &Method::Options {
            if origin_ok {
                let _ = req.respond(cors_preflight());
            } else {
                let _ = req.respond(
                    Response::from_string("").with_status_code(StatusCode(403)),
                );
            }
            continue;
        }

        if !origin_ok || !token_ok {
            let _ = req.respond(
                Response::from_string(r#"{"error":"forbidden"}"#)
                    .with_status_code(StatusCode(403)),
            );
            continue;
        }

        let url = req.url().to_string();
        let method = req.method().clone();

        let result: Result<String, String> = match (method, url.as_str()) {
            (Method::Get, "/context") => handle_context(root),
            (Method::Get, "/tree") => handle_tree(root),
            (Method::Get, "/packages") => handle_packages(root, &state),
            (Method::Post, "/apply") => handle_apply(root, &mut req),
            (Method::Post, "/build") => handle_build(root, &mut req),
            (Method::Post, "/run") => handle_run(root, &state, &mut req),
            (Method::Post, "/stop") => handle_stop(&state, &mut req),
            (Method::Post, "/commit") => handle_commit(root),
            (Method::Get, path) if path.starts_with("/run/") => {
                handle_run_status(path, &state)
            }
            _ => Err("not found".into()),
        };

        let response = match result {
            Ok(body) => {
                let r = Response::from_string(body)
                    .with_header(
                        Header::from_bytes("Content-Type", "application/json; charset=utf-8")
                            .unwrap(),
                    );
                cors_ok(r)
            }
            Err(e) => {
                let err = serde_json::to_string(&ErrorResp { error: e }).unwrap();
                let r = Response::from_string(err)
                    .with_status_code(StatusCode(400));
                cors_ok(r)
            }
        };
        let _ = req.respond(response);
    }

    Ok(())
}

fn cors_headers() -> Vec<Header> {
    vec![
        Header::from_bytes("Access-Control-Allow-Origin", ALLOWED_ORIGIN).unwrap(),
        Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap(),
        Header::from_bytes("Access-Control-Allow-Headers", "Content-Type, X-Lab-Token").unwrap(),
        Header::from_bytes("Access-Control-Max-Age", "86400").unwrap(),
    ]
}

fn cors_ok(mut resp: Response<std::io::Cursor<Vec<u8>>>) -> Response<std::io::Cursor<Vec<u8>>> {
    for h in cors_headers() {
        resp = resp.with_header(h);
    }
    resp
}

fn cors_preflight() -> Response<std::io::Cursor<Vec<u8>>> {
    let mut r = Response::from_string("").with_status_code(StatusCode(204));
    for h in cors_headers() {
        r = r.with_header(h);
    }
    r
}

fn check_origin(req: &tiny_http::Request) -> bool {
    for h in req.headers() {
        if h.field.equiv("Origin") {
            let v = h.value.as_str();
            // Origin есть — проверяем строго.
            return v == ALLOWED_ORIGIN || v == format!("{ALLOWED_ORIGIN}/");
        }
    }
    // Origin не пришёл — это GM_xmlhttpRequest от Tampermonkey.
    // Пропускаем дальше, защита остаётся на токене.
    true
}

fn check_token(req: &tiny_http::Request, expected: &str) -> bool {
    for h in req.headers() {
        if h.field.equiv("X-Lab-Token") {
            return h.value.as_str() == expected;
        }
    }
    false
}

fn read_body<T: for<'de> Deserialize<'de>>(
    req: &mut tiny_http::Request,
) -> Result<T, String> {
    let mut s = String::new();
    req.as_reader()
        .read_to_string(&mut s)
        .map_err(|e| format!("read body: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("parse body: {e}\nraw: {s}"))
}

fn handle_context(root: &Path) -> Result<String, String> {
    let text = crate::dump::collect(root)?;
    serde_json::to_string(&ContextResp { text }).map_err(|e| e.to_string())
}

fn handle_tree(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort();
    serde_json::to_string(&TreeResp { files }).map_err(|e| e.to_string())
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    const SKIP: &[&str] = &["target", ".git", "node_modules", "dist", "build", ".idea"];
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if SKIP.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }
            collect_files(root, &path, out)?;
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

fn handle_packages(root: &Path, state: &SharedState) -> Result<String, String> {
    let packages = crate::run::scan_packages_for_serve(root)?;
    let last = state.lock().unwrap().last_package.clone();
    let resp = PackagesResp {
        packages: packages
            .into_iter()
            .map(|(package, display)| PackageInfo { package, display })
            .collect(),
        last,
    };
    serde_json::to_string(&resp).map_err(|e| e.to_string())
}

fn handle_apply(root: &Path, req: &mut tiny_http::Request) -> Result<String, String> {
    let body: ApplyBody = read_body(req)?;
    let safe = sanitize_path(root, &body.path)?;

    if let Some(parent) = safe.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    fs::write(&safe, body.content)
        .map_err(|e| format!("write {}: {e}", safe.display()))?;

    println!("[serve] apply: {}", safe.display());
    let rel = safe
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| body.path.clone());

    serde_json::to_string(&ApplyResp { ok: true, path: rel }).map_err(|e| e.to_string())
}

fn sanitize_path(root: &Path, raw: &str) -> Result<PathBuf, String> {
    if raw.is_empty() {
        return Err("empty path".into());
    }
    let p = Path::new(raw);
    if p.is_absolute() {
        return Err("absolute paths not allowed".into());
    }
    for comp in p.components() {
        use std::path::Component;
        match comp {
            Component::ParentDir => return Err("'..' not allowed".into()),
            Component::Prefix(_) | Component::RootDir => {
                return Err("absolute paths not allowed".into())
            }
            _ => {}
        }
    }

    let joined = root.join(p);
    let parent = joined.parent().ok_or("no parent")?;
    fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    let canon_root = root
        .canonicalize()
        .map_err(|e| format!("canonicalize root: {e}"))?;
    let canon_parent = parent
        .canonicalize()
        .map_err(|e| format!("canonicalize parent: {e}"))?;
    if !canon_parent.starts_with(&canon_root) {
        return Err("path escapes workspace".into());
    }
    Ok(joined)
}

fn handle_build(root: &Path, req: &mut tiny_http::Request) -> Result<String, String> {
    let body: PackageBody = read_body(req)?;
    if !is_known_package(root, &body.package)? {
        return Err(format!("unknown package: {}", body.package));
    }

    println!("[serve] build: {}", body.package);

    let output = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg(&body.package)
        .arg("--message-format=json")
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo build spawn: {e}"))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut errors = Vec::new();
    for line in stdout.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if v.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                if let Some(rendered) = v
                    .get("message")
                    .and_then(|m| m.get("rendered"))
                    .and_then(|r| r.as_str())
                {
                    if !rendered.trim().is_empty() {
                        errors.push(rendered.to_string());
                    }
                }
            }
        }
    }

    let resp = BuildResp {
        ok: exit_code == 0,
        exit_code,
        errors,
        raw: stderr.to_string(),
    };
    serde_json::to_string(&resp).map_err(|e| e.to_string())
}

fn handle_run(
    root: &Path,
    state: &SharedState,
    req: &mut tiny_http::Request,
) -> Result<String, String> {
    let body: PackageBody = read_body(req)?;
    if !is_known_package(root, &body.package)? {
        return Err(format!("unknown package: {}", body.package));
    }

    // Куда пишем лог. Один файл на запуск, имя — с timestamp + имя пакета.
    let log_dir = root.join("target");
    fs::create_dir_all(&log_dir).map_err(|e| format!("mkdir target: {e}"))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let log_path = log_dir.join(format!("lab-run-{}-{}.log", body.package, ts));

    // Открываем файл и клонируем для stderr.
    let f_out = fs::File::create(&log_path)
        .map_err(|e| format!("create log {}: {e}", log_path.display()))?;
    let f_err = f_out.try_clone()
        .map_err(|e| format!("clone log handle: {e}"))?;

    // Запускаем `cargo run` отсоединённо.
    // ВАЖНО: НЕ через `cmd /C ... > file`. Передаём файл в Stdio напрямую,
    // чтобы не было гонки между spawn и созданием файла.
    #[cfg(target_os = "windows")]
    let child = {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

        Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg(&body.package)
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(f_out))
            .stderr(Stdio::from(f_err))
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .spawn()
            .map_err(|e| format!("spawn cargo: {e}"))?
    };

    #[cfg(not(target_os = "windows"))]
    let child = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg(&body.package)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::from(f_out))
        .stderr(Stdio::from(f_err))
        .spawn()
        .map_err(|e| format!("spawn cargo: {e}"))?;

    let pid = child.id();

    println!(
        "[serve] run: {} → pid {pid}, log {}",
        body.package,
        log_path.display()
    );

    {
        let mut st = state.lock().unwrap();
        st.running.insert(
            pid,
            RunningProc {
                package: body.package.clone(),
                pid,
                log_path: log_path.clone(),
                child,
            },
        );
        st.last_package = Some(body.package);
    }

    let resp = RunResp {
        ok: true,
        pid,
        log_path: log_path.to_string_lossy().to_string(),
    };
    serde_json::to_string(&resp).map_err(|e| e.to_string())
}

fn handle_run_status(path: &str, state: &SharedState) -> Result<String, String> {
    let pid_str = path.trim_start_matches("/run/");
    let pid: u32 = pid_str.parse().map_err(|_| "bad pid".to_string())?;

    let mut st = state.lock().unwrap();
    let proc = st.running.get_mut(&pid).ok_or("unknown pid")?;

    match proc.child.try_wait() {
        Ok(Some(status)) => {
            let code = status.code().unwrap_or(-1);
            let tail = tail_file(&proc.log_path, 50).unwrap_or_default();
            st.running.remove(&pid);
            let resp = RunStatusResp {
                running: false,
                exit_code: Some(code),
                tail,
            };
            serde_json::to_string(&resp).map_err(|e| e.to_string())
        }
        Ok(None) => {
            let tail = tail_file(&proc.log_path, 50).unwrap_or_default();
            let resp = RunStatusResp {
                running: true,
                exit_code: None,
                tail,
            };
            serde_json::to_string(&resp).map_err(|e| e.to_string())
        }
        Err(e) => Err(format!("try_wait: {e}")),
    }
}

fn handle_stop(state: &SharedState, req: &mut tiny_http::Request) -> Result<String, String> {
    let body: StopBody = read_body(req)?;
    let mut st = state.lock().unwrap();
    if let Some(p) = st.running.remove(&body.pid) {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &p.pid.to_string(), "/T", "/F"])
                .output();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = p.child.kill();
        }
        println!("[serve] stop: pid {}", body.pid);
        Ok(r#"{"ok":true}"#.to_string())
    } else {
        Err("unknown pid".into())
    }
}

/// Запускает `lab commit` в фоне, потому что git commit с редактором
/// блокирует поток на минуты. HTTP-ответ отдаём сразу.
fn handle_commit(root: &Path) -> Result<String, String> {
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        println!("[serve] commit: старт для {}", root.display());
        match crate::commit::run(&root) {
            Ok(()) => println!("[serve] commit: завершён"),
            Err(e) => eprintln!("[serve] commit: ошибка: {e}"),
        }
    });

    let resp = CommitResp {
        ok: true,
        message: "commit запущен в фоне, смотри вывод в консоли lab serve".into(),
    };
    serde_json::to_string(&resp).map_err(|e| e.to_string())
}

fn tail_file(path: &Path, lines: usize) -> std::io::Result<String> {
    let content = fs::read_to_string(path)?;
    let all: Vec<&str> = content.lines().collect();
    let start = all.len().saturating_sub(lines);
    Ok(all[start..].join("\n"))
}

fn is_known_package(root: &Path, name: &str) -> Result<bool, String> {
    let pkgs = crate::run::scan_packages_for_serve(root)?;
    Ok(pkgs.iter().any(|(p, _)| p == name))
}