use std::sync::Mutex;

use flume::{
    Sender,
    Receiver,
};
use super::{
    ViewId,
    styles::Style,
    annotations::AnnotationSlice,
    line_cache::Line,
};

pub struct Client {
    tx: Sender<Message>,
    rx: Mutex<Receiver<Message>>,
}

#[derive(Debug)]
pub enum Command {
    Scroll {
        line: usize,
        col: usize,
    },
    Idle {
        token: usize,
    },
    ShowHover {
        req_id: usize,
        content: String,
    },
    DefineStyle {
        style_id: usize,
        style: Style,
    },
}

#[derive(Debug)]
pub enum Payload {
    BufferUpdate(Update),
    Command(Command),
}

#[derive(Debug)]
pub struct Message {
    pub view_id: Option<ViewId>,
    pub payload: Payload,
}

impl Client {
    pub fn new() -> Self { 
        let (tx, rx) = flume::unbounded::<Message>();
        let rx = Mutex::new(rx);

        Self {
           tx,
           rx,
        } 
    }
    pub fn scroll_to(&self, view_id: ViewId, line: usize, col: usize) {
        let payload = Payload::Command(Command::Scroll { line, col });
        self.tx.send(Message { view_id: Some(view_id), payload }).unwrap();
    }
    pub fn update_view(&self, view_id: ViewId, update: &Update) {
        let payload = Payload::BufferUpdate(update.clone());
        self.tx.send(Message { view_id: Some(view_id), payload }).unwrap();
    }
    pub fn schedule_idle(&self, token: usize) {
        let payload = Payload::Command(Command::Idle { token });
        self.tx.send(Message { view_id: None, payload }).unwrap();
    }
    pub fn show_hover(&self, view_id: ViewId, req_id: usize, content: String) {
        let payload = Payload::Command(Command::ShowHover{ req_id, content });
        self.tx.send(Message { view_id: Some(view_id), payload }).unwrap();
    }
    pub fn define_style(&self, style_id: usize, style: Style) {
        let payload = Payload::Command(Command::DefineStyle { style_id, style });
        self.tx.send(Message { view_id: None, payload }).unwrap();
    }

    pub fn get_message_stream(&self) -> &Mutex<Receiver<Message>> {
        &self.rx
    }
}

#[derive(Debug, Clone)]
pub struct Update {
    pub ops: Vec<UpdateOp>,
    pub pristine: bool,
    pub annotations: Vec<AnnotationSlice>,
}


#[derive(Debug, Clone)]
pub struct UpdateOp {
    pub op: OpType,
    pub n: usize,
    pub lines: Option<Vec<Line>>,
    pub first_line_number: Option<usize>,
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

    pub(crate) fn insert(lines: Vec<Line>) -> Self {
        UpdateOp { op: OpType::Insert, n: lines.len(), lines: Some(lines), first_line_number: None }
    }

    pub(crate) fn update(lines: Vec<Line>, line_opt: Option<usize>) -> Self {
        UpdateOp {
            op: OpType::Update,
            n: lines.len(),
            lines: Some(lines),
            first_line_number: line_opt,
        }
    }
}


#[derive(Debug, Clone)]
pub enum OpType {
    Insert,
    Skip,
    Invalidate,
    Copy,
    Update,
}
