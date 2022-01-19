mod actions;
mod backspace;
mod editor;
mod edit_ops;
mod errors;
mod event_context;
mod file;
mod index_set;
mod layers;
mod line_cache_shadow;
mod line_offset;
mod linewrap;
mod selection;
mod movement;
mod unicode;
mod unicode_tables;
mod words;
mod view;

pub mod annotations;
pub mod client;
pub mod width_cache;
pub mod styles;
pub mod line_cache;

pub use actions::Action;
pub use editor::*;
pub use view::*;
pub use editor::*;
pub use event_context::*;
pub use file::FileManager;
pub use linewrap::Lines;
pub use client::Client;
pub use rope::Rope;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    None,
    Normal,
    Insert,
    Delete,
    Replace,
    Visual,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    None,
    First,
    Last,
    Begin,
    End,
    Forward,
    Backward,
    Above,
    Below,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Quantity {
    Character,
    Word,
    Bracket,
    Line,
    Selection,
    Paragraph,
    Page,
    Document,
}

pub const STATUS_ITEM_FILEPATH: &str = "status_filepath";
pub const STATUS_ITEM_MODE: &str = "status_mode";
pub const STATUS_ITEM_LINEINFO: &str = "status_lineinfo";

impl Into<String> for Mode {
    fn into(self) -> String {
        format!("{:?}", self)
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
