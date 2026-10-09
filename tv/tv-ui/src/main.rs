// tv-ui/src/main.rs

use windows::{
    core::w,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};
use std::{fs, path::PathBuf};

use tv_core::Config;

/// Корень проекта `tv/` — считается от манифеста крейта `tv-ui`,
/// а не от CWD. `CARGO_MANIFEST_DIR` = D:\rust\tv\tv-ui, один шаг вверх — D:\rust\tv.
fn tv_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tv-ui должен лежать внутри tv/")
        .to_path_buf()
}

/// Разворачивает относительный путь к asset-у в абсолютный от корня tv/.
fn resolve_from_tv_root(p: &str) -> String {
    let path = std::path::Path::new(p);
    if path.is_absolute() {
        p.to_string()
    } else {
        tv_root().join(path).to_string_lossy().into_owned()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tv_root();
    let config_path = root.join("config.ron");

    // Читаем config.ron относительно корня tv/, а не относительно CWD.
    let mut config: Config = ron::from_str(&fs::read_to_string(&config_path)?)?;

    // Делаем overlay_path абсолютным, чтобы tv-core не зависел от CWD.
    config.overlay_path = resolve_from_tv_root(&config.overlay_path);

    println!("[tv-ui] config.ron: {}", config_path.display());
    println!("[tv-ui] overlay:    {}", config.overlay_path);

    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = w!("TvPreviewWindow");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: HINSTANCE(hinstance.0),
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("TV Preview"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1280,
            720,
            None,
            None,
            Some(HINSTANCE(hinstance.0)),
            None,
        )?;

        let _context = tv_core::init_obs_with_display(&config, hwnd.0 as *mut _)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
