use std::sync::{
    Mutex,
    Weak,
};
use std::collections::HashMap;
use serde_json::Value;

use super::plugins::{
    PluginState,
};
use crate::events::EditorEventLoopProxy;
use rpc::{
    ViewId,
    PluginId,
    Action,
    Config,
    Theme,
    Style,
    Query,
};
use crate::editor::editor_rpc::Core;

pub enum EditViewCommands {
    ViewId(ViewId),
    ApplyUpdate(Value),
    ScrollTo(usize),
    Core(Weak<Mutex<Core>>),
    Proxy(EditorEventLoopProxy),
    Resize([f32; 2]),
    Position([f32; 2]),
    ConfigChanged(Config),
    ThemeChanged(Theme),
    LanguageChanged(String),
    SetStyles(HashMap<usize, Style>),
    SetPlugins(HashMap<PluginId, PluginState>),
    PluginChanged(PluginState),
    PluginStopped(PluginId),
    Queries(Vec<Query>),
    Action(Action),
}

