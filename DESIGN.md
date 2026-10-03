```markdown
# DESIGN.md — tv

Живой роадмап. Открыл — видишь, что сделано, что дальше.

## Видение

Программа в духе OBS с модульной архитектурой: эффективное headless-ядро для захвата и микширования видео на базе `libobs` и лёгкий нативный предпросмотр через встроенный `obs_display`.

## Архитектура: ядро и предпросмотр

Workspace из двух крейтов:

- **`tv-core`** — headless-ядро: инициализация сессии `libobs`, загрузка источников из конфигурации (`config.ron`), создание сцены, композиция, подключение `obs_display` к нативному окну и (в перспективе) вывод в RTMP.
- **`tv-ui`** — демонстрационный предпросмотр: создаёт нативное Win32-окно, получает его HWND и передаёт в `tv-core`, который подключает к нему `ObsDisplayCreationData`. Никакого графического тулкита — только Win32 и OBS.

Основной запуск настроен на точку входа `tv-ui`, которая поднимает нативное окно и управляет ядром.

## Технологический стек

- **Ядро захвата и композитинга:** `libobs-wrapper` / `libobs` / `libobs-simple`
- **Предпросмотр:** `obs_display_t` через `libobs-wrapper::display`
- **Окно предпросмотра:** сырой Win32 API через крейт `windows` (без GPUI, Bevy и прочих UI-фреймворков)
- **Конфигурация:** `ron` (Rusty Object Notation) + `serde`
- **Макросы источников:** `libobs-simple-macro` + `define_object_manager!` для собственных источников
- **Межпоточный транспорт:** `std::sync::mpsc` — передача событий `CoreEvent` из ядра в UI

## Рендер-пайплайн
```

Win32-окно (tv-ui)
│ HWND
▼
init_obs_with_display(config, hwnd) ← tv-core
│
├── ObsContext::new(StartupInfo)
├── context.scene("Main Scene")
├── ImageSourceBuilder("overlay").set_file(overlay_path)
│ └── .add_to_scene(&mut scene)
├── scene.set_to_channel(0)
└── ObsDisplayCreationData::new(hwnd, 0, 0, 1280, 720)
└── context.display(display_data)
│
▼
OBS рендерит сцену прямо в HWND

````

## Свой источник изображения

`libobs-simple` не имеет готового `ImageSource`, но даёт макрос `define_object_manager!`. Свой источник определяется в несколько строк:

```rust
define_object_manager!(
    #[derive(Debug)]
    struct ImageSource("image_source", *mut libobs::obs_source) for ObsSourceRef {
        #[obs_property(type_t = "string", settings_key = "file")]
        file: String,
    }
);

impl ObsSourceBuilder for ImageSourceBuilder {
    type T = ObsSourceRef;
    fn build(self) -> Result<Self::T, ObsError> {
        let runtime = self.runtime().clone();
        let info = self.object_build()?;
        ObsSourceRef::new_from_info(info, runtime)
    }
}
````

OBS использует встроенный ID `"image_source"` — то есть никакого стороннего плагина не нужно, это стандартный источник OBS, который теперь виден и в UI, и (в перспективе) попадёт в стрим.

## Конфигурация (`config.ron`)

```ron
(
    game_capture_name: "GameCaptureSource",
    overlay_path: "assets/overlay.png",
)
```

Все параметры вынесены в файл, чтобы менять сцену без пересборки.

## Что сделано

- [x] Workspace из двух крейтов (`tv-core`, `tv-ui`)
- [x] Парсинг `config.ron` и передача `Config` в UI через `CoreEvent::ConfigLoaded`
- [x] Инициализация `libobs` (`ObsContext::new`)
- [x] Создание сцены (`context.scene`)
- [x] Собственный источник `ImageSource` через `define_object_manager!`
- [x] Добавление оверлея в сцену (`image_builder.add_to_scene`)
- [x] Активация сцены на канале 0 (`scene.set_to_channel(0)`)
- [x] Создание нативного Win32-окна (крейт `windows`)
- [x] Подключение `obs_display_t` к HWND через `ObsDisplayCreationData`
- [x] **Работающий предпросмотр: сцена OBS рендерится прямо в окно**

## Что дальше

- [ ] Источник захвата игры (`game_capture`) или монитора (`monitor_capture`) — чтобы под оверлеем была не чёрная заливка, а реальная картинка
- [ ] Настройка `z-order` источников в сцене (игра внизу, оверлей сверху)
- [ ] Подключение вывода RTMP через `libobs-simple::output::streaming`
- [ ] Трансляция кадров сцены обратно в UI для отображения счётчиков, состояния стрима
- [ ] Динамическая перезагрузка сцены при изменении `config.ron` без перезапуска
- [ ] Менеджер источников (список в UI, добавление/удаление без правки конфига)
- [ ] Панель аудио-микшера
- [ ] Настройки кодировщика (NVENC / x264 / QSV) и битрейта

## Установка и запуск

```bash
# Собрать и запустить демо-предпросмотр
cargo run

# Собрать только ядро
cargo build -p tv-core

# Собрать только UI-предпросмотр
cargo build -p tv-ui
```

## Зависимости

```toml
# tv-core/Cargo.toml
libobs = "5.0.1"
libobs-simple = "8.0.1"
libobs-wrapper = "9.0.4"
libobs-simple-macro = "7.0.0"
ron = "0.12.2"
serde = { version = "1.0.229", features = ["derive"] }
paste = "1.0"
log = "0.4"

# tv-ui/Cargo.toml
tv-core = { path = "../tv-core" }
serde = { version = "1.0.229", features = ["derive"] }
ron = "0.12.2"
windows = { version = "0.62", features = [
    "Win32_Foundation",
    "Win32_System_LibraryLoader",
    "Win32_UI_WindowsAndMessaging",
] }
```

## Известные ограничения

- OBS требует, чтобы его бинарники (`obs.dll`, плагины, `data/`) лежали в `target/debug/`. Если `cargo clean` удалит `target/debug/`, файлы OBS нужно скопировать обратно из `obs-build/32.0.4/build_out/`.
- Крейт `windows` должен быть версии **0.62** — иначе типы `HWND` не совпадут с теми, что ожидает `libobs-wrapper`, и `obs_display` не подключится.
- Макрос `define_object_manager!` требует **прямых** зависимостей `libobs`, `log`, `paste` и `libobs-simple-macro` в `Cargo.toml` — транзитивных зависимостей через `libobs-wrapper` недостаточно, потому что сгенерированный код обращается к ним напрямую.
- Сцена без источников видео даёт чёрный фон: в текущей версии в сцене только `image_source` с оверлеем.

```

```
