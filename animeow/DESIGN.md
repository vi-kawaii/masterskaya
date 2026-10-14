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
   Загрузка ассетов bevy_asset_loader 0.27 Асинхронная подгрузка чанков и моделей
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
loading.rs
pause.rs
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
  Круче — персонаж скользит.
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
Направление движения берётся из ввода относительно камеры.

**Разделение физики и визуала.** Капсула (`RigidBody::Dynamic`)
несёт трансформ позиции и вращения. Модель — её ребёнок, имеет
локальный offset по Y и локальный yaw-offset (см. ниже). Физика
крутится Tnua, визуал просто следует за родителем.

**Спавн игрока — по событию готовности коллайдеров.**
Игрок не спавнится в `OnEnter(InGame)` напрямую. Вместо этого:
- `spawn_island` создаёт только свет и остров.
- Обсервер `on_island_ready` ждёт `ColliderConstructorHierarchyReady`
  и спавнит игрока в этот момент.
- Это устраняет «гонку загрузки»: игрок появляется только тогда,
  когда коллайдеры острова уже в мире, и не проваливается в пустоту.

`ColliderConstructorHierarchyReady` срабатывает **даже если
коллайдеры не создались**. Поэтому нужна проверка
`player_exists.is_empty()`.

**Схема управления: `CharacterControlScheme`.**
Объявлена в `controller/scheme.rs`. Базис — `TnuaBuiltinWalk`,
действие — `Jump(TnuaBuiltinJump)`. Макрос `#[derive(TnuaScheme)]`
генерирует:
- `CharacterControlSchemeConfig` — ассет конфига с полями
  `basis: TnuaBuiltinWalkConfig` и `jump: TnuaBuiltinJumpConfig`.
- Дискриминант и состояние действий.

Конфиг-хендл хранится в ресурсе `CharacterControlConfigHandle`
и вешается на игрока через `TnuaConfig::<CharacterControlScheme>(handle)`.
**Без конфига Tnua молча пропускает контроллер** — персонаж стоит,
но не двигается. Это неочевидная ловушка.

**Текущий конфиг (под капсулу 0.4×1.0):**
- `basis.float_height = 0.80` — ниже полвысоты капсулы (0.9),
  чтобы низ капсулы уходил в землю на ~0.1 и не было визуального
  зазора (floating controller всегда держит центр, а не низ).
- `basis.cling_distance = 0.05` — прилипание на спусках,
  без «запаса» пружины, который давал паразитный подъём.
- `basis.spring_strength = 100.0`, `spring_dampening = 1.0` —
  умеренно жёсткая пружина.
- `basis.speed = 6.0` — максимальная скорость в м/с.
- `basis.acceleration = 1000.0`, `air_acceleration = 100.0` —
  почти мгновенный старт/стоп, нет инерции.
- `jump.height = 1.2`, `jump.takeoff_extra_gravity = 15.0` —
  умеренный прыжок без «телепорта».

**Сенсор:** `TnuaAvian3dSensorShape(Collider::sphere(0.1))`.
Маленькая сфера даёт точное «прощупывание» земли. Раньше пробовали
капсулу радиуса 0.4 — сенсор «видел» землю раньше низа капсулы
и держал персонажа выше.

**Смещение модели внутри капсулы.** Модель — ребёнок капсулы,
локальный `Transform::from_xyz(0.0, -0.92, 0.0).with_rotation(yaw)`.
Y подбирается эмпирически, шаг 0.01–0.02, чтобы ноги визуально
стояли на земле. Правило: утопает → Y ближе к `-0.88`; парит →
Y ближе к `-0.95`.

**Yaw-компенсация модели.** `MODEL_YAW_OFFSET = PI` в
`spawn_player_entity`. Модель `anime-girl.glb` экспортирована
с forward в **-Z**, а Tnua использует **+Z**. Это временная
компенсация: при следующем импорте модели привести forward к +Z
в Blender (нос/лицо смотрят в **-Y** в Blender = +Z в Bevy) и
убрать константу. См. правило в §4.2 «Модель — самодостаточная».

**Ввод (InputState).** Абстракция ввода из `core/input.rs`:
`move_dir: Vec2` (локальные оси: x — вправо, y — вперёд),
`sprint: bool`, `jump_pressed: bool`. Заполняется в `Update`,
читается в `FixedUpdate` из `tnua_movement`.

**Порядок систем.** `tnua_movement` в `FixedUpdate` (Tnua и физика
в том же расписании). `flatten_materials` в `Update` (визуал).
`cleanup_player` на `OnExit(GameState::InGame)`.

4.3. Камера
CameraRig (родитель) + Camera3d (ребёнок).

Yaw/pitch от мыши, зум колесом, коллизия через raycast.

Сглаживание позиции и поворота.

Режимы: TPS-свободный, TPS-прицеливание, FPS-прицеливание.

**Сглаживание — только ввод, не слежение.**
- Риг жёстко привязан к игроку + `EYE_HEIGHT`. Без `lerp` по позиции —
  иначе укачивает.
- Мышь двигает *целевые* yaw/pitch (`target_yaw`, `target_pitch`),
  система `orbit_camera` плавно ведёт текущие значения к целям через
  `alpha = 1 - exp(-k·dt)`. Зум — так же через `target_distance`.
- Ощущение: «мышь тянет камеру с лёгкой инерцией», как в Genshin.

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
видеть коллизии, чем экономить FPS. Для релиза обернуть в
`#[cfg(debug_assertions)]` или фиче-флаг.

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

7. План работ
   Каждый этап заканчивается играбельным билдом.
   Не начинать следующий, пока предыдущий не запускается.

Этап 0. Фиксация решений
☑ Жанр, стек, архитектура зафиксированы в DESIGN.md

Этап 1. 3D-фундамент
Цель: пол, свет, камера, управляемый куб.

☑ Cargo.toml: добавить avian3d, bevy_asset_loader
☑ main.rs: Camera3d + DirectionalLight
☑ types.rs: расширить Player, GameSettings (move_speed, jump, sensitivity)
☑ controller/player.rs: переписать под 3D (спавн Cuboid, движение XZ)
☑ modes/game/mod.rs: убрать/переделать Triangle под 3D
☑ modes/vehicle.rs: временно отключить (вернём на Этапе 5)

Этап 2. TPS-камера + персонаж
Цель: ходить/бегать/прыгать, камера не застревает.

☑ core/camera.rs: CameraRig, орбита, pitch clamp, захват курсора,
  слежение за игроком
☑ core/camera.rs: зум колесом с clamp (min/max distance)
☑ core/camera.rs: плавность ввода — yaw/pitch/зум через
  `target_*` + экспоненциальное сглаживание
☑ core/input.rs: абстракция ввода (WASD, спринт, прыжок)
☑ actor/character.rs: общие компоненты (Health, MovementStats)
☑ core/physics.rs: PhysicsPlugins + Gravity + PhysicsDebugPlugin
☑ modes/loading.rs: реальная загрузка через bevy_asset_loader
☑ assets.rs: GameAssets (модель персонажа anime-girl.glb)
☑ controller/scheme.rs: CharacterControlScheme + CharacterControlSchemeConfig
☑ controller/player.rs: tnua_movement — заполняет TnuaController
   (basis: TnuaBuiltinWalk, action: Jump)
☑ core/physics.rs: TnuaControllerPlugin + TnuaAvian3dPlugin
☑ core/physics.rs: конфиг схемы как ассет + handle в ресурсе
☑ modes/game/mod.rs: spawn_player_entity с TnuaController,
  TnuaConfig, TnuaAvian3dSensorShape, LockedAxes
☑ modes/game/mod.rs: on_island_ready ждёт ColliderConstructorHierarchyReady
☑ core/debug.rs: отладочный текст с координатами

[ ] states.rs: locomotion-стейты (idle/run/jump/fall)
[ ] Анимации персонажа (AnimationPlayer + выбор клипа по состоянию)

Этап 3. Мир и стриминг
Цель: уровень, текстуры, тени, чанки.

□ Debug-полигон: холм, стенка, ступеньки, наклон — всё
  статическое, с коллайдерами. Цель — увидеть, где Tnua
  ломается на рельефе.
□ Ходьба по склонам: настройка TnuaBuiltinWalkConfig под геймплей
□ Snap to ground на спуске (cling_distance)
□ Автоподъём на ступеньки (Tnua сам умеет)
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
□ Анимации персонажей и NPC (экспорт моделей в .glb через Blender)
□ Звук, музыка
□ Эффекты, частицы
□ LOD, оптимизация
□ Баланс, контент

8. Порядок изменений в существующем коде
   states.rs — оставить, расширить подстейтами по §3.1.
   types.rs — расширить.
   controller/player.rs — построен вокруг TnuaController.
   controller/scheme.rs — CharacterControlScheme + конфиг.
   core/physics.rs — PhysicsPlugins + Tnua + ассет конфига.
   modes/vehicle.rs, modes/game/mod.rs — переписать под 3D-меши.
   modes/loading.rs — заменить таймер на реальную загрузку.
   actor/*, controller/bot.rs, modes/game/{dialog,fight,hud,move_}.rs — пустые, наполнять по этапам.

9. Журнал решений
   Короткие записи, почему сделано именно так. Не удалять — контекст.

Loading через bevy_asset_loader. Таймер убран. LoadingState сам
переключает GameState, когда ассеты загружены. Это соответствует
ARCHITECTURE.md §«Loading — это стейт, а не таймер».

**Переход с MoveAndSlide на bevy-tnua.** Изначально контроллер
был на `RigidBody::Kinematic + CustomPositionIntegration +
MoveAndSlide`. Это давало полный контроль над движением, но
упиралось в ghost collisions на тримешах: персонаж «парил» над
ровными участками. `FIX_INTERNAL_EDGES` помогал не полностью.
Решение — перейти на floating controller bevy-tnua. Это смена
парадигмы: Tnua держит **центр** капсулы на высоте над землёй,
а не двигает тело «до контакта». Ghost collisions исчезают
архитектурно. Минус — появляется визуальный зазор, но он
устраняется понижением `float_height` и подгонкой offset модели.

**Версия Tnua — 0.32 (для Bevy 0.19).** Версии 0.24 и раньше
собраны под Bevy 0.16, их нельзя использовать. Первые попытки
подключить 0.24 привели к конфликту двух версий Bevy в графе
зависимостей (bevy 0.19 + bevy 0.16 транзитивно) и к ошибкам
типа «`TnuaController` is not a `Component`». Правильные версии:
`bevy-tnua = "0.32"`, `bevy-tnua-avian3d = "0.12"`.

**Схема управления обязательна.** В Tnua 0.32 `TnuaController`
параметризован схемой (`TnuaScheme`). Схема — enum с вариантами
действий; базис задаётся в атрибуте `#[scheme(basis = ...)]`.
Макрос `#[derive(TnuaScheme)]` генерирует:
- `CharacterControlSchemeConfig` — ассет конфига, поля `basis`
  и `<действие>` (по имени варианта).
- Дискриминант действий и состояние.

`CharacterControlSchemeConfig` **не имеет `Default`** — его надо
создавать вручную, задавая поля.

**`TnuaConfig<S>(handle)` обязателен.** Без него
`apply_controller_system` делает `let Some(config) = ... else
{ continue; }` и молча пропускает контроллер. Симптом:
персонаж стоит и не двигается, ошибок в консоли нет. Отладка —
проверить наличие `TnuaConfig` на игроке и ассет в
`Assets<CharacterControlSchemeConfig>`.

**`controller.basis` — поле, не метод.** В 0.32 `basis` — это
поле типа `S::Basis` (не `Option`, не метод). Присваивается
напрямую: `controller.basis = TnuaBuiltinWalk { ... };`.

**`controller.action(...)` требует `initiate_action_feeding()`.**
Паника «Feeding action without invoking
`initiate_action_feeding()`» — прямой признак того, что
`action(...)` вызван без предварительного
`controller.initiate_action_feeding()`. Парного «end» метода нет:
флаг сбрасывается внутри `apply_controller_system` в конце кадра.
Вызывать `initiate_action_feeding()` нужно **каждый кадр**, когда
планируем вызвать `action(...)`.

**`action(...)` принимает вариант схемы.** Не «сырое» действие
(`TnuaBuiltinJump`), а вариант `CharacterControlScheme::Jump(...)`.
Аналогично `action_start`, `action_trigger`, `action_interrupt`,
`action_end` принимают соответствующие типы (действие или
дискриминант).

**Три-компонента для сенсора.** Tnua сама создаёт сенсоры
(через `TnuaSensorsEntities<S>`), но **форма** сенсора задаётся
пользователем через `TnuaAvian3dSensorShape`. Без этого
компонента Tnua использует ray cast от центра капсулы — это
работает, но грубее. Маленькая сфера (`Collider::sphere(0.1)`)
даёт более точное «прощупывание» земли, чем капсула радиуса 0.4:
последняя «видит» землю раньше низа капсулы и поднимает персонажа.

**Floating controller всегда держит центр, а не низ.** Это
архитектурная особенность, не баг. Нет способа заставить Tnua
«касаться земли низом капсулы». Приходится идти на компромисс:
- `float_height` ниже полвысоты капсулы (0.80 против 0.90) —
  низ уходит в землю, зазор исчезает.
- `cling_distance` уменьшен (0.05) — убирает паразитный подъём
  пружины.
- Модель внутри капсулы смещена по Y эмпирически
  (`-0.92` под капсулу 0.4×1.0 и `float_height = 0.80`).

Правило подбора: утопает → Y ближе к `-0.88`; парит → Y ближе
к `-0.95`. Шаг 0.01–0.02.

**`LockedAxes` через билдер в avian3d 0.7.** Константы
`ROTATION_LOCKED_X`, `ROTATION_LOCKED_Z` удалены. Использовать
`LockedAxes::new().lock_rotation_x().lock_rotation_z()`.

**`Grounded` удалён.** Tnua сам отслеживает состояние
«на земле / в воздухе» через `TnuaController::is_airborne()`.
Отдельный компонент не нужен. Для геймплея — читать из Tnua.

**Разворот модели — через Tnua `desired_forward`.** Не через
`rotate_model_towards_movement` (система удалена). Tnua крутит
тело, модель-ребёнок наследует rotation. `desired_forward:
Dir3::new(move_dir).ok()` — `None`, когда ввода нет (персонаж
не крутится).

**Yaw-компенсация модели — временный костыль.**
`MODEL_YAW_OFFSET = PI` в `spawn_player_entity`. Модель
`anime-girl.glb` экспортирована с forward в **-Z**, а Tnua
использует **+Z**. Модель крутится «наоборот». Правильное решение —
пересобрать модель в Blender с forward в +Z. Сейчас — компенсация
в коде, с явным именем константы и комментарием. См. §4.2
«Модель — самодостаточная».

**Правила приведения модели (DESIGN.md §4.2).** Обновление:
модель должна быть:
- Metric, Meters, Scale 1.0.
- Высота 1.8 м по Z (под капсулу `Collider::capsule(0.4, 1.0)`).
- Origin между ступней, по центру X/Y.
- Forward в **-Y** в Blender = **+Z** в Bevy.
- Rotation/scale/location применены (`Ctrl+A → All Transforms`).
- Empty между арматурой и мешами убраны, трансформы схлопнуты.

При следующем импорте модели — следовать этим правилам,
`MODEL_YAW_OFFSET` убрать.

**Камера — плавность во вводе, не в слежении.** Сглаживание
слежения (`lerp` позиции рига к игроку) даёт эффект «укачивания» и
смотрится «по-инди», не к месту. Плавность нужна в ощущении мыши:
мышь двигает `target_yaw`/`target_pitch`, система `orbit_camera`
ведёт текущие значения к целям через `alpha = 1 - exp(-k·dt)`. Риг
при этом **жёстко** привязан к игроку (`follow_player` без lerp).
Так камера «тянется» за мышью, как в Genshin, но не «отстаёт» от
игрока. Зум — через тот же сглаживатель (`target_distance`).
`camera_smoothing = 25.0` (1/с), подстраивается по ощущению.

**Разделение «текущее / цель» в CameraRig.** `yaw/pitch/distance` —
что видно на экране сейчас; `target_yaw/target_pitch/target_distance`
— куда мышь «тянет». Это позволяет сглаживать ввод, не сглаживая
позицию рига. Смешивать нельзя — иначе либо укачивает, либо мышь
«прилипает».

**Отладочный визуал коллайдеров — через PhysicsDebugPlugin.**
avian3d умеет рисовать коллайдеры линиями. Это даёт мгновенную
визуальную проверку размеров капсулы, пола и любых препятствий.
На ранних этапах важнее видеть коллизии, чем экономить FPS. Для
релиза — обернуть в `#[cfg(debug_assertions)]` или фиче-флаг.

**Остров — `main.glb` с trimesh-коллайдером из иерархии.**
Остров грузится через `GameAssets.island_scene`. Коллайдер строится
`ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)`
— avian сам обходит все меши внутри сцены и создаёт для каждого
trimesh-коллайдер. Порядок компонентов важен: `RigidBody::Static` и
`ColliderConstructorHierarchy` идут ДО `WorldAssetRoot`, иначе avian
не увидит сцену в момент её появления.

**Спавн игрока — по событию `ColliderConstructorHierarchyReady`.**
Раньше игрок спавнился в `OnEnter(InGame)` параллельно с островом,
и получалась «гонка загрузки»: игрок появлялся раньше, чем
коллайдеры острова, и проваливался в пустоту. Решение — обсервер
на `ColliderConstructorHierarchyReady`, который срабатывает, когда
avian закончил строить коллайдеры. В этот момент и спавнится игрок.
Это устраняет «гонку» без таймеров и «заморозок», через событийную
модель ECS.

`ColliderConstructorHierarchyReady` срабатывает **даже если
коллайдеры не создались**. Поэтому в обсервере есть проверка
`player_exists.is_empty()` — защита от повторного спавна.

**Bevy 0.17+: `Trigger` → `On`.** В обсервере параметр события
называется `On<Event>` (не `Trigger<Event>`). Доступ к цели события
— через поле `trigger.entity`, а не через метод. Это ломающее
изменение API между 0.16 и 0.17.

**Ghost collisions на trimesh — решены архитектурно.** См. выше
§«Переход с MoveAndSlide на bevy-tnua».

**Minecraft — это воксели, а не тримеш.** Для понимания: в Minecraft
физика работает на уровне отдельных блоков (кубов), а не на уровне
поверхности. Движок проверяет AABB игрока только с теми блоками,
которые он пересекает. Никаких «внутренних рёбер», никаких ghost
collisions. Пещеры — это блоки воздуха внутри массива блоков.
Тримеш так не может, потому что у него нет «адресов» — только
вершины и треугольники.

**Heightmap vs trimesh — разные инструменты.** Heightmap — это
регулярная 2D-сетка высот (`y = f(x, z)`). Не может иметь нависаний
и «низа». Trimesh — произвольная 3D-геометрия, может быть любой
формы. Heightmap не даёт ghost collisions, но и не может быть
островом с пещерами и нависающими скалами. Trimesh может всё, но
даёт ghost collisions на краях. В играх типа «Ведьмака» земля —
heightmap, а объекты (скалы, здания) — тримеши.

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
