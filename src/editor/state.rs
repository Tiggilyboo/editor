use std::collections::HashMap;
use std::time::{
    Instant,
    Duration,
};
use super::ui::{
    view::EditView,
    widget::Widget,
    widget::WidgetKind,
};
use crate::render::Renderer;
use crate::events::{
    state::InputState,
    mapper_winit::map_input_into_string,
};

use winit::event::VirtualKeyCode;

pub type ViewId = String;

pub struct EditorState {
    show_info: bool,
    time: Instant,

    pub focused: Option<ViewId>,
    pub views: HashMap<ViewId, EditView>,
    pub widgets: Vec<WidgetKind>, 
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            show_info: true,
            widgets: vec!(),
            time: Instant::now(),
            focused: Default::default(),
            views: HashMap::new(),
        }
    }
    
    pub fn time_elapsed(&self) -> Duration {
        self.time.elapsed()
    }

    pub fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }

    pub fn add_widget(&mut self, widget: WidgetKind) {
        match widget {
            WidgetKind::Text(text_widget) => {
                self.widgets.insert(text_widget.index(), WidgetKind::Text(text_widget));
            },
            WidgetKind::View(view_widget) => {
                self.widgets.insert(view_widget.index(), WidgetKind::View(view_widget));
            },
        }
    }

    pub fn get_focused_view(&mut self) -> &mut EditView {
        let view_id = self.focused.clone()
            .expect("no focused EditView");

        self.views.get_mut(&view_id)
            .expect("Focused EditView not found in views")
    }

    pub fn queue_draw(&mut self, renderer: &mut Renderer) {
        let mut requires_redraw = false;

        for w in self.widgets.iter_mut() {
            match w {
                WidgetKind::Text(text_widget) => {
                    if text_widget.dirty() {
                        text_widget.queue_draw(renderer);
                        requires_redraw = true;
                    }
                },
                WidgetKind::View(view) => {
                    if view.dirty() {
                        view.queue_draw(renderer);
                        requires_redraw = true;
                    }
                },
            }
        }

        if requires_redraw {
            renderer.request_redraw();
        }
    }

    pub fn update_from_input(&mut self, input_state: &InputState) {
        let delta_time = self.time_elapsed().as_secs_f32();

        if input_state.keycode.is_some() {
            match input_state.keycode.unwrap() {
                VirtualKeyCode::F1 => {
                    self.toggle_info();
                },
                _ => {
                    let should_keydown = input_state.keycode.is_some();
                    let should_mouse = input_state.mouse.button.is_some()
                        || input_state.mouse.line_scroll.1 != 0.0;

                    if !should_keydown && !should_mouse {
                        println!("!keydown && !mouse");
                        return;
                    }

                    if let Some(widget_view) = &mut self.widgets.iter_mut().filter_map(|w| {
                        match w {
                            WidgetKind::View(view) => Some(view),
                            _ => None,
                        }
                    }).next() {
                        if should_keydown {
                            if let Some(input_string) = map_input_into_string(input_state.modifiers, input_state.keycode) {
                                let ch = input_string.chars().next().unwrap();
                                widget_view.char(ch);
                            } else {
                                widget_view.keydown(input_state.keycode.unwrap(), input_state.modifiers);
                            }
                        }
                        if should_mouse {
                            widget_view.mouse_scroll(input_state.mouse.line_scroll.1);
                        }
                    }
                },
            }
        }
    }
}
