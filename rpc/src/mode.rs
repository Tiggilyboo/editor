use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Replace,
    Select,
    LineSelect,
    BlockSelect,
    Command,

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
            Mode::BlockSelect => write!(f, "V-BLOCK"),
            Mode::LineSelect => write!(f, "V-LINE"),
            _ => write!(f, "{:?}", self),
        }
    }
}
