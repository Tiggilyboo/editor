use xi_core_lib::plugins::Command;

pub type PluginId = String;

#[derive(Clone, Debug)]
pub struct PluginState {
    pub name: String,
    pub active: bool,
    pub commands: Vec<Command>,
}
