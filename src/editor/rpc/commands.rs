use std::sync::{
    Mutex,
    Weak,
};
use serde_json::Value;
use crate::editor::rpc::Core;
use super::config::Config;
use super::theme::Theme;
use crate::events::binding::Action;

pub enum EditViewCommands {
    ViewId(String),
    ApplyUpdate(Value),
    ScrollTo(usize),
    Core(Weak<Mutex<Core>>),
    Resize([f32; 2]),
    ConfigChanged(Config),
    ThemeChanged(Theme),
    Action(Action) 
}
