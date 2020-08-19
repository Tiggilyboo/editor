use rpc::PluginId;
use xi_core_lib::plugins::Command;


#[derive(Clone, Debug)]
pub struct PluginState {
    pub name: PluginId,
    pub active: bool,
    pub commands: Vec<Command>,
}
