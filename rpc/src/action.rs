use super::motion::Motion;
use super::mode::Mode;
use super::quantity::Quantity;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    FocusedView,
    StatusBar,
    EventLoop,
}

pub type MotionQuantity = (Motion, Option<Quantity>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Open(Option<String>),
    Split(Option<String>),
    Motion(MotionQuantity),
    Select(MotionQuantity),
    Delete(MotionQuantity),
    AddCursor(Motion),
    SetMode(Mode),
    SetTheme(String),
    SetLanguage(String),
    DefineCommand((String, Box<Action>)),
    InsertChar(char),
    Close,
    ExecuteCommand,
    ToggleLineNumbers,
    Indent,
    Outdent,
    NewLine,
    SearchNext,
    SearchPrev,
    SearchStart,
    SearchEnd,
    Save,
    Copy,
    Cut,
    Paste,
    IncreaseFontSize,
    DecreaseFontSize,
    ClearSelection,
    SingleSelection,
    Undo,
    Redo,
    UpperCase,
    LowerCase,
}
