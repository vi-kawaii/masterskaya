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
InGameState: Playing ⇄ Paused (по Esc), Dialog (открывается по T).

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
vehicle.rs # транспорт (визуальный каркас)
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
**Статус: визуальный каркас готов.** Машина спавнится рядом
с игроком, стоит на земле (пока — как `RigidBody::Dynamic`
с кубическим коллайдером, без подвески и управления).

**Модели.** Кузов — `fast-car.glb`, колёса — `wheel1.glb`,
`wheel2.glb`, `wheel3.glb`, `wheel4.glb`. Кузов — отдельный
glTF, колёса — отдельные glTF. В `.blend`-файле
(`art/models/fast-car.blend`) вся машина собрана целиком,
экспорт делается по частям через `Include → Selected Objects`.

**Иерархия в ECS.**
```
VehicleRoot (RigidBody::Dynamic, Collider::cuboid, Transform)  ← физический корень
├── VehicleBodyModel (WorldAssetRoot("fast-car.glb"), rotation=180° по Y)
├── Wheel (Transform, Wheel { steerable, powered })              ← переднее левое
│   └── WheelModel (WorldAssetRoot("wheel1.glb"))
├── Wheel                                                         ← переднее правое
│   └── WheelModel (WorldAssetRoot("wheel2.glb"))
├── Wheel                                                         ← заднее левое
│   └── WheelModel (WorldAssetRoot("wheel3.glb"))
└── Wheel                                                         ← заднее правое
    └── WheelModel (WorldAssetRoot("wheel4.glb"))
```

**Почему так:**
- `VehicleRoot` — невидимое физическое тело. Модели — его дети,
  отдельно от физики.
- `Wheel` — пустой узел, точка для будущего raycast'а подвески
  и держатель модели колеса. Позиция колеса задаётся здесь.
- `WheelModel` — визуал колеса, `Transform::default()` (модель
  в `.glb` уже в (0,0,0) с origin в центре).

**Компоненты (`actor/vehicle.rs`):**
- `VehicleRoot` — маркер корня.
- `VehicleBodyModel` — маркер узла модели кузова.
- `WheelModel` — маркер узла модели колеса.
- `Wheel { steerable: bool, powered: bool }` — пустой узел
  колеса. `steerable` — переднее (поворачивается рулём),
  `powered` — ведущее (получает силу мотора).

**Экспорт из Blender (правила):**
- `.blend` содержит всю машину собранной.
- Кузов: `Ctrl+A → All Transforms`, `Set Origin → Origin to Geometry`.
- Колёса: `Ctrl+A → All Transforms`, `Set Origin → Origin to Geometry`,
  `Alt+G` перед экспортом (обнулить позицию), `Ctrl+Z` после —
  чтобы в `.glb` колесо приехало в (0,0,0).
- Экспорт с `Include → Selected Objects`, `+Y Up`.
- Порядок: `wheel1.glb` → FL, `wheel2.glb` → FR, `wheel3.glb` → BL,
  `wheel4.glb` → BR.

**Координаты колёс (из Blender-дампа `fast-car.blend`).**
В осях Bevy (X влево/вправо, Y вверх/вниз, Z вперёд/назад):
```
FL: [ 0.953, -0.482, -1.234]
FR: [-0.953, -0.482, -1.234]
BL: [ 0.953, -0.482,  1.534]
BR: [-0.953, -0.482,  1.534]
```
Перёд машины в Blender смотрит в −Y → после экспорта `+Y Up`
это −Z в Bevy. Модель кузова развёрнута на 180° вокруг Y,
чтобы «нос» смотрел в +Z (стандарт Bevy/Tnua).

**Габариты кузова (из дампа):**
- Blender dims (X, Y, Z) = (2.01, 4.89, 1.32)
- Bevy dims (X, Y, Z) = (2.01, 1.32, 4.89)
- Полуразмеры коллайдера: `(1.00, 0.66, 2.44)`.

**Что ещё НЕ сделано:**
- [ ] Подвеска (raycast от каждого `Wheel` вниз, пружина + демпфер).
- [ ] Движение (газ/тормоз → сила к ведущим колёсам).
- [ ] Поворот руля (`steerable`-колёса вращаются вокруг Y).
- [ ] Кручение колёс (визуальное, по скорости).
- [ ] Механика «сел / вышел» (переключение управляемой сущности).
- [ ] Камера для режима вождения.
- [ ] Переключение `TnuaController` игрока off, когда он в машине.

**Планы по физике:**
Физика: raycast-vehicle на avian3d. Машина — `RigidBody::Dynamic`
(уже так). Каждое колесо пускает луч вниз, пружина + демпфер
держат кузов на высоте. Ведущие колёса получают силу вдоль
своего forward. `steerable`-колёса поворачиваются вокруг Y.

Типы: машина → потом мото, лодка и т.д.

4.6. NPC и AI
State machine: idle / patrol / alert / chase / attack / flee / talk.

Навигация: waypoint/сетка → позже navmesh.

Толпы: спавн/деспавн по чанкам.

**Диалоги.**
Система диалогов — отдельный подстейт `InGameState::Dialog`,
живёт в `modes/game/dialog/`. Запускается по триггеру (пока —
клавиша `T` в `Playing`, позже — приближение к NPC + `E`).

**Модель данных (`assets/dialogs/*.ron`).**
Диалог — дерево узлов. Каждый узел — текст + список опций,
каждая опция ведёт к `next_node`. Пустой `next_node` или
отсутствующий узел = конец диалога.

```ron
(
    id: "npc_test_1",
    start_node: "start",
    nodes: {
        "start": (
            text: "Hello, traveler!",
            options: [
                (text: "I need help.",     next_node: "help"),
                (text: "Just passing by.", next_node: "bye"),
            ],
        ),
        "help": (
            text: "...",
            options: [],
        ),
        "bye":  (
            text: "...",
            options: [],
        ),
    },
)
```

**Стейт `InGameState::Dialog`.**
- `OnEnter(Dialog)` — спавн UI (`setup_dialog_ui`).
- `Update` в `Dialog` — `handle_dialog_input` + `update_dialog_ui`.
- `Update` в `Playing` — `start_debug_dialog` (триггер открытия).
- `OnExit(Dialog)` — удаление UI (`cleanup_dialog_ui`).
- Esc внутри диалога обрабатывается самим диалогом, не паузой
  (см. `modes/pause.rs`, ветка `InGameState::Dialog => {}`).

**Ввод и UI.**
- Пока: `Digit1` / `Digit2` — выбор опции. `Esc` — закрыть.
- UI — простая колонка: текст узла + список опций,
  полупрозрачный чёрный фон внизу экрана.
- Задел на «много ответов»: `handle_dialog_input` легко
  расширяется до `Digit1..Digit9`. UI рисует столько
  кнопок, сколько опций в узле.

**Что НЕ сделано (см. план):**
- Загрузка `.ron` через `bevy_common_assets` (сейчас диалог
  захардкожен в `start_debug_dialog`).
- Триггер от NPC (расстояние + кнопка `E` вместо `T`).
- Ветвление по флагам (`condition` в опции, `on_enter` в узле).
- Реплики NPC vs реплики игрока (сейчас всё — один поток текста).
- Влияние выборов на мир (квесты, отношения, репутация).
- Локализация / кириллица (нужен TTF-шрифт с юникодом).

4.7. Квесты и прогрессия
Квесты как данные (ron/json): id, шаги, условия, награды.

Инвентарь: предметы, оружие, ресурсы.

Прогрессия: уровни, статы, скиллы.

Всё сериализуемо.

4.8. UI
MainMenu, Pause, Settings, Inventory, Map, QuestLog, Dialog, HUD.

HUD: HP, стамина, патроны, миникарта, компас, квест-трекер.

**Статус:**
- Pause — [x] готов
- Loading — [x] готов
- Dialog — [~] базовый UI готов, ввод `1`/`2`, закрытие по `Esc`
- Остальное — [ ]

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

**Esc в диалоге ≠ пауза.**
`toggle_pause` реагирует только на `InGameState::Playing` и
`InGameState::Paused`. Ветка `InGameState::Dialog => {}` — пустая,
диалог сам ловит `Esc` и закрывается. Иначе первый же `Esc`
в диалоге уходил бы в паузу, а не в логику диалога.

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
   Raycast-vehicle — сложная настройка подвески Начать с
   визуального каркаса (§4.5), потом постепенно добавлять
   физику: прилипание → газ/тормоз → руль → трение

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

☑ Этап 5.1. Визуальный каркас
☑ actor/vehicle.rs: компоненты VehicleRoot, VehicleBodyModel,
WheelModel, Wheel { steerable, powered }
☑ assets.rs: хендлы car_scene, wheel1..4_scene
☑ modes/loading.rs: загрузка fast-car.glb и wheel1..4.glb
☑ modes/game/mod.rs: spawn_vehicle — кузов + 4 колеса
☑ Экспорт моделей из Blender по частям (Selected Objects, +Y Up)
☑ Координаты колёс из Blender-дампа, ориентация кузова 180° по Y

□ Этап 5.2. Прилипание к земле (подвеска)
□ controller/vehicle.rs: VehicleControllerPlugin
□ Raycast от каждого Wheel вниз
□ Сила пружины + демпфер к RigidBody в точке колеса
□ Настройка spring_strength, damper_strength, rest_length

□ Этап 5.3. Движение (газ/тормоз)
□ VehicleInput: педаль газа, тормоз
□ Сила вперёд к powered-колёсам
□ Настройка силы мотора

□ Этап 5.4. Поворот руля
□ VehicleInput: руль влево/вправо
□ Поворот Transform у steerable-колёс вокруг Y
□ Настройка угла поворота

□ Этап 5.5. Камера и «сел / вышел»
□ Режим камеры для машины
□ Переключение активной сущности (Player ↔ Vehicle)
□ Отключение TnuaController игрока, пока он в машине

Этап 6. NPC + диалоги + квесты
Цель: поговорить с NPC, взять квест, выполнить.

☑ Этап 6.1. Диалоговая система (прототип)
☑ Модель данных `DialogTree` / `DialogNode` / `DialogOption`
☑ Стейт `InGameState::Dialog`
☑ UI: текст + список опций, `1`/`2`, закрытие по `Esc`
☑ Задел на много опций (легко расширить до `Digit1..Digit9`)
☑ Esc в диалоге не пересекается с паузой
□ Загрузка `.ron` через `bevy_common_assets`
□ Спавн NPC-сущности с `Handle<DialogTree>`
□ Триггер диалога: расстояние + кнопка `E`
□ Ветвление по флагам (`condition`, `on_enter`)
□ Разделение реплик NPC / игрока
□ Влияние выборов на мир (флаги, отношения)

□ actor/npc.rs — сущность NPC с моделью и параметрами
□ quest/data.rs, quest/tracker.rs
□ ui/dialog.rs (расширение текущего)
□ ui/quest_log.rs

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
   controller/vehicle.rs — VehicleControllerPlugin (Этап 5.2).
   core/physics.rs — PhysicsPlugins + Tnua + ассет конфига.
   modes/loading.rs — ручной счёт через AssetServer + UI.
   modes/pause.rs — Time<Virtual>::pause.
   modes/game/mod.rs — IslandReady, finish_loading, spawn_vehicle.
   actor/vehicle.rs — компоненты транспорта.
   assets.rs — хендлы кузова и колёс.
   actor/*, controller/bot.rs — пустые, наполнять по этапам.
   modes/game/dialog/ — [~] прототип готов (см. §4.6).
   Осталось: загрузка .ron, NPC-триггер, ветвление по флагам.
   modes/game/{fight,hud,move_}.rs — пустые, наполнять по этапам.

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

**Vehicle: визуальный каркас отдельно от физики.**
Кузов — `RigidBody::Dynamic` с `Collider::cuboid`, но **без**
подвески и управления. Модели (кузов и колёса) — дети
физического корня. Это разделение позволяет:
- Сначала увидеть, что модель стоит на земле правильно.
- Потом добавлять физику подвески отдельно.
- Потом — движение и руль.

**Vehicle: одно колесо = один `.glb`, а не сборка из
`fast-car-wheel.glb`.** Изначально думали грузить одно колесо
и зеркалить, но при наличии четырёх отдельных `wheelN.glb`
проще грузить каждое как отдельный ассет. Плюс — если колёса
визуально разные (диски), это сохраняется.

**Vehicle: пустой узел Wheel между физическим корнем и моделью
колеса.** `Wheel` — пустой `Transform`, держит позицию колеса
относительно корня. Модель (`WheelModel`) — его ребёнок.
Это нужно, потому что:
- Позиция колеса задаётся **снаружи** модели (в `spawn_vehicle`),
  а не внутри `.glb`. Так можно подгонять без переэкспорта.
- Позже `steerable`-колёса будут крутиться вокруг Y — это
  вращение родителя, а не модели.
- Raycast подвески будет пускаться из `Wheel`, а не из модели.

**Vehicle: координаты колёс из Blender-дампа, а не формулы.**
Раньше в `spawn_vehicle` были формулы `half_length * 0.7` и т.п.
После того как машина в Blender была собрана вручную, стало
проще взять точные координаты. Формулы убраны.

**Vehicle: модель кузова повёрнута на 180° вокруг Y.**
В Blender перёд машины смотрит в −Y, после `+Y Up` — в −Z.
Стандарт Bevy/Tnua — +Z. Поворачиваем модель кузова в
`spawn_vehicle`, а координаты колёс оставляем как есть —
они уже выстроены относительно корня, и перёд машины — тот,
где стоят передние колёса (−Z).

**Vehicle: экспорт из Blender по частям, обнуляя позиции колёс.**
В `.blend` машина собрана целиком (кузов + 4 колеса). При
экспорте `Selected Objects` каждого колеса отдельно позиция
сохраняется из Blender — иначе в Bevy было бы двойное смещение
(сначала позиция `Wheel`, потом позиция внутри `.glb`). Порядок:
`Alt+G` (обнулить) → Export → `Ctrl+Z` (вернуть). Также
`Ctrl+A → All Transforms` и `Set Origin → Origin to Geometry`
для правильного origin.

**Vehicle: glTF-экспорт с `+Y Up` обязателен.** Без него
машина приедет в Bevy на боку — Blender работает в Z-up,
Bevy в Y-up.

**Vehicle: коллайдер кузова `Collider::cuboid(1.00, 0.66, 2.44)`.**
Полугабариты из дампа: dims в Bevy = (2.01, 1.32, 4.89), значит
полу = (1.00, 0.66, 2.44). Коллайдер — приблизительный (cuboid),
не совпадает с визуалом точно. Для статики и raycast-подвески
этого достаточно. Позже можно заменить на convex hull, если
понадобится точность.

**Диалоги — дерево, а не граф.** Модель:
`DialogTree { nodes: HashMap<String, DialogNode> }`, где каждый
узел содержит текст и список опций, каждая опция ведёт к
`next_node`. Не разделяем узлы и рёбра, не делаем отдельные
«переходы». Плюс: сериализуется в RON почти без усилий,
интуитивно понятно, легко расширяется. Минус: если понадобятся
условия на переходах — придётся навесить `condition` на
`DialogOption`, что тоже несложно.

**Диалог — подстейт `InGameState`, а не флаг.**
`InGameState::Dialog` — отдельный подстейт, а не булевый
`is_dialog_open`. Причина: в будущем это позволит одним
`run_if(in_state(...))` отключать физику, Tnua-ввод, стрельбу,
движение камеры, не разбрасывая флаги по системам. Заодно
пауза (`modes/pause.rs`) явно игнорирует Esc в этом стейте.

**Esc в диалоге ≠ пауза.** `toggle_pause` обрабатывает
`InGameState::Playing` и `Paused`, но не `Dialog`. Сам диалог
ловит `Esc` и закрывается. Иначе при первом же Esc диалог бы
пропадал через паузу, а не через свою логику, и это путало бы
поведение. Если позже понадобится открывать паузу поверх
диалога — сделать это явной веткой.

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
