#![cfg_attr(not(feature = "std"), no_std)]

pub mod dice;
pub mod ui;
pub mod animation;
pub mod history;
pub mod sprites;

use dice::DieType;
use animation::AnimationState;
use history::RollHistory;

/// Main application state — shared between simulator and firmware
pub struct AppState {
    pub current_die: DieType,
    pub last_result: Option<u16>,
    pub animation: AnimationState,
    pub history: RollHistory,
    pub show_history: bool,
    pub die_types: [DieType; 7],
    pub die_index: usize,
}

impl AppState {
    pub fn new() -> Self {
        let die_types = [
            DieType::D4,
            DieType::D6,
            DieType::D8,
            DieType::D10,
            DieType::D12,
            DieType::D20,
            DieType::D100,
        ];
        Self {
            current_die: DieType::D20,
            last_result: None,
            animation: AnimationState::Idle,
            history: RollHistory::new(),
            show_history: false,
            die_types,
            die_index: 5, // D20
        }
    }

    pub fn cycle_die(&mut self) {
        self.die_index = (self.die_index + 1) % self.die_types.len();
        self.current_die = self.die_types[self.die_index];
    }

    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
