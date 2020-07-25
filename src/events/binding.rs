use winit::event::{
    ModifiersState
};


pub struct Binding<T> {
    mods: ModifiersState,
    trigger: T,
}

pub type KeyBinding = Binding<Key>;
pub type MouseBinding = Binding<MouseButton>;

#[derive(Debug, Clone, Deserialize)]
pub enum Action {
    Motion(Motion),
    ToggleMode(Mode),
    SearchNext,
    SearchPrev,
    SearchStart,
    SearchEnd,
    Open,
    Copy,
    Paste,
    IncreaseFontSize,
    DecreaseFontSize,
    ScrollPageUp,
    ScrollPageDown,
    ScrollHalfPageUp,
    ScrollHalfPageDown,
    ScrollLineUp,
    ScrollLineDown,
    ScrollToTop,
    ScrollToBottom,
    Quit,
    ClearSelection,
    ReceiveChar,

    None,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Mode {
    Normal,
    Insert,
    LineSelect,
    BlockSelect,
    SemanticSelect,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,
    First,
    Last,
    FirstOccupied,
    High,
    Middle,
    Low,
    SemanticLeft,
    SemanticRight,
    SemanticRightEnd,
    WordLeft,
    WordRight,
    WordRightEnd,
    Bracket,
}
