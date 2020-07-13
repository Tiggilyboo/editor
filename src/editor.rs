mod xi_thread;
mod state;

pub mod rpc;
pub mod linecache;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::str;

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
use serde_json::{
    Value,
    json,
};
use super::render::ui::view::{
    EditView,
    EditViewCommands,
};
use crate::editor::state::EditorState;
use super::render::ui::update_ui;
use super::events::state::InputState;

use rpc::{ Core, Handler };



struct Dispatcher {
    core: Arc<Mutex<Core>>,
    state: Arc<Mutex<EditorState>>,
}

impl Dispatcher {
    fn new(core: Core) -> Self {
        Self {
            core: Arc::new(Mutex::new(core)),
            state: Arc::new(Mutex::new(EditorState::new())),
        }
    }
    
    fn send_notification(&self, method: &str, params: &Value) {
        self.core.lock().unwrap()
            .send_notification(method, params);
    }

    fn send_view_cmd(&self, view_id: ViewId, command: EditViewCommands) {
        let mut state = self.state.lock().unwrap();
        let view_state = state.get_focused_view();

        view_state.poke(command);
    }

    pub fn open_file_in_view(&self, filename: Option<&str>, screen_size: [f32; 2], font_size: f32) {
        let mut params = json!({});

        let filename = if filename.is_some() {
            params["file_path"] = json!(filename.unwrap());
            Some(filename.unwrap().to_string())
        } else {
            None
        };

        let edit_view = 0;
        let core = Arc::downgrade(&self.core);
        let state = self.state.clone();

        self.core.lock().unwrap().send_request("new_view", &params, move |value| {
            let view_id = value.clone().as_str().unwrap().to_string();
            let mut state = state.lock().unwrap();
            
            state.focused = Some(view_id.clone());
            state.views.insert(view_id.clone(), EditView::new(0, screen_size, font_size));

            self.send_view_cmd(edit_view, EditViewCommands::Core(core));
            self.send_view_cmd(edit_view, EditViewCommands::ViewId(view_id));
            
        });
    }
}

impl Handler for Dispatcher {
    fn notification(&self, method: &str, params: &Value) {
        if let ref core = &self.core.lock() {
            match method {
                "update" => self.send_view_cmd(EditViewCommands::ApplyUpdate(params["update"].clone())),
            }
        }
    }
}

pub fn run(title: &str) {
    let events_loop = events::create_event_loop();
    let renderer = RefCell::from(Renderer::new(&events_loop, title));
    let mut input_state = InputState::new();
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();

    let handler = 
    let (xi_peer, rx) = xi_thread::start_xi_thread();
    let core = Core::new(xi_peer, rx, handler.clone());

    events_loop.run(move |event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(event) => {
                match event {
                    EditorEvent::OpenWidget(_widget) => {
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
