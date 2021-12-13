use std::collections::BTreeMap;
use render::Renderer;
use super::widget::Widget;

pub struct WidgetTree<T> 
where T: Widget + Send + Sync {
    widgets: BTreeMap<usize, T>,
}

impl<T: Widget + Send + Sync> WidgetTree<T> {
    pub fn new() -> Self {
        Self {
            widgets: BTreeMap::new(),
        }
    }
    
    pub fn get(&self, widget_id: usize) -> Option<&T> {
        self.widgets.get(&widget_id)
    }
    pub fn get_mut(&mut self, widget_id: usize) -> Option<&mut T> {
        self.widgets.get_mut(&widget_id)
    }

    pub fn insert(&mut self, widget_id: usize, widget: T) {
        self.widgets.insert(widget_id, widget);
    }

    pub fn queue_draw(&self, renderer: &mut Renderer) {
        self.widgets.iter()
            .for_each(|(_, w)| w.queue_draw(renderer));
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.widgets.iter_mut()
            .for_each(|(_, w)| w.set_dirty(dirty));
    }

    pub fn dirty(&self) -> bool {
        self.widgets.iter().any(|(_, w)| w.dirty())
    }

    pub fn len(&self) -> usize {
        self.widgets.len()
    }
}
