use render::{
    Renderer,
    colour::ColourRGBA,
};
use crate::widget::{
    Widget,
    Size,
    Position,
};
use crate::text::TextWidget;
use crate::primitive::PrimitiveWidget;

pub struct GutterWidget {
    dirty: bool,
    position: Position,
    first_line: usize,
    height: usize,
    colour_foreground: ColourRGBA,
    colour_background: ColourRGBA,
    pad_left: f32,
    pad_right: f32,

    ln_number_text: TextWidget,
    ln_number_background: PrimitiveWidget,
}

impl Widget for GutterWidget {
    fn set_dirty(&mut self, dirty: bool) {
        self.ln_number_text.set_dirty(dirty);
        self.ln_number_background.set_dirty(dirty);
        self.dirty = dirty;
    }
    fn dirty(&self) -> bool {
        self.dirty
    }
    fn position(&self) -> Position {
        self.position
    }
    fn size(&self) -> Size {
        self.ln_number_background.size()
    }
    fn queue_draw(&self, renderer: &mut Renderer) {
        self.ln_number_text.queue_draw(renderer);
        self.ln_number_background.queue_draw(renderer);
    }
}

impl GutterWidget {
    pub fn new(position: Position, size: Size, font_scale: f32, colour_background: ColourRGBA, colour_foreground: ColourRGBA) -> Self {
        let ln_number_text = TextWidget::new().multiline();
        let ln_number_background = PrimitiveWidget::new(position, size, 0.3, colour_background);

        Self {
            ln_number_text,
            ln_number_background,
            colour_background,
            colour_foreground,
            position,
            dirty: true,
            first_line: 0,
            height: 0,
            pad_left: font_scale / 4.0,
            pad_right: font_scale / 3.0,
        }
    }

    pub fn update(&mut self, first_line: usize, height: usize, scale: f32, foreground: ColourRGBA) {
        if self.first_line == first_line 
        && self.height == height {
            return;
        }
        self.first_line = first_line;
        self.height = height;
        self.colour_foreground = foreground;
        self.set_padding(scale / 4.0, scale / 4.0);

        let mut lines = Vec::with_capacity(height);
        for ln in first_line .. first_line + height {
            let ln_str = (ln + 1).to_string();
            lines.push(ln_str + "\n");
        }

        self.ln_number_text.populate(lines, scale, foreground);
        self.set_dirty(true);
    }

    pub fn set_background(&mut self, background: ColourRGBA) {
        if self.colour_background == background {
            return;
        }
        self.colour_background = background;
        self.ln_number_background.set_colour(background);
        self.set_dirty(true);
    }

    pub fn set_width(&mut self, width: f32) {
        let h = self.ln_number_background.size().y;
        let pad = self.pad_left + self.pad_right;
        self.set_size(width + pad, h);
        self.set_dirty(true);
    }

    pub fn set_padding(&mut self, left: f32, right: f32) {
        self.pad_left = left;
        self.pad_right = right;
        self.set_position(self.position);
        self.set_dirty(true);
    }

    pub fn set_height(&mut self, height: f32) {
        let w = self.ln_number_background.size().x;
        self.set_size(w, height);
        self.set_dirty(true);
    }

    pub fn set_position(&mut self, position: Position) {
        let left = self.pad_left;
        self.ln_number_background.set_position(position.x, position.y);
        self.ln_number_text.set_position(left + position.x, position.y);
        self.position = position;
        self.set_dirty(true);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.ln_number_background.set_size(width, height);
        self.ln_number_text.set_linewrap_width(width);
    }
}
