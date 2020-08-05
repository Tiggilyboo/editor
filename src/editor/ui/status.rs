use std::boxed::Box;
use std::hash::{
    Hash,
    Hasher,
};
use glyph_brush::{
    Section,
    OwnedSection,
    Layout,
    HorizontalAlign,
    Text,
    ab_glyph::PxScale,
};

use super::widget::{
    Widget,
    hash_widget,
};
use super::primitive::PrimitiveWidget;
use super::view::Resources;

use crate::editor::{
    Mode,
    Action,
    Motion,
};
use crate::render::Renderer;

type ColourRGBA = [f32; 4];

// TODO: Derive from config
const MODE_NORMAL_COLOUR: ColourRGBA = [0.3, 0.9, 0.3, 1.0];
const MODE_INSERT_COLOUR: ColourRGBA = [0.0, 0.6, 1.0, 1.0];
const MODE_SELECT_COLOUR: ColourRGBA = [0.8, 0.0, 0.8, 1.0];
const MODE_REPLACE_COLOUR: ColourRGBA = [1.0, 0.5, 0.5, 1.0];

pub struct StatusWidget {
    index: usize,
    position: [f32; 2],
    depth: f32,
    scale: f32,
    size: [f32; 2],
    bg_colour: ColourRGBA,
    fg_colour: ColourRGBA,
    mode_colour: ColourRGBA,
    status: Status,
    background: PrimitiveWidget,
    mode_primitive: PrimitiveWidget,
    mode_section: OwnedSection,
    filename_section: OwnedSection,
    status_section: OwnedSection,
    command_section: OwnedSection,
    command_select_pos: usize,

    dirty: bool,
}

#[derive(Hash, Clone)]
pub struct Status {
    pub mode: Mode,
    pub filename: Option<String>,
    pub line_current: usize,
    pub line_count: usize,
    pub language: Option<String>,
}

impl Hash for StatusWidget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_widget(self, state);
        self.depth.to_le_bytes().hash(state);
        self.scale.to_le_bytes().hash(state);
        self.bg_colour.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.fg_colour.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.mode_colour.iter().for_each(|b| b.to_le_bytes().hash(state));
        self.status.hash(state);
        self.background.hash(state);
        self.mode_primitive.hash(state);
    }
}

impl Widget for StatusWidget {
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> [f32; 2] {
        self.position
    }

    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn queue_draw(&mut self, renderer: &mut Renderer) {
        let (width, height) = (self.size[0], self.size[1]);
        let text_ctx = renderer.get_text_context().clone();

        // Primitives (Background quads)
        self.background.queue_draw(renderer);
        self.mode_primitive.queue_draw(renderer);

        // Mode
        if let ctx = &mut text_ctx.borrow_mut() {
            ctx.queue_text(&self.mode_section.to_borrowed());
            if self.mode() == Mode::Command {
                ctx.queue_text(&self.command_section.to_borrowed());
            } else {
                ctx.queue_text(&self.filename_section.to_borrowed());
            }
        
            // Status
            let status_width = ctx.get_text_width(&self.status_section.text[0].text.to_string());
            self.status_section.bounds = (status_width + self.scale, self.size[1]);
            self.status_section.screen_position = (self.size[0] - status_width - self.scale, self.position[1]);
            ctx.queue_text(&self.status_section.to_borrowed());    
        }

        self.filename_section.screen_position = (
            self.mode_primitive.position()[0] + self.mode_primitive.size()[0] + 6.0,
            self.scale * 1.03
        );
    }
}

#[inline]
fn create_empty_section(align: HorizontalAlign) -> OwnedSection {
    let layout = Layout::default_single_line()
        .h_align(align);
    Section::default()
        .add_text(Text::default()
            .with_color([0.0, 0.0, 0.0, 1.0])
        )
        .with_layout(layout)
        .to_owned()
}

impl StatusWidget {
    pub fn new(index: usize, status: Status, resources: &Resources) -> Self {
        let background = PrimitiveWidget::new(2, [0.0, 0.0, 0.2], [0.0, 0.0], resources.bg);
        let mode_primitive = PrimitiveWidget::new(3, [0.0, 0.0, 0.2], [0.0, 0.0], MODE_NORMAL_COLOUR);
        let command_section = create_empty_section(HorizontalAlign::Left);
        let filename_section = create_empty_section(HorizontalAlign::Left);
        let mode_section = create_empty_section(HorizontalAlign::Left);
        let status_section = create_empty_section(HorizontalAlign::Left);

        let mut widget = Self {
            index,
            status: status.clone(),
            size: [0.0, resources.scale * 1.03],
            position: [0.0, 0.0],
            bg_colour: resources.bg,
            fg_colour: resources.fg,
            mode_colour: resources.sel,
            scale: resources.scale,
            depth: 0.5,
            command_select_pos: 0,
            dirty: true,
            background,
            mode_primitive,
            command_section,
            filename_section,
            mode_section,
            status_section,
        };

        widget.set_scale(widget.scale);
        widget.set_colours(widget.bg_colour, widget.fg_colour, widget.mode_colour);
        widget.set_mode(status.mode);
        widget.update_filename(status.filename);
        widget.update_line_status(status.line_current, status.line_count, status.language.clone());

        widget
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        let mode_width = self.mode_primitive.size()[0];

        self.position = [x, y];
        self.background.set_position(x, y);
        self.mode_primitive.set_position(x, y);
        self.mode_section.screen_position = (x + (self.scale / 4.0), y);
        self.command_section.screen_position = (x + mode_width + self.scale, y);
        self.filename_section.screen_position = (x + mode_width, y);
        self.dirty = true;
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
        self.background.set_size(self.size);
        self.dirty = true;
    }
    
    pub fn set_mode_width(&mut self, mode_width: f32) {
        let width = mode_width + self.scale / 4.0;
        self.mode_primitive.set_size([width, self.size[1]]);
        self.mode_section.bounds = (width, self.size[1]);
        self.dirty = true;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode_section.text[0].text = mode.to_string();
        self.mode_section.text[0].extra.color = self.mode_colour;
        self.mode_primitive.set_colour(mode.colour());

        if mode != Mode::Command {
            self.update_command_section("");
            self.move_command_cursor(Motion::First);
        }

        self.status.mode = mode; 
        self.dirty = true;
    }

    pub fn set_colours(&mut self, bg: ColourRGBA, fg: ColourRGBA, mode: ColourRGBA) {
        self.fg_colour = fg;
        self.bg_colour = bg;
        self.mode_colour = mode;
        self.background.set_colour(self.bg_colour);

        self.mode_section.text[0].extra.color = mode;
        self.status_section.text[0].extra.color = fg;
        self.command_section.text[0].extra.color = fg;
        self.filename_section.text[0].extra.color = fg;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.size[1] = scale;

        let pxs = PxScale::from(scale);
        self.mode_section.text[0].scale = pxs;
        self.status_section.text[0].scale = pxs;
        self.command_section.text[0].scale = pxs;
        self.filename_section.text[0].scale = pxs;
    }

    pub fn update_line_status(&mut self, line_num: usize, line_count: usize, language: Option<String>) {
        let line_percent: usize = if line_count > 0 {
            ((line_num as f32 / line_count as f32) * 100.0) as usize
        } else {
            0
        };

        let status_content = format!("{} {}% {}/{}", 
            language.clone().unwrap_or(String::new()), 
            line_percent,
            line_num, line_count);
        println!("{}", status_content);

        self.status.line_count = line_count;
        self.status.line_current = line_num;
        self.status.language = language;

        self.status_section.text[0].text = status_content;
        self.status_section.text[0].scale = PxScale::from(self.scale);
        self.status_section.text[0].extra.color = self.fg_colour;
    }

    pub fn update_command_section(&mut self, command: &str) {
        self.command_section.text[0].text = command.to_string();
    }

    pub fn update_filename(&mut self, filename: Option<String>) {
        self.status.filename = filename;
    }

    pub fn mode(&self) -> Mode {
        self.status.mode
    }

    pub fn get_command(&self) -> String {
        self.command_section.text[0].text.to_string()
    }

    fn move_command_cursor(&mut self, motion: Motion) -> bool {
        match motion {
            Motion::Left => {
                if self.command_select_pos > 0 {
                    self.command_select_pos -= 1;
                }
            },
            Motion::Right => {
                if self.command_select_pos < self.command_section.text[0].text.len() {
                    self.command_select_pos += 1;
                } else {
                    self.command_select_pos = self.command_section.text[0].text.len();
                }
            },
            Motion::First => self.command_select_pos = 0,
            Motion::Last => self.command_select_pos = self.command_section.text[0].text.len(),
            _ => return false,
        }

        true
    }

    fn handle_char(&mut self, ch: char) -> bool {
        self.command_section.text[0].text.push(ch);
        self.move_command_cursor(Motion::Right);
        true
    }
    fn handle_delete(&mut self, motion: Motion) -> bool {
        match motion {
            Motion::Left => {
                if self.command_section.text[0].text.len() > 0
                && self.command_select_pos < self.command_section.text[0].text.len() {
                    self.command_section.text[0].text.remove(self.command_select_pos);
                    self.move_command_cursor(Motion::Left);
                    true
                } else {
                    false
                }
            },
            Motion::Right => {
                if self.command_select_pos + 1 < self.command_section.text[0].text.len() {
                    self.command_section.text[0].text.remove(self.command_select_pos + 1);
                    true
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    pub fn poke(&mut self, action: Box<Action>) -> bool {
        self.dirty = true;

        match *action {
            Action::Back => self.handle_delete(Motion::Left),
            Action::Delete => self.handle_delete(Motion::Right),
            Action::InsertChar(ch) => self.handle_char(ch),
            Action::Motion(motion) => self.move_command_cursor(motion),
            _ => false,
        }
    }
}

impl Mode {
    fn colour(&self) -> ColourRGBA {
        match self {
            Mode::Normal => MODE_NORMAL_COLOUR,
            Mode::Command => MODE_NORMAL_COLOUR,
            Mode::Insert => MODE_INSERT_COLOUR,
            Mode::Select => MODE_SELECT_COLOUR,
            Mode::BlockSelect => MODE_SELECT_COLOUR,
            Mode::LineSelect => MODE_SELECT_COLOUR,
            Mode::Replace => MODE_REPLACE_COLOUR,
            _ => MODE_NORMAL_COLOUR,
        }
    }
}
