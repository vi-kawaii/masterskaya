use bevy::prelude::*;

/// Корень машины — физическое тело (RigidBody::Dynamic + Collider).
/// Визуал кузова и колёса — его дети.
#[derive(Component)]
pub struct VehicleRoot;

/// Маркер узла с моделью кузова (ребёнок VehicleRoot).
#[derive(Component)]
pub struct VehicleBodyModel;

/// Маркер узла с моделью колеса (внук VehicleRoot, ребёнок Wheel).
#[derive(Component)]
pub struct WheelModel;

/// Пустой узел колеса. Держит Transform (позиция и поворот руля)
/// и служит точкой для raycast'а подвески.
#[derive(Component)]
pub struct Wheel {
    /// Переднее колесо — поворачивается рулём.
    pub steerable: bool,
    /// Ведущее колесо — получает силу мотора.
    pub powered: bool,
}
