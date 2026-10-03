// tv-core/src/lib.rs

use serde::Deserialize;
use std::fs;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub game_capture_name: String,
    pub overlay_path: String,
}

/// События, которые ядро отправляет в UI
pub enum CoreEvent {
    /// Конфиг успешно загружен — UI может использовать пути к ассетам
    ConfigLoaded(Config),
    /// Очередной кадр видеопотока (RGBA, 1280x720)
    Frame(Vec<u8>),
}

pub fn run_core(tx: Sender<CoreEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("config.ron")
        .map_err(|e| format!("Не удалось прочитать config.ron: {e}"))?;
    let config: Config = ron::from_str(&config_str)
        .map_err(|e| format!("Ошибка парсинга RON: {e}"))?;

    println!("[tv-core] Конфигурация загружена: {:?}", config);

    // Сразу отправляем конфиг в UI, чтобы он мог загрузить оверлей
    tx.send(CoreEvent::ConfigLoaded(config.clone()))?;

    println!("[tv-core] Запуск генератора видеопотока (1280x720)...");

    let width = 1280;
    let height = 720;
    let frame_size = width * height * 4;

    let mut counter: usize = 0;

    loop {
        let mut frame_data = vec![0u8; frame_size];
        counter = counter.wrapping_add(1);

        // Генерируем динамическую картинку (анимированный градиент)
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;

                let r = ((x + counter) % 256) as u8;
                let g = ((y + counter / 2) % 256) as u8;
                let b = 150u8;

                frame_data[idx] = r;
                frame_data[idx + 1] = g;
                frame_data[idx + 2] = b;
                frame_data[idx + 3] = 255;
            }
        }

        if tx.send(CoreEvent::Frame(frame_data)).is_err() {
            println!("[tv-core] Канал связи с UI закрыт.");
            break;
        }

        thread::sleep(Duration::from_millis(33)); // ~30 FPS
    }

    Ok(())
}
