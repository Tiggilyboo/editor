use super::motion::Motion;
use super::mode::Mode;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    FocusedView,
    StatusBar,
    EventLoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Motion(Motion),
    MotionSelect(Motion),
    MotionDelete(Motion),
    SetMode(Mode),
    SetTheme(String),
    InsertChar(char),
    DefineCommand((String, Box<Action>)),
    ExecuteCommand,
    ToggleLineNumbers,
    Back,
    Delete,
    Indent,
    Outdent,
    NewLine,
    SearchNext,
    SearchPrev,
    SearchStart,
    SearchEnd,
    Open(Option<String>),
    Split(Option<String>),
    Close,
    Save,
    Copy,
    Cut,
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
    ClearSelection,
    SingleSelection,
    Undo,
    Redo,
    UpperCase,
    LowerCase,
    AddCursorAbove,
    AddCursorBelow,
    SelectAll,

    None,
}
