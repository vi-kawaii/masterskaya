// tv-ui/src/main.rs

use windows::{
    core::w,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};
use std::fs;

// Используем Config из tv-core, чтобы не было конфликта типов.
use tv_core::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = ron::from_str(&fs::read_to_string("config.ron")?)?;

    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = w!("TvPreviewWindow");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            // HINSTANCE — новый тип, а hinstance — HMODULE. Собираем вручную.
            hInstance: HINSTANCE(hinstance.0),
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        // CreateWindowExW возвращает Result<HWND, Error>, поэтому `?`
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
            // hInstance в CreateWindowExW имеет тип Option<HINSTANCE>
            Some(HINSTANCE(hinstance.0)),
            None,
        )?;

        // Передаём HWND в OBS (пока игнорируется внутри tv-core)
        let _context = tv_core::init_obs_with_display(&config, hwnd.0 as *mut _)?;

        // Цикл сообщений
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
            // Явный unsafe-блок: требование edition 2024.
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => unsafe {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },
    }
}
