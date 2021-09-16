use std::ops::Range;

use super::{
    Mode,
    Motion,
    Quantity,
};
use super::view::Size;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NewView { path: Option<String> },
    Resize(Size),
    RequestLines(usize, usize),
    RequestHover {
        request_id: usize, 
        position: Option<Position>,
    },
    Reindent,
    SetMode(Mode),
    Delete(Motion, Quantity),
    InsertChars(String),
    InsertNewline,
    InsertTab,
    SelectAll,
    Undo, 
    Redo,
    Yank,
    Indent,
    Outdent,
    DuplicateLine,
    IncreaseNumber,
    DecreaseNumber,
    Uppercase,
    Lowercase,
    Duplicate(Quantity),
    GoToLine(u64),
    Paste(String),
    Replace(Quantity),
    Move(Motion, Quantity),
    MoveSelection(Motion, Quantity),
    AddSelection(Motion),
    CollapseSelections,
    Gesture {
        line: u64,
        col: u64,
        ty: GestureType,
    },
    Scroll(Range<i64>),
    Repeat(Box<Action>, Quantity),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GestureType {
    Select { quantity: Quantity, multi: bool },
    SelectExtend { quantity: Quantity },
    Drag,

    // Deprecated
    PointSelect,
    ToggleSel,
    RangeSelect,
    LineSelect,
    WordSelect,
    MultiLineSelect,
    MultiWordSelect,
}

