use super::motion::Motion;
use super::mode::Mode;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    FocusedView,
    StatusBar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Motion(Motion),
    MotionSelect(Motion),
    MotionDelete(Motion),
    SetMode(Mode),
    SetTheme(String),
    ToggleLineNumbers,
    InsertChar(char),
    DefineCommand((String, Box<Action>)),
    ExecuteCommand,
    Back,
    Delete,
    Indent,
    Outdent,
    NewLine,
    SearchNext,
    SearchPrev,
    SearchStart,
    SearchEnd,
    Open,
    Quit,
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
