use std::sync::{
    Mutex,
    Weak,
};
use serde_json::Value;

use super::plugins::{
    PluginState,
    PluginId,
};
use crate::events::EditorEventLoopProxy;
use rpc::{
    Action,
    Config,
    Theme,
    Style,
    Query,
};
use crate::editor::editor_rpc::Core;

pub enum EditViewCommands {
    ViewId(String),
    ApplyUpdate(Value),
    ScrollTo(usize),
    Core(Weak<Mutex<Core>>),
    Proxy(EditorEventLoopProxy),
    Resize([f32; 2]),
    Position([f32; 2]),
    ConfigChanged(Config),
    ThemeChanged(Theme),
    LanguageChanged(String),
    DefineStyle(Style),
    PluginStarted(PluginState),
    PluginStopped(PluginId),
    Queries(Vec<Query>),
    Action(Action),
}
