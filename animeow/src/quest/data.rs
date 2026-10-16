use bevy::prelude::*;
use serde::Deserialize;

/// Что должно произойти, чтобы шаг квеста засчитался.
///
/// В RON пишется как вариант enum с payload:
///   target: TalkTo("npc_test_1"),
///   target: ReachLocation("village"),
///   target: KillCount("wolf", 3),
///   target: CollectItem("herb", 5),
///
/// ВАЖНО: варианты enum в RON — без кавычек. `TalkTo("...")`, не `"TalkTo(...)"`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum QuestTarget {
    TalkTo(String),
    ReachLocation(String),
    KillCount(String, u32),
    CollectItem(String, u32),
}

/// Один шаг квеста.
#[derive(Debug, Clone, Deserialize)]
pub struct QuestStep {
    pub id: String,
    pub description: String,
    pub target: QuestTarget,
}

/// Награда за завершение квеста.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct QuestReward {
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub items: Vec<String>,
}

/// Определение квеста — ассет из `.ron`.
/// Один файл — один квест. Расширение обязательно `.quest.ron`.
#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct QuestDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<QuestStep>,
    #[serde(default)]
    pub rewards: QuestReward,
}