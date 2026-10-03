// tv-ui/src/main.rs

use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tv_core::CoreEvent;

mod assets;

use assets::TvAssets;

struct TvApp {
    event_receiver: mpsc::Receiver<CoreEvent>,
    is_capturing: bool,
    frame_count: usize,
    latest_frame: Option<Vec<u8>>,
    overlay_path: Option<String>,
}

impl TvApp {
    pub fn new(receiver: mpsc::Receiver<CoreEvent>, cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                // Неблокирующая задержка (~30 FPS)
                cx.background_executor()
                    .timer(Duration::from_millis(33))
                    .await;

                let updated = this.update(cx, |this: &mut Self, cx: &mut Context<Self>| {
                    // Забираем все накопившиеся события из канала
                    while let Ok(event) = this.event_receiver.try_recv() {
                        match event {
                            CoreEvent::ConfigLoaded(config) => {
                                println!(
                                    "[tv-ui] Получен путь к оверлею: {}",
                                    config.overlay_path
                                );
                                this.overlay_path = Some(config.overlay_path);
                            }
                            CoreEvent::Frame(data) => {
                                this.frame_count += 1;
                                this.latest_frame = Some(data);
                            }
                        }
                    }
                    cx.notify();
                });

                if updated.is_err() {
                    // Сущность удалена — завершаем задачу
                    break;
                }
            }
        })
        .detach();

        Self {
            event_receiver: receiver,
            is_capturing: true,
            frame_count: 0,
            latest_frame: None,
            overlay_path: None,
        }
    }
}

impl Render for TvApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Достаём цвета из первого кадра для фона
        let mut r_val = 50u8;
        let mut g_val = 100u8;
        let mut b_val = 150u8;

        if let Some(frame) = &self.latest_frame {
            if frame.len() > 200 {
                r_val = frame[0];
                g_val = frame[100];
                b_val = frame[200];
            }
        }

        let dynamic_hex = ((r_val as u32) << 16) | ((g_val as u32) << 8) | (b_val as u32);

        div()
            .v_flex()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .child(
                // ===== Верхняя панель управления =====
                div()
                    .h_12()
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x181825))
                    .child(div().font_bold().child("TV Studio — Workspace (Live Stream)"))
                    .child(
                        Button::new("toggle_capture")
                            .label(if self.is_capturing {
                                "Остановить"
                            } else {
                                "Запустить"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.is_capturing = !this.is_capturing;
                                println!("[tv-ui] Захват переключен: {}", this.is_capturing);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // ===== Область предпросмотра =====
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p_6()
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .border_1()
                            .border_color(rgb(0x313244))
                            .rounded_xl()
                            .bg(rgb(0x11111b))
                            .child(
                                // "Экран" с видео
                                div()
                                    .w(px(800.0))
                                    .h(px(450.0))
                                    .rounded_lg()
                                    .border_2()
                                    .border_color(rgb(0x89b4fa))
                                    .bg(rgb(dynamic_hex))
                                    .shadow_2xl()
                                    .relative() // нужно для наложения оверлея через absolute
                                    .overflow_hidden()
                                    // ---- Слой оверлея (картинка из assets) ----
                                    .child(
                                        if let Some(path) = &self.overlay_path {
                                            div()
                                                .absolute()
                                                .inset_0()
                                                .child(img(path.clone()).size_full())
                                                .into_any_element()
                                        } else {
                                            div().into_any_element()
                                        },
                                    )
                                    // ---- Бейдж "LIVE" сверху ----
                                    .child(
                                        div()
                                            .absolute()
                                            .top_3()
                                            .left_3()
                                            .bg(rgba(0x1e1e2ecc))
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .text_color(rgb(0xa6e3a1))
                                            .text_sm()
                                            .font_bold()
                                            .child("● LIVE STREAM (30 FPS)"),
                                    )
                                    // ---- Счётчик кадров снизу ----
                                    .child(
                                        div()
                                            .absolute()
                                            .bottom_3()
                                            .left_1_2()
                                            // сдвигаем влево на половину ширины (приблизительно)
                                            .ml(px(-120.0))
                                            .bg(rgba(0x11111bcc))
                                            .px_4()
                                            .py_2()
                                            .rounded_lg()
                                            .text_color(rgb(0xcdd6f4))
                                            .child(format!(
                                                "Потоковых кадров получено: {}",
                                                self.frame_count
                                            )),
                                    ),
                            ),
                    ),
            )
    }
}

fn main() {
    let (tx, rx) = mpsc::channel::<CoreEvent>();

    // Запускаем headless-ядро в отдельном потоке
    thread::spawn(move || {
        if let Err(e) = tv_core::run_core(tx) {
            eprintln!("[tv-core] Ошибка выполнения ядра: {e}");
        }
    });

    gpui_kit::application()
        .with_assets(TvAssets)
        .run(|cx| {
            gpui_kit::init(cx);

            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::centered(
                        size(px(1280.0), px(720.0)),
                        cx,
                    )),
                    titlebar: Some(TitlebarOptions {
                        title: Some("TV Workspace".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
                |_, cx| cx.new(|cx| TvApp::new(rx, cx)),
            )
            .expect("Не удалось открыть окно GPUI");
        });
}
