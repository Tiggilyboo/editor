use std::collections::BTreeMap;
use render::Renderer;
use super::widget::Widget;

pub struct WidgetTree {
    widgets: BTreeMap<usize, Box<dyn Widget>>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            widgets: BTreeMap::new(),
        }
    }
    
    pub fn get(&self, widget_id: usize) -> Option<&Box<dyn Widget>> {
        self.widgets.get(&widget_id)
    }

    pub fn push(&mut self, widget: Box<dyn Widget>) {
        let id = self.widgets.len() + 1;
        self.widgets.insert(id, widget);
    }

    pub fn queue_draw(&self, renderer: &mut Renderer) {
        self.widgets.iter()
            .for_each(|(_, w)| w.queue_draw(renderer));
    }
}
