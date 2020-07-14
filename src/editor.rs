mod xi_thread;

pub mod ui;
pub mod state;
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
use ui::view::{
    EditView,
    EditViewCommands,
};
use state::{
    EditorState,
};
use super::events::state::InputState;

use rpc::{ Core, Handler };

#[derive(Clone)]
struct App {
    core: Arc<Mutex<Core>>,
    state: Arc<Mutex<EditorState>>,
}

#[derive(Clone)]
struct AppDispatcher {
    app: Arc<Mutex<Option<App>>>,
}

impl AppDispatcher {
    fn new() -> Self {
        Self {
            app: Default::default(),
        }
    }
    
    fn set_app(&self, app: &App) {
        *self.app.lock().unwrap() = Some(app.clone());
    }

    fn get_app(&self) -> std::sync::MutexGuard<'_, Option<App>, > {
        self.app.lock().unwrap()
    }
}

impl App {
    fn new(core: Core) -> Self { 
        Self {
            core: Arc::new(Mutex::new(core)),
            state: Arc::new(Mutex::new(EditorState::new())),
        }
    }

    fn get_core(&self) -> std::sync::MutexGuard<'_, Core, > {
        self.core.lock().unwrap()
    }

    fn get_state(&self) -> std::sync::MutexGuard<'_, EditorState, > {
        self.state.lock().unwrap()
    }
    
    fn send_notification(&self, method: &str, params: &Value) {
        self.get_core().send_notification(method, params);
    }

    fn send_view_cmd(&self, command: EditViewCommands) {
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

        let core = Arc::downgrade(&self.core);
        let state = self.state.clone();

        self.get_core().send_request("new_view", &params, move |value| {
            let view_id = value.clone().as_str().unwrap().to_string();
            let mut state = state.lock().unwrap();

            state.focused = Some(view_id.clone());
            state.views.insert(view_id.clone(), EditView::new(0, screen_size, font_size));

            let edit_view = state.get_focused_view();
            edit_view.poke(EditViewCommands::Core(core));
            edit_view.poke(EditViewCommands::ViewId(view_id));
        });
    }

    fn handle_cmd(&self, method: &str, params: &Value) {
        match method {
            "update" => self.send_view_cmd(EditViewCommands::ApplyUpdate(params["update"].clone())),
            "scroll_to" => self.send_view_cmd(EditViewCommands::ScrollTo(params["line"].as_u64().unwrap() as usize)),
            _ => println!("unhandled core->fe method: {}", method),
        }
    }
}

impl Handler for AppDispatcher {
    fn notification(&self, method: &str, params: &Value) {
        if let Some(ref app) = *self.app.lock().unwrap() {
            app.handle_cmd(method, params);
        }
    }
}

pub fn run(title: &str) {
    let events_loop = events::create_event_loop();
    let renderer = RefCell::new(Renderer::new(&events_loop, title));
    let mut input_state = InputState::new();
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();

    let handler = AppDispatcher::new();
    let (xi_peer, rx) = xi_thread::start_xi_thread();
    let core = Core::new(xi_peer, rx, handler.clone());
    let app = App::new(core);
    handler.set_app(&app);

    let state = app.state.clone();

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

                let mut state = state.lock().unwrap();
                state.queue_draw(&mut renderer.borrow_mut());
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
                    let mut state = state.lock().unwrap();
                    
                    state.update_from_input(&input_state);

                    if redraw {
                        state.queue_draw(&mut renderer.borrow_mut());
                    }
                },
                _ => (),
            },
            _ => (),
        }
    });
}
