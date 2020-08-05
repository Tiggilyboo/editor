use std::sync::{
    Mutex,
    Weak,
};
use serde_json::Value;
use crate::editor::Action;
use crate::editor::rpc::Core;
use super::config::Config;
use super::theme::Theme;
use super::theme::Style;

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
