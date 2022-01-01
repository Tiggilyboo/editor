use render::{
    Renderer,
    colour::ColourRGBA,
};
use eddy::Mode;
use crate::widget::{
    Widget,
    Size,
    Position,
};

use crate::text::TextWidget;
use crate::primitive::PrimitiveWidget;
use crate::tree::WidgetTree;

pub struct StatusWidgetModeStyle {
    pub normal: ColourRGBA,
    pub insert: ColourRGBA,
    pub visual: ColourRGBA,
    pub command: ColourRGBA,
}

impl Default for StatusWidgetModeStyle {
    fn default() -> Self {
        Self {
            normal: [0.0, 0.8, 0.0, 1.0].into(),
            insert: [0.0, 0.0, 0.8, 1.0].into(),
            visual: [0.4, 0.0, 0.4, 1.0].into(),
            command: [0.0, 0.8, 0.0, 1.0].into(),
        }
    }
}

pub struct StatusWidgetState {
    pub mode: Mode,
    pub filepath: String,
    pub selected_line: usize,
    pub line_count: usize,
}

pub struct StatusWidget {
    dirty: bool,
    colour_foreground: ColourRGBA,
    colour_background: ColourRGBA,
    background: PrimitiveWidget,
    items: WidgetTree<TextWidget>,
    status: Option<StatusWidgetState>,
    mode_style: StatusWidgetModeStyle,
    scale: f32,
}

impl Widget for StatusWidget {
   fn dirty(&self) -> bool {
       self.dirty
   }
   fn set_dirty(&mut self, dirty: bool) {
       self.background.set_dirty(dirty);
       self.items.set_dirty(dirty);
       self.dirty = dirty;
   }
   fn size(&self) -> Size {
       self.background.size()
   }
   fn position(&self) -> Position {
       self.background.position()
   }
   fn queue_draw(&self, renderer: &mut Renderer) {
       self.background.queue_draw(renderer);
       self.items.queue_draw(renderer);
   }
}

impl StatusWidget {
    pub fn new(position: Position, size: Size, scale: f32, colour_background: ColourRGBA, colour_foreground: ColourRGBA) -> Self {
        let background = PrimitiveWidget::new(position, size, 0.3, colour_background);
        let items = WidgetTree::new();

        Self {
            colour_background,
            colour_foreground,
            background,
            items,
            scale,
            mode_style: StatusWidgetModeStyle::default(),
            status: None,
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

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.background.set_position(x, y);
    }

    fn determine_mode_colour(&self) -> ColourRGBA {
        if let Some(status) = &self.status {
            match status.mode {
                Mode::Normal => self.mode_style.normal,
                Mode::Insert => self.mode_style.insert,
                Mode::Visual => self.mode_style.visual,
                Mode::Command => self.mode_style.command,
                _ => self.mode_style.normal,
            }
        } else {
            self.colour_background
        }
    }

    pub fn populate(&mut self) {
        self.items.clear();

        if let Some(status) = &self.status {
            {
                let mode_text = format!("{:?}", status.mode);
                let mode_colour = self.determine_mode_colour();
                let mode_widget = TextWidget::with_text(mode_text, self.scale, mode_colour);
                self.items.insert(0, mode_widget);
            }
            {
                let file_text = status.filepath.clone();
                let file_widget = TextWidget::with_text(file_text, self.scale, self.colour_foreground);
                self.items.insert(1, file_widget);
            }
        } else {
            return;
        }
    }

    pub fn set_filepath(&mut self, filepath: String) {
        if let Some(status) = &mut self.status {
            status.filepath = filepath;
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if let Some(status) = &mut self.status {
            status.mode = mode;
        }
    }
}
