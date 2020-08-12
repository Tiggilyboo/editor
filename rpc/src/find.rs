use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Query {
    pub id: usize,
    pub chars: String,
    pub case_sensitive: bool,
    pub is_regex: bool,
    pub whole_words: bool,
    pub matches: usize,
    pub lines: Vec<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FindStatus {
    pub view_id: String,
    pub queries: Vec<Query>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Status {
    chars: String,
    preserve_case: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReplaceStatus {
    view_id: String,
    status: Status, 
}
