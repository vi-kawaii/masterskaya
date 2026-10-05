//! pen-core — ядро редактора рисования.
//! Headless, без UI-зависимостей.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
