use rpc::{
    Action,
    PluginId,
    PluginAction,
};
use super::ui::view::EditView;

// Translates text commands into Actions
pub fn command_to_actions(view: &EditView, command_text: String) -> Vec<Action> {
    let mut actions: Vec<Action> = vec!(); 
    let args: Vec<String> = command_text.split(" ").map(|a| a.to_string()).collect();

    let filename = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        view.get_filepath().clone()
    };

    // TODO: make this not crap
    match args[0].as_str() {
        "e" => actions.push(Action::Open(filename)),
        "w" => actions.push(Action::Save(filename)),
        "q" => actions.push(Action::Close),
        "wq" => actions.extend(vec![Action::Save(filename), Action::Close]),
        "sp" => actions.push(Action::Split(filename)),
        "plug" => {
            if args.len() < 3 {
                println!("usage: plug [start|stop] <plugin_name>");
            } else {
                let plugin_id = PluginId::from(args[2].clone());
                match args[1].as_str() {
                    "start" => actions.push(Action::Plugin(PluginAction::Start(plugin_id))),
                    "stop" => actions.push(Action::Plugin(PluginAction::Stop(plugin_id))),
                    _ => println!("args: {:?}", args),
                }
            }
        },
        _ => {},
    }

    if actions.len() == 0 {
        println!("No command found: '{}'", command_text.clone());
    }

    actions
}
