use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Replace,
    Command,
    Select,
    SelectLine,
    SelectBlock,
    Window,
    Motion,
    None,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Replace => write!(f, "REPLACE"),
            Mode::Command => write!(f, "COMMAND"),
            Mode::Select => write!(f, "VISUAL"),
            Mode::SelectBlock => write!(f, "V-BLOCK"),
            Mode::SelectLine => write!(f, "V-LINE"),
            Mode::Window => write!(f, "WINDOW"),
            Mode::Motion => write!(f, "MOTION"),
            _ => write!(f, "{:?}", self),
        }
    }
}
