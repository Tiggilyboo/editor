use std::cell::RefCell;
use std::str;
use std::time::{
    Instant,
    Duration,
};
use super::render::Renderer;
use super::events::{
    EditorEvent,
    self,
};

use winit::event_loop::ControlFlow;
use winit::event::{
    WindowEvent,
    Event,
};
use super::render::ui::widget::{
    Widget,
    WidgetKind,
};
use super::render::ui::update_ui;
use super::events::state::InputState;

pub struct EditorState {
    show_info: bool,
    time: Instant,
    
    pub widgets: Vec<WidgetKind>, 
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            show_info: true,
            widgets: vec!(),
            time: Instant::now(),
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
            _ => (),
        }
    }
}

pub fn run(title: &str) {
    let events_loop = events::create_event_loop();
    let renderer = RefCell::from(Renderer::new(&events_loop, title));
    let mut editor_state = EditorState::new();
    let mut input_state = InputState::new();
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();

    let mut initial_widgets = super::render::ui::create_initial_ui_state(
        screen_dimensions,
    );

    for w in initial_widgets.drain(..) {
        editor_state.add_widget(w);
    }

    events_loop.run(move |event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(event) => {
                match event {
                    EditorEvent::OpenWidget(widget) => {
                        println!("OpenWidget!");
                    },
                }
            },
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                println!("CloseRequested");
                *control_flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                println!("WindowEvent::Resized");
                renderer.borrow_mut().recreate_swap_chain_next_frame();
                screen_dimensions[0] = size.width as f32;
                screen_dimensions[1] = size.height as f32;

                update_ui(&mut editor_state, &mut renderer.borrow_mut());
            },
            Event::RedrawEventsCleared => {
            },
            Event::RedrawRequested(_window_id) => {
                println!("RedrawRequested");
                
                renderer.borrow_mut().draw_frame();
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::ModifiersChanged(_) => {
                    let redraw = input_state.update(event, screen_dimensions);
                    events::handle_input(&mut editor_state, &input_state);

                    if redraw {
                        update_ui(&mut editor_state, &mut renderer.borrow_mut());
                    }
                },
                _ => (),
            },
            _ => (),
        }
    });
}
