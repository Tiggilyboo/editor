use super::motion::Motion;
use super::mode::Mode;
use super::quantity::Quantity;
use super::plugins::PluginId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    FocusedView,
    StatusBar,
    EventLoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginAction {
    Start(PluginId),
    Stop(PluginId),
}

pub type MotionQuantity = (Motion, Option<Quantity>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Open(Option<String>),
    Save(Option<String>),
    Split(Option<String>),
    Motion(MotionQuantity),
    Select(MotionQuantity),
    Delete(MotionQuantity),
    AddCursor(Motion),
    InsertChar(char),
    SetMode(Mode),
    SetTheme(String),
    SetLanguage(String),
    Plugin(PluginAction),
    DefineCommand((String, Box<Action>)),
    Close,
    Execute,
    ToggleLineNumbers,
    Indent,
    Outdent,
    InsertTab,
    NewLine,
    DuplicateLine,
    SearchNext,
    SearchPrev,
    SearchStart,
    SearchEnd,
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
