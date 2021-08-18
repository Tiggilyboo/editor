use super::ViewId;
use super::annotations::AnnotationSlice;

// TODO
pub struct Client{}

impl Client {
    pub fn new() -> Self { Self{} }
    pub fn scroll_to(&self, view_id: ViewId, line: usize, col: usize) {
        unimplemented!()
    }
    pub fn update_view(&self, view_id: ViewId, update: &Update) {
        unimplemented!()
    }
    pub fn schedule_idle(&self, token: usize) {
        unimplemented!()
    }
    pub fn show_hover(&self, view_id: ViewId, req_id: usize, content: String) {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Update {
    pub(crate) ops: Vec<UpdateOp>,
    pub(crate) pristine: bool,
    pub(crate) annotations: Vec<AnnotationSlice>,
}

#[derive(Debug)]
pub(crate) struct UpdateOp {
    op: OpType,
    n: usize,
    lines: Option<Vec<String>>,
    first_line_number: Option<usize>,
}

impl UpdateOp {
    pub(crate) fn invalidate(n: usize) -> Self {
        UpdateOp { op: OpType::Invalidate, n, lines: None, first_line_number: None }
    }

    pub(crate) fn skip(n: usize) -> Self {
        UpdateOp { op: OpType::Skip, n, lines: None, first_line_number: None }
    }

    pub(crate) fn copy(n: usize, line: usize) -> Self {
        UpdateOp { op: OpType::Copy, n, lines: None, first_line_number: Some(line) }
    }

    pub(crate) fn insert(lines: Vec<String>) -> Self {
        UpdateOp { op: OpType::Insert, n: lines.len(), lines: Some(lines), first_line_number: None }
    }

    pub(crate) fn update(lines: Vec<String>, line_opt: Option<usize>) -> Self {
        UpdateOp {
            op: OpType::Update,
            n: lines.len(),
            lines: Some(lines),
            first_line_number: line_opt,
        }
    }
}

#[derive(Debug, Clone)]
enum OpType {
    Insert,
    Skip,
    Invalidate,
    Copy,
    Update,
}
