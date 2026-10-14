# DESIGN.md — animeow

Документ фиксирует жанр, стек, архитектуру и план разработки.
Обновляется по мере движения. Отметки статуса: [ ] — не сделано,
[~] — в работе, [x] — сделано.

1. Что это за игра
   Открытый 3D-мир в духе GTA / Genshin, от третьего лица.

Бесшовный открытый мир с чанковым стримингом.

TPS-камера с орбитой, зумом, коллизией.

Полный locomotion-набор: ходьба, бег, спринт, прыжок, падение,
приземление. Позже — лазание, плавание, глайд.

Гибридный бой: огнестрел + ближний бой + скиллы.

Транспорт (Vehicle) с механикой «сел / вышел».

NPC с AI, диалогами, квестами.

Инвентарь, прогрессия, save/load.

Полноценный UI: меню, HUD, карта, квест-лог, инвентарь, диалоги.

Масштаб на старте: маленький регион (~2×2 км), потом расширяем.

2. Технологический стек
   Слой Решение Обоснование
   Движок Bevy 0.19.1 Уже в проекте
   Физика avian3d 0.7 Кинематический персонаж + динамика + raycast-vehicle
   Контроллер персонажа bevy-tnua 0.32 + bevy-tnua-avian3d 0.12 Floating controller, устраняет ghost collisions
   Загрузка ассетов своя (AssetServer + get_load_state) Полный контроль над прогрессом и переходом
   Анимации glTF (.glb) + AnimationPlayer Стандарт Bevy (единый пайплайн через Blender)
   Навигация своя (waypoint/сетка), позже navmesh Готовых под 0.19 нет
   UI bevy_ui Не плодить зависимости
   Сохранения serde + ron Просто, читаемо, версионируемо
   Аудио bevy_audio Из коробки
   Стриминг чанков своя система поверх AssetServer Готовых нет

3. Архитектура
   3.1. Стейт-машина
   GameState:
   Boot
   MainMenu
   Loading
   InGame
   Paused
   Dialog
   Cutscene
   GameOver

InGameState (подстейт InGame):
Playing
Paused
Inventory
Map
QuestLog
Dialog
Cutscene

Фактически реализовано:
GameState: Loading → InGame. Остальные пока не используются.
InGameState: Playing ⇄ Paused (по Esc).

3.2. Модульная структура
src/
main.rs
states.rs # GameState + подстейты
types.rs # общие компоненты/ресурсы
core/ # фундамент
camera.rs # CameraRig, режимы
input.rs # абстракция ввода
physics.rs # PhysicsPlugins + Gravity + Tnua + конфиг схемы
debug.rs # отладочный текст с координатами
save.rs # save/load
audio.rs # аудио-менеджер
world/ # мир
chunk.rs # чанки
streaming.rs # подгрузка/выгрузка
level.rs # сборка уровня
actor/ # сущности
character.rs # общий персонаж (MovementStats, Health)
player.rs # игрок
npc.rs # NPC
vehicle.rs # транспорт
shoot.rs # стрельба
abilities.rs # скиллы
health.rs # HP/статусы
controller/ # управление
mod.rs
player.rs # ввод игрока → TnuaController
bot.rs # AI
scheme.rs # CharacterControlScheme + Config
vehicle.rs # ввод машины
camera.rs # управление камерой
quest/ # квесты
mod.rs
data.rs
tracker.rs
inventory/ # инвентарь
ui/ # UI
hud.rs
menu.rs
dialog.rs
inventory_ui.rs
map.rs
quest_log.rs
modes/ # игровые режимы
loading.rs # загрузка (ручной счёт, прогресс-бар)
pause.rs # пауза (Time<Virtual>::pause)
game/

4. Ключевые подсистемы
   4.1. Мир
   Чанки фиксированного размера (например, 64×64 м).

Каждый чанк — glTF/GLB-сцена + метаданные (NPC, спавны, триггеры).

Активны только чанки вокруг игрока + буфер. Остальные выгружаются.

LOD для дальних объектов.

Освещение: DirectionalLight + динамические источники, тени в ближней зоне.

**Рельеф и ходьба по нему.**
- Склоны: `TnuaBuiltinWalkConfig::max_slope_angle` (уточнить имя в 0.32).
- Ступеньки: автоподъём через raycast вперёд-вниз (Tnua сам умеет).
- Спуск без отрыва: `cling_distance` в конфиге.
- Рельеф: гибрид — крупные формы дизайнер лепит в Blender, детали
  (трава, камни) — процедурно. Формат пока не зафиксирован.

**Коллайдеры земли: trimesh vs heightmap vs примитивы.**

Историческая справка. Раньше персонаж использовал
`RigidBody::Kinematic + MoveAndSlide + CustomPositionIntegration`.
Это давало ghost collisions на тримешах: персонаж «парил» над
ровными участками из-за ложных контактов на внутренних рёбрах.
`FIX_INTERNAL_EDGES` помогал частично, но не полностью.

Решение: перешли на **floating character controller bevy-tnua**.
Принципиально другой подход: персонаж — `RigidBody::Dynamic`,
Tnua держит его **центр** на высоте `float_height` над землёй
и не пытается «прижаться» низом капсулы к тримешу. Ghost
collisions исчезают архитектурно, а не костылями.

Остров (`main.glb`) остаётся **trimesh-коллайдером** с
`FIX_INTERNAL_EDGES` — это нормально для статики, Tnua сам
разбирается со своими сенсорами.

Сравнение подходов (для истории):
| | Trimesh | Heightmap | Примитивы (cuboid/capsule) |
|---|---|---|---|
| Что это | Произвольный набор треугольников | Регулярная 2D-сетка высот | Выпуклые формы |
| Топология | Неизвестна движку | Известна (сетка) | Известна (форма) |
| Ghost collisions | Фундаментальны для Kinematic | Почти нет | Нет |
| Нависания, пещеры | Да | Нет | Да |
| Где использовать | Скалы, здания, машины | Террейн, ландшафт | Геймплейные зоны, пол |
| Поддержка в avian3d 0.7 | Да | Нет из коробки | Да |

4.2. Персонаж
**Контроллер: bevy-tnua (floating controller).**
Персонаж — `RigidBody::Dynamic` с капсулой `Collider::capsule(0.4, 1.0)`.
Tnua держит центр капсулы на высоте `float_height` над землёй,
используя сенсор `TnuaAvian3dSensorShape(Collider::sphere(0.1))`.
Ввод: WASD относительно направления камеры.

**Состояния locomotion:** idle / walk / run / sprint / jump / fall / land.
Пока не реализованы явно (нет анимаций), но база для них — в схеме.

**Поворот.** Tnua сам крутит тело через `desired_forward` в
`TnuaBuiltinWalk`. Модель-ребёнок наследует rotation родителя.

**Разделение физики и визуала.** Капсула (`RigidBody::Dynamic`)
несёт трансформ позиции и вращения. Модель — её ребёнок, имеет
локальный offset по Y и локальный yaw-offset (см. ниже).

**Спавн игрока — по событию готовности коллайдеров.**
Игрок не спавнится в `OnEnter(InGame)` напрямую. Вместо этого:
- `spawn_island` вызывается из `check_loading` (Loading), когда
  ассеты загружены.
- `spawn_island` создаёт свет и остров с `WorldAssetRoot`.
- Параллельно ждут два события: `WorldInstanceReady` (сцена
  инстанцирована) и `ColliderConstructorHierarchyReady`
  (коллайдеры построены).
- Когда оба — `finish_loading` переключает `Loading → InGame`,
  удаляет Loading UI и меняет ClearColor на мировой.

**Схема управления: `CharacterControlScheme`.**
Объявлена в `controller/scheme.rs`. Базис — `TnuaBuiltinWalk`,
действие — `Jump(TnuaBuiltinJump)`. Макрос `#[derive(TnuaScheme)]`
генерирует `CharacterControlSchemeConfig` — ассет конфига с полями
`basis: TnuaBuiltinWalkConfig` и `jump: TnuaBuiltinJumpConfig`.

Конфиг-хендл хранится в ресурсе `CharacterControlConfigHandle`
и вешается на игрока через `TnuaConfig::<CharacterControlScheme>(handle)`.
**Без конфига Tnua молча пропускает контроллер** — персонаж стоит,
но не двигается.

**Текущий конфиг (под капсулу 0.4×1.0):**
- `basis.float_height = 0.80` — ниже полвысоты капсулы (0.9),
  чтобы низ капсулы уходил в землю на ~0.1 (floating controller
  всегда держит центр, а не низ).
- `basis.cling_distance = 0.05` — прилипание на спусках.
- `basis.spring_strength = 100.0`, `spring_dampening = 1.0`.
- `basis.speed = 6.0` — максимальная скорость, м/с.
- `basis.acceleration = 1000.0`, `air_acceleration = 100.0` —
  почти мгновенный старт/стоп.
- `jump.height = 1.2`, `jump.takeoff_extra_gravity = 15.0`.

**Сенсор:** `TnuaAvian3dSensorShape(Collider::sphere(0.1))`.
Маленькая сфера даёт точное «прощупывание» земли.

**Смещение модели внутри капсулы.** Модель — ребёнок капсулы,
локальный `Transform::from_xyz(0.0, -0.92, 0.0).with_rotation(yaw)`.
Y подбирается эмпирически, шаг 0.01–0.02. Правило: утопает → Y
ближе к `-0.88`; парит → Y ближе к `-0.95`.

**Yaw-компенсация модели.** `MODEL_YAW_OFFSET = PI` в
`spawn_player_entity`. Модель `anime-girl.glb` экспортирована
с forward в **-Z**, а Tnua использует **+Z**. Временный костыль:
при следующем импорте модели привести forward к +Z в Blender
(нос/лицо смотрят в **-Y** в Blender = +Z в Bevy) и убрать
константу.

**Правила приведения модели (Blender → Bevy):**
- Metric, Meters, Scale 1.0.
- Высота 1.8 м по Z (под капсулу `Collider::capsule(0.4, 1.0)`).
- Origin между ступней, по центру X/Y.
- Forward в **-Y** в Blender = **+Z** в Bevy.
- Rotation/scale/location применены (`Ctrl+A → All Transforms`).
- Empty между арматурой и мешами убраны, трансформы схлопнуты.

**Ввод (InputState).** Абстракция ввода из `core/input.rs`:
`move_dir: Vec2`, `sprint: bool`, `jump_pressed: bool`.
Заполняется в `Update`, читается в `FixedUpdate` из `tnua_movement`.

4.3. Камера
CameraRig (родитель) + Camera3d (ребёнок).

Yaw/pitch от мыши, зум колесом, коллизия через raycast.

**Сглаживание — только ввод, не слежение.**
- Риг жёстко привязан к игроку + `EYE_HEIGHT`.
- Мышь двигает `target_yaw`/`target_pitch`, система `orbit_camera`
  плавно ведёт текущие значения к целям через
  `alpha = 1 - exp(-k·dt)`.
- Зум — через `target_distance`.
- `camera_smoothing = 25.0` (1/с).

4.4. Бой
Гибрид: огнестрел (hitscan + снаряды) + ближний бой + скиллы.

Оружие — компонент: Weapon { damage, cooldown, range, ammo, kind }.

Скиллы — компонент: Ability { cooldown, cost, effect }.

Элементы/статусы — позже.

4.5. Транспорт
Физика: raycast-vehicle на avian3d.

Механика «сел / вышел»: переключение управляемой сущности.

Камера переключается между режимами.

Типы: машина → потом мото, лодка и т.д.

4.6. NPC и AI
State machine: idle / patrol / alert / chase / attack / flee / talk.

Навигация: waypoint/сетка → позже navmesh.

Толпы: спавн/деспавн по чанкам.

Диалоги: система + UI + стейт Dialog.

4.7. Квесты и прогрессия
Квесты как данные (ron/json): id, шаги, условия, награды.

Инвентарь: предметы, оружие, ресурсы.

Прогрессия: уровни, статы, скиллы.

Всё сериализуемо.

4.8. UI
MainMenu, Pause, Settings, Inventory, Map, QuestLog, Dialog, HUD.

HUD: HP, стамина, патроны, миникарта, компас, квест-трекер.

4.9. Сохранения
Сериализация: позиция игрока, дельта мира, инвентарь, квесты, флаги.

Формат: ron, версионирование.

Слоты сохранений.

4.10. Производительность
Чанки, LOD, frustum culling, occlusion — позже.

Пул сущностей для пуль и эффектов.

Профилирование через bevy_dev_tools + Tracy при необходимости.

**Отладочный визуал.** `PhysicsDebugPlugin` из avian3d рисует
коллайдеры линиями. Включён всегда — на ранних этапах важнее
видеть коллизии. Для релиза обернуть в `#[cfg(debug_assertions)]`.

**Известный визуальный артефакт — промежуток между Loading UI
и появлением моделей.** `WorldInstanceReady` и
`ColliderConstructorHierarchyReady` срабатывают раньше, чем меши
и материалы доходят до GPU. Визуально: Loading UI исчезает, на
секунду виден пустой ClearColor, потом появляются модели.
Причина — асинхронная подготовка `RenderAssetUsages::RENDER_WORLD`
в Bevy. Оставлено как есть до отдельной итерации полировки.

4.11. Загрузка
**Ручной счёт через AssetServer.** `bevy_asset_loader` и
`iyes_progress` **не используются** — их API в 0.27/0.17
несовместим с Bevy 0.19 без правок. Вместо них — своя система.

**Как работает:**
- `LoadingPlugin` на `OnEnter(Loading)` спавнит UI и стартует
  загрузку через `asset_server.load("models/...")`.
- Хендлы складываются в ресурс `LoadingHandles` (Vec + счётчик).
- `check_loading` в `Update` (только в Loading) проверяет
  `asset_server.get_load_state(handle)` для каждого хендла.
- Прогресс = `done / total`.
- Когда все загружены и прошло `min_display` (0.5 с) —
  вызывается `spawn_island`.
- Дальше ждём `IslandReady` (сцена + коллайдеры).
- `finish_loading` переключает `Loading → InGame`, удаляет
  Loading UI/камеру, меняет `ClearColor` с чёрного на мировой.

**Camera2d для Loading.** В animeow нет камеры до `InGame`,
поэтому Loading UI не рендерится. Добавлен `LoadingCamera`
(Camera2d) на `Startup`, удаляется в `finish_loading`.

**Цвета фона.** `ClearColor` глобальный, не per-camera.
- Старт и Loading: `Color::BLACK` (из `main.rs`).
- InGame: `Color::srgb(0.05, 0.05, 0.1)` (устанавливается в
  `finish_loading`).

Граница между Loading и InGame видна по смене фона.

4.12. Пауза
Esc в `InGameState::Playing` → `InGameState::Paused`. Esc обратно.

**Пауза времени.** `Time<Virtual>::pause()` на входе,
`Time<Virtual>::unpause()` на выходе. Это останавливает **всё**,
что зависит от `Time`: физику avian, Tnua, анимации, cooldown-таймеры.

`Time<Virtual>` управляет и `Time`, и `Time<Fixed>`. Когда
виртуальное время на паузе, `FixedUpdate` получает delta = 0,
и avian не двигает тела.

**Что это даёт:** прыжок в воздухе → Esc → персонаж замирает
в воздухе. Esc → падение продолжается с той же точки.

5. Что явно НЕ делаем на старте
   ❌ Мультиплеер.

❌ Полноценный navmesh (сначала waypoint/сетка).

❌ Моды / скриптинг для игроков.

❌ Продвинутая графика (SSAO, SSR, GI) — только базовый PBR + тени.

❌ Открытый мир масштаба GTA V — начинаем с 2×2 км.

❌ Деформация / разрушение окружения.

6. Риски
   Риск Митигация
   Стриминг чанков — сложно 1 чанк → 4 → ring-буфер
   Производительность AI толп ECS Bevy + LOD для AI
   Save/load всего мира Сериализовать только дельту
   Анимации + физика Анимация визуальная, физика через Tnua
   Разрастание модулей Строго следовать структуре из §3.2
   Ходьба по рельефу — много нюансов Начать с debug-полигона
   (холм, стенка, ступеньки, наклон), потом чанки
   Ghost collisions на trimesh Решено переходом на Tnua (§4.1)
   Floating controller даёт визуальный зазор Настроен float_height
   ниже полвысоты капсулы + offset модели (§4.2)
   Промежуток Loading → модели на экране Отложено, требует
   проверки готовности RenderAssetUsages (§4.10)

7. План работ
   Каждый этап заканчивается играбельным билдом.
   Не начинать следующий, пока предыдущий не запускается.

Этап 0. Фиксация решений
☑ Жанр, стек, архитектура зафиксированы в DESIGN.md

Этап 1. 3D-фундамент
Цель: пол, свет, камера, управляемый куб.

☑ Cargo.toml: добавить avian3d
☑ main.rs: Camera3d + DirectionalLight
☑ types.rs: расширить Player, GameSettings
☑ controller/player.rs: переписать под 3D
☑ modes/game/mod.rs: убрать/переделать Triangle под 3D
☑ modes/vehicle.rs: временно отключить

Этап 2. TPS-камера + персонаж
Цель: ходить/бегать/прыгать, камера не застревает.

☑ core/camera.rs: CameraRig, орбита, pitch clamp, захват курсора
☑ core/camera.rs: зум колесом с clamp
☑ core/camera.rs: плавность ввода — через `target_*` + сглаживание
☑ core/input.rs: абстракция ввода
☑ actor/character.rs: общие компоненты
☑ core/physics.rs: PhysicsPlugins + Gravity + PhysicsDebugPlugin
☑ controller/scheme.rs: CharacterControlScheme + Config
☑ controller/player.rs: tnua_movement — заполняет TnuaController
☑ core/physics.rs: TnuaControllerPlugin + TnuaAvian3dPlugin
☑ core/physics.rs: конфиг схемы как ассет + handle в ресурсе
☑ modes/game/mod.rs: spawn_player_entity с TnuaController,
  TnuaConfig, TnuaAvian3dSensorShape, LockedAxes
☑ modes/game/mod.rs: IslandReady (scene + colliders) → finish_loading
☑ core/debug.rs: отладочный текст с координатами

☑ Этап 2.5. Загрузка и пауза
☑ modes/loading.rs: ручной счёт через AssetServer
☑ modes/loading.rs: прогресс-бар с процентами
☑ modes/loading.rs: Camera2d для рендера UI
☑ main.rs: start_loading → check_loading → spawn_island
☑ modes/game/mod.rs: on_scene_ready + on_island_ready → IslandReady
☑ modes/game/mod.rs: finish_loading — переход в InGame
☑ main.rs: ClearColor чёрный на старте
☑ modes/game/mod.rs: ClearColor мировой в finish_loading
☑ modes/pause.rs: Time<Virtual>::pause на Esc

[ ] states.rs: locomotion-стейты (idle/run/jump/fall)
[ ] Анимации персонажа (AnimationPlayer + выбор клипа по состоянию)

Этап 3. Мир и стриминг
Цель: уровень, текстуры, тени, чанки.

□ Debug-полигон: холм, стенка, ступеньки, наклон
□ Ходьба по склонам: настройка TnuaBuiltinWalkConfig
□ Snap to ground на спуске (cling_distance)
□ world/chunk.rs: структура чанка
□ world/streaming.rs: подгрузка/выгрузка по радиусу
□ world/level.rs: сборка уровня
□ PBR-материалы, тени, освещение

Этап 4. Бой + AI + HUD
Цель: стрелять, попадать в ботов, видеть HP.

□ actor/health.rs
□ actor/shoot.rs: Weapon, hitscan
□ controller/bot.rs: state machine idle/patrol/chase/attack
□ ui/hud.rs: HP, патроны, прицел

Этап 5. Vehicle
Цель: сесть в машину, поехать, выйти.

□ actor/vehicle.rs
□ controller/vehicle.rs
□ физика raycast-vehicle
□ механика «сел / вышел»
□ переключение камеры

Этап 6. NPC + диалоги + квесты
Цель: поговорить с NPC, взять квест, выполнить.

□ actor/npc.rs
□ quest/data.rs, quest/tracker.rs
□ ui/dialog.rs
□ ui/quest_log.rs
□ стейт Dialog

Этап 7. Инвентарь + прогрессия
Цель: подбирать, использовать, развиваться.

□ inventory/ модуль
□ ui/inventory_ui.rs
□ уровни, статы, скиллы

Этап 8. Меню + карта + save/load
Цель: полноценное меню, сохранения.

□ ui/menu.rs (MainMenu, Pause, Settings)
□ ui/map.rs
□ core/save.rs
□ слоты сохранений

Этап 9. Полировка (постоянно)
□ Анимации персонажей и NPC
□ Звук, музыка
□ Эффекты, частицы
□ LOD, оптимизация
□ Баланс, контент
□ Промежуток Loading → модели (§4.10)

8. Порядок изменений в существующем коде
   states.rs — оставить, расширить подстейтами по §3.1.
   types.rs — расширить.
   controller/player.rs — вокруг TnuaController.
   controller/scheme.rs — CharacterControlScheme + конфиг.
   core/physics.rs — PhysicsPlugins + Tnua + ассет конфига.
   modes/loading.rs — ручной счёт через AssetServer + UI.
   modes/pause.rs — Time<Virtual>::pause.
   modes/game/mod.rs — IslandReady, finish_loading.
   actor/*, controller/bot.rs, modes/game/{dialog,fight,hud,move_}.rs — пустые, наполнять по этапам.

9. Журнал решений
   Короткие записи, почему сделано именно так. Не удалять — контекст.

**Переход с MoveAndSlide на bevy-tnua.** Изначально контроллер
был на `RigidBody::Kinematic + CustomPositionIntegration +
MoveAndSlide`. Это давало ghost collisions на тримешах. Решение —
floating controller bevy-tnua. Tnua держит центр капсулы на высоте
над землёй, а не двигает тело «до контакта». Ghost collisions
исчезают архитектурно. Минус — визуальный зазор, устраняется
понижением `float_height` и подгонкой offset модели.

**Версия Tnua — 0.32 (для Bevy 0.19).** Версии 0.24 и раньше
собраны под Bevy 0.16, их нельзя использовать — конфликт двух
версий Bevy в графе зависимостей. Правильные версии:
`bevy-tnua = "0.32"`, `bevy-tnua-avian3d = "0.12"`.

**Схема управления обязательна.** `TnuaController` параметризован
схемой (`TnuaScheme`). Макрос `#[derive(TnuaScheme)]` генерирует
`CharacterControlSchemeConfig` — ассет конфига. Он **не имеет
`Default`** — надо создавать вручную.

**`TnuaConfig<S>(handle)` обязателен.** Без него
`apply_controller_system` делает `else { continue; }` и молча
пропускает контроллер. Симптом: персонаж стоит, ошибок нет.

**`controller.basis` — поле, не метод.** В 0.32 присваивается
напрямую: `controller.basis = TnuaBuiltinWalk { ... };`.

**`controller.action(...)` требует `initiate_action_feeding()`.**
Иначе паника. Парного «end» нет: флаг сбрасывается внутри
`apply_controller_system` в конце кадра.

**`action(...)` принимает вариант схемы.** Не `TnuaBuiltinJump`,
а `CharacterControlScheme::Jump(TnuaBuiltinJump::default())`.

**Три компонента для сенсора.** Tnua создаёт сенсоры сама, но
форму задаёт `TnuaAvian3dSensorShape`. Маленькая сфера
(`Collider::sphere(0.1)`) даёт точное «прощупывание» земли.
Капсула радиуса 0.4 «видит» землю раньше низа капсулы.

**Floating controller всегда держит центр, а не низ.** Архитектура.
Компромисс: `float_height` ниже полвысоты капсулы (0.80 против
0.90), `cling_distance` = 0.05, offset модели = `-0.92`.

**`LockedAxes` через билдер в avian3d 0.7.**
`LockedAxes::new().lock_rotation_x().lock_rotation_z()`.

**`Grounded` удалён.** Tnua сам отслеживает состояние через
`TnuaController::is_airborne()`.

**Разворот модели — через Tnua `desired_forward`.**
`Dir3::new(move_dir).ok()` — `None`, когда ввода нет.

**Yaw-компенсация модели — временный костыль.**
`MODEL_YAW_OFFSET = PI`. Модель экспортирована с forward в -Z.
Правильно — пересобрать модель в Blender.

**Loading — ручной счёт, а не bevy_asset_loader.**
`bevy_asset_loader 0.27` и `iyes_progress 0.17` несовместимы
с Bevy 0.19 без правок (миграция API). Вместо них — свой ресурс
`LoadingHandles` и `asset_server.get_load_state`.

**Camera2d для Loading обязательна.** В animeow нет камеры до
InGame, UI без камеры не рендерится. `LoadingCamera` (Camera2d)
спавнится на `Startup`, удаляется в `finish_loading`.

**Цвет фона различается для Loading и InGame.** Старт и Loading:
чёрный. InGame: `srgb(0.05, 0.05, 0.1)`. `ClearColor` — глобальный
ресурс, переключается в `finish_loading`. Граница видна визуально.

**IslandReady — два события вместо одного.**
`WorldInstanceReady` (сцена инстанцирована) и
`ColliderConstructorHierarchyReady` (коллайдеры построены) —
оба нужны. Одна система `finish_loading` ждёт `is_ready()` и
делает переход. Не разбрасывать проверки по обсерверам — решает
проблему порядка событий.

**Пауза через `Time<Virtual>::pause()`.** Останавливает всё, что
зависит от `Time`: физику avian, Tnua, анимации, cooldown.
`FixedUpdate` получает delta = 0, тела не двигаются.

**Промежуток Loading UI → модели — известный артефакт.**
`WorldInstanceReady` срабатывает раньше, чем меши/материалы
доходят до GPU. Визуально: Loading UI исчез, секунда пустого
фона, потом модели. Отложено. Решения: (1) проверять
`Assets::contains` для `Mesh3d`/`Image`, (2) ждать N кадров,
(3) спец-хук на готовность рендера.

10. Инструменты разработки (lab)
    Воркфлоу сборки контекста и применения ответов DeepSeek
    автоматизирован через `lab` (см. `lab/` и `ARCHITECTURE.md`
    §«lab — тулинг мастерской»).

Команды, которые используются каждый день:

- `lab dump` — собирает дерево и содержимое всех исходников
  (`*.rs`, `*.toml`, `*.md`, …) в буфер обмена. Отправляется в
  DeepSeek как контекст. Игнорирует `target/`, `.git/`,
  `node_modules/`, `dist/`, `build/`, `.idea/`.

- `lab paste` — слушает буфер обмена. Как только в буфере
  появляется блок вида
  ```
  FILE путь/к/файлу
