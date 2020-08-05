use serde::{
    Serialize,
    Deserialize,
};
use serde_json::{
    Value,
    Map,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    pub font_face: Option<String>,
    pub font_size: Option<f32>,
    pub line_ending: Option<String>,
    pub plugin_search_path: Option<Vec<String>>,
    pub tab_size: Option<u64>,
    pub translate_tabs_to_spaces: Option<bool>,
    pub word_wrap: Option<bool>,
}

impl Config {
    pub fn get_json_changes(&self, config: Config) -> Value {
        let old_json = serde_json::to_value(self.clone()).unwrap();
        let config_json = serde_json::to_value(config.clone()).unwrap();
        
        match config_json {
            Value::Object(map) => {
                let mut changes: Map<String, Value> = Map::with_capacity(map.len());
                for (k,v) in map.iter() {
                    if let Some(old_value) = old_json.get(k) {
                        if *old_value != Value::Null 
                            && *v != Value::Null
                            && *old_value != *v {
                            changes.insert(k.clone(), v.clone());
                        }
                    }
                }

                Value::from(changes)
            },
            _ => unreachable!(),
        }
    }
}
