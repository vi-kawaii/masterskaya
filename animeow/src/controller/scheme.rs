use bevy_tnua::builtins::{TnuaBuiltinJump, TnuaBuiltinWalk};
use bevy_tnua::prelude::*;

/// Схема управления персонажем.
/// Базис — ходьба. Действие — прыжок.
///
/// `#[derive(TnuaScheme)]` сам генерирует тип
/// `CharacterControlSchemeConfig`, реализующий `Asset`.
/// Он и есть конфиг схемы, отдельный тип писать не нужно.
#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum CharacterControlScheme {
    Jump(TnuaBuiltinJump),
}
