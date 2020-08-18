use serde::Deserialize;
use super::ViewId;

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
    pub view_id: ViewId,
    pub queries: Vec<Query>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Status {
    chars: String,
    preserve_case: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReplaceStatus {
    view_id: ViewId,
    status: Status, 
}
