use std::collections::BTreeMap;
use std::collections::VecDeque;
use tcod::input::{self as tcod_input, Event as TcodEvent, KeyCode};

const DEFAULT_EVENT_QUEUE_SIZE: usize = 20;

#[derive(Debug, Clone)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
    KeyPress(KeyCode, Option<char>, Modifiers),
    KeyDown(KeyCode, Option<char>, Modifiers),
    KeyUp(KeyCode, Option<char>, Modifiers),
    Mouse,
}

pub struct Input {
    // TODO: Key type should be a KeyCode, but since it
    //       does not satisfy the constrains we will be
    //       casting to isize.
    key_states: BTreeMap<isize, bool>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            key_states: BTreeMap::new(),
        }
    }

    pub fn events(&mut self) -> VecDeque<Event> {
        let mut events = VecDeque::with_capacity(DEFAULT_EVENT_QUEUE_SIZE);
        for (_, e) in tcod_input::events() {
            match e {
                TcodEvent::Key(k) => {
                    let modifiers = Modifiers {
                        shift: k.shift,
                        alt: k.alt,
                        ctrl: k.ctrl,
                    };
                    let character: Option<char> = if k.code == KeyCode::Char {
                        Some(k.printable)
                    } else {
                        None
                    };
                    let e = if k.pressed {
                        Event::KeyDown(k.code, character, modifiers)
                    } else {
                        Event::KeyUp(k.code, character, modifiers)
                    };
                    let e2_option = self.detect_keypress(&e);
                    events.push_back(e);
                    e2_option.map(|e| {
                        events.push_back(e);
                    });
                }
                TcodEvent::Mouse(_) => events.push_back(Event::Mouse),
            }
        }
        events
    }

    fn detect_keypress(&mut self, e: &Event) -> Option<Event> {
        let key_code: &KeyCode = match e {
            Event::KeyDown(key_code, ..) => key_code,
            Event::KeyUp(key_code, ..) => key_code,
            _ => panic!(),
        };
        let key_state: bool = self
            .key_states
            .get(&(*key_code as isize))
            .unwrap_or(&false)
            .clone();
        match (e, key_state) {
            (Event::KeyDown(_, character, modifiers), true) => {
                Some(Event::KeyPress(*key_code, *character, (*modifiers).clone()))
            }
            (Event::KeyDown(..), false) => {
                self.key_states.insert(*key_code as isize, true);
                None
            }
            (Event::KeyUp(_, character, modifiers), _) => {
                self.key_states.remove(&(*key_code as isize));
                Some(Event::KeyPress(*key_code, *character, (*modifiers).clone()))
            }
            _ => None,
        }
    }
}
