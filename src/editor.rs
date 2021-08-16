use crate::events::{
    binding::{
        KeyBinding,
        MouseBinding,
        default_key_bindings,
        default_mouse_bindings,
    },
    state::InputState,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    None,
    Normal,
    Insert,
    Replace,
    Visual,
    Command,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Motion {
    First,
    Last,
    Begin,
    End,
    Forward,
    Backward,
    Above,
    Below,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Quantity {
    Character,
    Line,
    Word,
    Page,
    Selection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SetMode(Mode),
    Delete(Motion, Quantity),
    Insert(Motion, Quantity),
    InsertChar,
    Join,
    Paste(Motion),
    Replace(Quantity),
    SelectBegin(Motion, Quantity),
    SelectEnd(Motion, Quantity),
    SelectAction(Box<Action>),
    Yank(Quantity),
    Undo(Quantity),
    Redo(Quantity),
    Move(Motion, Quantity),
    Repeat(Box<Action>, Quantity),
}

pub struct EditorState {
    key_bindings: Vec<KeyBinding>,
    mouse_bindings: Vec<MouseBinding>,
    mode: Mode,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            key_bindings: default_key_bindings(),
            mouse_bindings: default_mouse_bindings(),
            mode: Mode::None,
        }
    }

    pub fn acquire_input_actions(&self, state: &InputState) -> Vec<Action> {
        let mut triggered_actions: Vec<Action> = Vec::new();

        if let Some(pressed_key) = state.key {
            let mut key_triggers: Vec<Action> = self.key_bindings
                .iter()
                .filter(|b| b.is_triggered_by(self.mode, state.modifiers, &pressed_key))
                .flat_map(|b| b.actions.clone())
                .collect();

            triggered_actions.append(&mut key_triggers);
        }
        if let Some(mouse_button) = state.mouse.button {
            let mut mouse_triggers: Vec<Action> = self.mouse_bindings
                .iter()
                .filter(|b| b.is_triggered_by(self.mode, state.modifiers, &mouse_button))
                .flat_map(|b| b.actions.clone())
                .collect();

            triggered_actions.append(&mut mouse_triggers);
        }

        triggered_actions
    }
}
