use std::hash::{
    Hasher,
    Hash,
};

use super::{
    colour::ColourRGBA,
    widget::{
        Widget,
        hash_widget,
    },
    editable_text::EditableTextWidget,
    view::Resources,
};
use crate::rpc::{
    Action,
    Motion,
    Quantity,
    Query,
};
use crate::render::Renderer;

pub struct FindWidget {
    index: usize,
    text: EditableTextWidget,
    queries: Vec<Query>,
    regex: bool,
    whole_word: bool,
}

impl Widget for FindWidget {
    fn index(&self) -> usize {
        self.index
    }
    fn size(&self) -> [f32; 2] {
        self.text.size()
    }
    fn position(&self) -> [f32; 2] {
        self.text.position()
    }
    fn dirty(&self) -> bool {
        self.text.dirty()
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        self.text.queue_draw(renderer);
    }
}

impl Hash for FindWidget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_widget(self, state);
        self.text.hash(state);
        self.regex.hash(state);
        self.whole_word.hash(state);
    }
}

impl FindWidget {
    pub fn new(index: usize, resources: &Resources) -> Self {
        let text = EditableTextWidget::new(index, resources);
        Self {
            index,
            text,
            whole_word: false,
            regex: false,
            queries: vec!(),
        }
    }

    pub fn set_queries(&mut self, queries: Vec<Query>) {
        self.queries = queries;
    }
}
