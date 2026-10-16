use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::data::{QuestDef, QuestStep};

/// Один активный квест: определение + прогресс.
#[derive(Debug, Clone)]
pub struct ActiveQuest {
    pub def: QuestDef,
    /// Индекс текущего шага в `def.steps`.
    pub current_step: usize,
    /// Счётчики для KillCount/CollectItem: ключ — цель, значение — сколько набрано.
    /// Например, `{"wolf" -> 2}`.
    pub counters: HashMap<String, u32>,
}

impl ActiveQuest {
    pub fn new(def: QuestDef) -> Self {
        Self {
            def,
            current_step: 0,
            counters: HashMap::new(),
        }
    }

    /// Текущий шаг (если ещё есть).
    pub fn current(&self) -> Option<&QuestStep> {
        self.def.steps.get(self.current_step)
    }

    pub fn is_finished(&self) -> bool {
        self.current_step >= self.def.steps.len()
    }
}

/// Журнал квестов: что сейчас в работе, что завершено.
#[derive(Resource, Default)]
pub struct QuestLog {
    /// Активные квесты. Ключ — `QuestDef::id`.
    pub active: HashMap<String, ActiveQuest>,
    /// Завершённые квесты (только id).
    pub completed: HashSet<String>,
}

impl QuestLog {
    /// Запустить квест. Если уже активен или завершён — игнорируем.
    pub fn start(&mut self, def: QuestDef) -> bool {
        let id = def.id.clone();
        if self.active.contains_key(&id) || self.completed.contains(&id) {
            return false;
        }
        println!("[quest] start: {} ({})", def.name, id);
        self.active.insert(id, ActiveQuest::new(def));
        true
    }

    /// Первый активный квест — для отображения в HUD.
    /// HashMap неупорядочен, но для одного-двух квестов это неважно.
    /// Когда появится нормальный журнал — заменить на Vec или IndexMap.
    pub fn first_active(&self) -> Option<&ActiveQuest> {
        self.active.values().next()
    }
}