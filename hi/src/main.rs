use gpui_kit::component::Root;
use gpui_kit::*;

// 1. Состояние приложения.
// Позже сюда ляжет модель данных стикеров (Vec<Sticker>, поиск, теги).
struct StickerApp {}

// 2. Описываем, как состояние превращается в UI.
impl Render for StickerApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))          // тёмный фон
            .text_color(rgb(0xffffff))  // белый текст
            .justify_center()
            .items_center()
            .child(
                div()
                    .text_xl()
                    .child("hi: стикеры")
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child("GPUI Kit каркас работает")
            )
    }
}

// 3. Точка входа.
fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            // init() обязателен ПЕРЕД любым использованием компонентов GPUI Kit
            gpui_kit::init(cx);

            // Окно создаём напрямую, без spawn.
            // spawn возвращает Task, который при игнорировании отменяется —
            // именно поэтому окно не появлялось.
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| StickerApp {});
                // Root должен быть первым уровнем в окне,
                // иначе компоненты будут вести себя непредсказуемо
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("failed to open window");
        });
}
