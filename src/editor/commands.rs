use std::sync::{
    Mutex,
    Weak,
};
use serde_json::Value;
use rpc::{
    Action,
    Config,
    Theme,
    Style,
};
use crate::editor::editor_rpc::Core;

pub enum EditViewCommands {
    ViewId(String),
    ApplyUpdate(Value),
    ScrollTo(usize),
    Core(Weak<Mutex<Core>>),
    Resize([f32; 2]),
    ConfigChanged(Config),
    ThemeChanged(Theme),
    LanguageChanged(String),
    DefineStyle(Style),
    Action(Action) 
}
