use std::collections::BTreeMap;
use render::Renderer;
use super::widget::Widget;

pub struct WidgetTree {
    widgets: BTreeMap<usize, Box<dyn Widget + Send + Sync>>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            widgets: BTreeMap::new(),
        }
    }
    
    pub fn get(&self, widget_id: usize) -> Option<&Box<dyn Widget + Send + Sync>> {
        self.widgets.get(&widget_id)
    }

    pub fn insert(&mut self, widget_id: usize, widget: Box<dyn Widget + Send + Sync>) {
        self.widgets.insert(widget_id, widget);
    }

    pub fn queue_draw(&mut self, renderer: &mut Renderer) {
        self.widgets.iter_mut()
            .for_each(|(_, w)| {
                w.queue_draw(renderer);
                w.set_dirty(false);
            });
    }

    pub fn len(&self) -> usize {
        self.widgets.len()
    }
}
