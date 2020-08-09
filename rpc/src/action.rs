use super::motion::Motion;
use super::mode::Mode;
use super::quantity::Quantity;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    FocusedView,
    StatusBar,
    EventLoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Open(Option<String>),
    Split(Option<String>),
    Motion((Motion, Option<Quantity>)),
    Select((Motion, Option<Quantity>)),
    Delete((Motion, Option<Quantity>)),
    SetMode(Mode),
    SetTheme(String),
    SetLanguage(String),
    DefineCommand((String, Box<Action>)),
    InsertChar(char),
    Close,
    DeleteChar,
    Back,
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
    AddCursor(Motion),
    Undo,
    Redo,
    UpperCase,
    LowerCase,

    None,
}
