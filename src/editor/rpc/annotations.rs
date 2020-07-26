
pub enum AnnotationType {
    Selection,
    Find,
    Added,
    Deleted,
    Modified,
}

pub struct Annotation {
    start_line: u64,
    start_col: u64,
    end_line: u64,
    end_col: u64,
    annoation_type: AnnotationType,
}


