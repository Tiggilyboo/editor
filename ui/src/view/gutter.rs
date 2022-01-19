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

pub struct GutterWidget {
    dirty: bool,
    position: Position,
    first_line: usize,
    height: usize,
    colour_foreground: ColourRGBA,
    colour_background: ColourRGBA,
    ln_number_text: TextWidget,
}

impl Drawable for GutterWidget {
    fn set_dirty(&mut self, dirty: bool) {
        self.ln_number_text.set_dirty(dirty);
        self.dirty = dirty;
    }
    fn dirty(&self) -> bool {
        self.dirty
    }
    fn queue_draw(&self, renderer: &mut Renderer) {
        self.ln_number_text.queue_draw(renderer);
    }
}
impl Widget for GutterWidget {
    fn position(&self) -> Position {
        self.position
    }
    fn size(&self) -> Size {
        self.ln_number_text.size()
    }
    fn set_position(&mut self, x: f32, y: f32) {
        self.ln_number_text.set_position(x, y);
        self.position = (x, y).into();
        self.set_dirty(true);
    }
}

impl GutterWidget {
    pub fn new(position: Position, depth: f32, font_scale: f32, colour_background: ColourRGBA, colour_foreground: ColourRGBA) -> Self {
        let mut ln_number_text = TextWidget::with_multiline();
        ln_number_text.set_background(colour_background, depth);
        ln_number_text.set_padding(font_scale / 4.0, font_scale / 3.0);

        Self {
            ln_number_text,
            colour_background,
            colour_foreground,
            position,
            dirty: true,
            first_line: 0,
            height: 0,
        }
    }

    pub fn update(&mut self, first_line: usize, last_line: usize, scale: f32, foreground: ColourRGBA) {
        if self.first_line == first_line 
        && self.height == last_line - first_line {
            return;
        }
        self.first_line = first_line;
        self.height = last_line - first_line;
        self.colour_foreground = foreground;
        self.set_padding(scale / 4.0, scale / 4.0);

        let mut lines = Vec::with_capacity(self.height);
        for ln in first_line .. last_line {
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
        self.ln_number_text.set_background(background, 0.3);
        self.set_dirty(true);
    }

    pub fn set_width(&mut self, width: f32) {
        let h = self.size().y;
        self.set_size(width, h);
        self.set_dirty(true);
    }

    pub fn set_padding(&mut self, left: f32, right: f32) {
        self.ln_number_text.set_padding(left, right);
        let pos = self.position();
        self.set_position(pos.x, pos.y);
        self.set_dirty(true);
    }

    pub fn set_height(&mut self, height: f32) {
        let w = self.size().x;
        self.set_size(w, height);
        self.set_dirty(true);
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.ln_number_text.set_size(width, height);
        self.ln_number_text.set_linewrap_width(width);
    }
}
