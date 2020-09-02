use serde::{
    Deserialize,
    Deserializer,
};
use serde_json::{Value};

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Selection,
    Find,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AnnotationRange {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Annotation {
    #[serde(alias = "type")]
    pub annotation_type: AnnotationType,
    pub ranges: Vec<AnnotationRange>,
    pub payloads: Option<Vec<Value>>,
}

impl<'de> Deserialize<'de> for AnnotationRange {
    fn deserialize<D>(deserializer: D) -> Result<AnnotationRange, D::Error> where D: Deserializer<'de> {
        let mut range = AnnotationRange { ..Default::default() };
        let seq = <[usize; 4]>::deserialize(deserializer)?;

        range.start_line = seq[0];
        range.start_col = seq[1];
        range.end_line = seq[2];
        range.end_col = seq[3];

        Ok(range)
    }
}

