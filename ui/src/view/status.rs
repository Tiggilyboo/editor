use std::collections::BTreeMap;
use render::{
    Renderer,
    colour::ColourRGBA,
};
use crate::widget::{
    Widget,
    Drawable,
    Size,
    Position,
};
use crate::text::TextWidget;

use crate::primitive::PrimitiveWidget;

pub struct StatusItemWidget<T>
where T: Widget + Send + Sync{
    align: StatusItemAlign,
    pub widget: T,
}

impl <T> StatusItemWidget<T>
where T: Widget + Send + Sync {
    pub fn set_align(&mut self, parent: &T, align: StatusItemAlign) {
        let parent_pos = parent.position();
        let parent_size = parent.size();

        self.align = align;
        self.realign(parent_pos, parent_size);
    }

    pub fn realign(&mut self, parent_pos: Position, parent_size: Size) {

        let (x, y) = match self.align {
            StatusItemAlign::Left => (parent_pos.x, parent_pos.y),
            StatusItemAlign::Right => (parent_pos.x + parent_size.x, parent_pos.y),
            StatusItemAlign::Center => (parent_pos.x + parent_size.x * 0.5, parent_pos.y),
        };

        self.widget.set_position(x, y);
    }
}

type WidgetTree<T> = BTreeMap<String, StatusItemWidget<T>>;

impl <W> Drawable for WidgetTree<W>
where W: Widget + Drawable + Send + Sync {
    fn set_dirty(&mut self, dirty: bool) {
        self.iter_mut().for_each(|(_, i)| i.widget.set_dirty(dirty));
    }
    fn dirty(&self) -> bool {
        self.iter().any(|(_, i)| i.widget.dirty())
    }
    fn queue_draw(&self, renderer: &mut Renderer) {
        self.iter().for_each(|(_, i)| i.widget.queue_draw(renderer));
    }
}

pub enum StatusItemAlign {
    Left,
    Center,
    Right,
}

pub struct StatusWidget {
    dirty: bool,
    colour_background: ColourRGBA,
    background: PrimitiveWidget,
    items: WidgetTree<TextWidget>,
}

impl Drawable for StatusWidget {
   fn dirty(&self) -> bool {
       self.dirty
   }

   fn set_dirty(&mut self, dirty: bool) {
       self.background.set_dirty(dirty);
       self.items.set_dirty(dirty);
       self.dirty = dirty;
   }
   
   fn queue_draw(&self, renderer: &mut Renderer) {
       self.background.queue_draw(renderer);
       self.items.queue_draw(renderer);
   }
}
impl Widget for StatusWidget {
   fn size(&self) -> Size {
       self.background.size()
   }
   fn position(&self) -> Position {
       self.background.position()
   }
   fn set_position(&mut self, x: f32, y: f32) {
       self.background.set_position(x, y);
       self.realign();
   }
}

impl StatusWidget {
    pub fn new(position: Position, size: Size, depth: f32, colour_background: ColourRGBA) -> Self {
        let background = PrimitiveWidget::new(position, size, depth, colour_background);
        let items = WidgetTree::new();

        Self {
            colour_background,
            background,
            items,
            dirty: true,
        }
    }

    pub fn set_background(&mut self, background: ColourRGBA) {
        if self.colour_background == background {
           return; 
        }
        self.colour_background = background;
        self.background.set_colour(background);
        self.set_dirty(true);
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.background.set_size(width, height);
        self.realign();
        self.set_dirty(true);
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.items.iter_mut()
            .for_each(|(_, i)| i.widget.set_scale(scale));
    }

    pub fn add_text(&mut self, item_key: String) -> &mut StatusItemWidget<TextWidget> {
        let item = StatusItemWidget {
            align: StatusItemAlign::Left,
            widget: TextWidget::new(),
        };

        self.items.insert(item_key.clone(), item);
        self.set_dirty(true);

        self.get(item_key).unwrap()
    }

    pub fn get(&mut self, item_key: String) -> Option<&mut StatusItemWidget<TextWidget>> {
        self.items.get_mut(&item_key)
    }

    fn realign(&mut self) {
        let position = self.position();
        let size = self.size();

        for (_, i) in self.items.iter_mut() {
            i.realign(position, size);
        }
    }
}
