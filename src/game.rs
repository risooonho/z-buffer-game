//! Application data that brings together display, input & game state.

use crate::input::Input;
use crate::stage::{Stage, StageTransition};

use crate::ui::UI;
use std::time::Duration;
use tcod::system::get_elapsed_time;

#[derive(Default)]
pub struct Game {
    stage: Stage,
    ui: UI,
    dt: u32,
    input: Input,
    time: u64,
}

impl Game {
    pub fn new() -> Game {
        Game {
            ui: UI::new(),
            stage: Stage::new(),
            dt: 0,
            time: 0,
            input: Input::new(),
        }
    }

    pub fn dt(&self) -> u32 {
        self.dt
    }

    /// Application main loop, blocks until UI terminates.
    pub fn main_loop(&mut self) {
        while self.ui.is_running() {
            self.update_time();
            let events = self.input.events();
            match self.stage.tick(self.dt, events) {
                StageTransition::Continue => (),
                StageTransition::SwitchTo(new_stage) => self.stage = new_stage,
            }
            self.ui.draw(&self.stage);
        }
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    fn update_time(&mut self) {
        let old_time = self.time;
        self.time = duration_to_millis(&get_elapsed_time());
        self.dt = (self.time - old_time) as u32;
    }
}

fn duration_to_millis(t: &Duration) -> u64 {
    t.as_secs() * 1000 + u64::from(t.subsec_millis())
}
