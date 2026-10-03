// tv-core/src/lib.rs

use serde::Deserialize;
use std::fs;
use std::sync::mpsc::Sender;

use libobs_simple::define_object_manager;
use libobs_wrapper::{
    context::ObsContext,
    data::ObsObjectBuilder,
    sources::{ObsSourceBuilder, ObsSourceRef},
    utils::{ObsError, StartupInfo},
};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub game_capture_name: String,
    pub overlay_path: String,
}

/// События, которые ядро отправляет в UI
pub enum CoreEvent {
    /// Конфиг успешно загружен — UI может использовать пути к ассетам
    ConfigLoaded(Config),
    /// Текстовый статус (для отладки)
    Status(String),
}

// ---------------------------------------------------------------------------
// Определяем свой источник изображения через макрос define_object_manager.
// OBS использует ID "image_source" для встроенного источника картинок.
// ---------------------------------------------------------------------------
define_object_manager!(
    #[derive(Debug)]
    /// Источник изображения (PNG/JPG) поверх сцены.
    struct ImageSource("image_source", *mut libobs::obs_source) for ObsSourceRef {
        /// Путь к файлу изображения.
        #[obs_property(type_t = "string", settings_key = "file")]
        file: String,
    }
);

// ---------------------------------------------------------------------------
// Реализация трейта ObsSourceBuilder вручную.
// Макрос define_object_manager! генерирует только builder и updater,
// но не реализует ObsSourceBuilder (это делает внутренний impl_default_builder!,
// который недоступен извне).
// ---------------------------------------------------------------------------
impl ObsSourceBuilder for ImageSourceBuilder {
    type T = ObsSourceRef;

    fn build(self) -> Result<Self::T, ObsError> {
        let runtime = self.runtime().clone();
        let info = self.object_build()?;
        ObsSourceRef::new_from_info(info, runtime)
    }
}

pub fn run_core(tx: Sender<CoreEvent>) -> Result<(), Box<dyn std::error::Error>> {
    // ---- 1. Загрузка конфигурации ----
    let config_str = fs::read_to_string("config.ron")
        .map_err(|e| format!("Не удалось прочитать config.ron: {e}"))?;
    let config: Config = ron::from_str(&config_str)
        .map_err(|e| format!("Ошибка парсинга RON: {e}"))?;

    println!("[tv-core] Конфигурация загружена: {:?}", config);
    tx.send(CoreEvent::ConfigLoaded(config.clone()))?;

    // ---- 2. Инициализация OBS ----
    let mut context = ObsContext::new(StartupInfo::default())?;
    // ---- 3. Создание сцены ----
    let mut scene = context.scene("Main Scene", None)?;

    // ---- 4. Добавление оверлея в сцену ----
    let image_builder = ImageSourceBuilder::new(
        "overlay",
        context.runtime().clone(),
    )?
    .set_file(config.overlay_path.clone());

    image_builder.add_to_scene(&mut scene)?;

    println!(
        "[tv-core] Оверлей '{}' добавлен в сцену.",
        config.overlay_path
    );

    // ---- 5. Активация сцены ----
    scene.set_to_channel(0)?;

    tx.send(CoreEvent::Status("Сцена готова".into()))?;

    // ---- 6. Держим поток живым ----
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

use libobs_wrapper::display::{ObsDisplayCreationData, ObsWindowHandle};

/// Инициализирует OBS, создаёт сцену, добавляет оверлей и подключает display к HWND.
pub fn init_obs_with_display(
    config: &Config,
    hwnd_raw: *mut std::ffi::c_void,
) -> Result<ObsContext, Box<dyn std::error::Error>> {
    let mut context = ObsContext::new(StartupInfo::default())?;
    let mut scene = context.scene("Main Scene", None)?;

    let image_builder = ImageSourceBuilder::new(
        "overlay",
        context.runtime().clone(),
    )?
    .set_file(config.overlay_path.clone());

    image_builder.add_to_scene(&mut scene)?;
    scene.set_to_channel(0)?;

    println!("[tv-core] Сцена с оверлеем готова, подключаем display...");

    // Оборачиваем сырой HWND в ObsWindowHandle.
    // Точный API нужно проверить: возможно, ObsWindowHandle::new_from_handle(hwnd),
    // либо ObsWindowHandle::from(hwnd), либо через сырое число.
    let window_handle = unsafe { ObsWindowHandle::new_from_handle(hwnd_raw) };

    // Создаём display: координаты (0, 0), размер 1280x720.
    // create_child = true (по умолчанию) — OBS создаст дочернее окно внутри нашего HWND.
    let display_data = ObsDisplayCreationData::new(
        window_handle,
        0,
        0,
        1280,
        720,
    );

    let display = context.display(display_data)?;

    // Возможно, нужно явно вызвать show, если метод существует.
    // display.show()?;

    println!("[tv-core] Display подключён к HWND.");

    Ok(context)
}
