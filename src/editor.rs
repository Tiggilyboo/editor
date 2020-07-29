mod xi_thread;

pub mod ui;
pub mod state;
pub mod rpc;
pub mod linecache;
pub mod font;

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
    from_value,
};

use ui::view::{
    EditView,
};
use ui::widget::Widget;
use state::EditorState;
use rpc::{ 
    Core, 
    Handler,
    Config, 
    Theme,
    EditViewCommands,
};
use super::events::{
    state::InputState,
    binding::Action,
};


#[derive(Clone)]
struct App {
    core: Arc<Mutex<Core>>,
    state: Arc<Mutex<EditorState>>,
    input: Arc<Mutex<InputState>>,
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
}

impl App {
    fn new(core: Core) -> Self { 
        Self {
            core: Arc::new(Mutex::new(core)),
            state: Arc::new(Mutex::new(EditorState::new())),
            input: Arc::new(Mutex::new(InputState::new())),
        }
    }

    fn get_core(&self) -> std::sync::MutexGuard<'_, Core, > {
        self.core.lock().unwrap()
    }

    fn send_notification(&self, method: &str, params: &Value) {
        self.get_core().send_notification(method, params);
    }

    fn send_view_cmd(&self, command: EditViewCommands) {
        let mut state = self.state.lock().unwrap();
        let view_state = state.get_focused_view();

        view_state.poke(command);
    }

    pub fn open_file_in_view(&self, filename: Option<&str>, screen_size: [f32; 2], font_size: f32, line_height: f32) {
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
            println!("sending request for new view: {}", view_id);

            if let Ok(ref mut state) = state.try_lock() {
                state.focused = Some(view_id.clone());
                state.views.insert(view_id.clone(), EditView::new(0, font_size));

                let edit_view = state.get_focused_view();
                edit_view.poke(EditViewCommands::Core(core));
                edit_view.poke(EditViewCommands::ViewId(view_id));
                edit_view.poke(EditViewCommands::Resize(screen_size));
            } else {
                println!("unable to lock state to set focused view_id with new EditView widget");
            }
        });
    }

    // TODO: Derive from config somewhere?
    fn set_default_theme(&self) {
        self.send_view_cmd(EditViewCommands::Action(Action::SetTheme("Solarized (dark)".to_string())));
    }

    fn handle_cmd(&self, method: &str, params: &Value) {
        match method {
            "update" => self.send_view_cmd(EditViewCommands::ApplyUpdate(params["update"].clone())),
            "scroll_to" => self.send_view_cmd(EditViewCommands::ScrollTo(params["line"].as_u64().unwrap() as usize)),
            "config_changed" => {
                let config = from_value::<Config>(params["changes"].clone()).unwrap();
                self.send_view_cmd(EditViewCommands::ConfigChanged(config));
                self.set_default_theme();
            },
            "available_themes" => {
                if let Ok(ref mut state) = self.state.clone().try_lock() {
                    let raw_themes = params["themes"].as_array();
                    if let Some(themes) = raw_themes {
                        let mut available_themes: Vec<String> = vec!();
                        for t in themes.iter() {
                            available_themes.push(String::from(t.as_str().unwrap()));
                        }

                        state.set_available_themes(available_themes);
                    }
                }
            },
            "theme_changed" => {
                let theme = from_value::<Theme>(params["theme"].clone()).unwrap();
                self.send_view_cmd(EditViewCommands::ThemeChanged(theme.clone()));
                if let Ok(ref mut state) = self.state.clone().try_lock() {
                    if let Some(name) = params["name"].as_str() {
                        state.set_theme(String::from(name));
                    }
                }
            },
            _ => println!("unhandled core->fe method: {}", method),
        }
    }

    fn update_input(&self, event: WindowEvent, window_dimensions: [f32; 2]) -> bool {
        let processed: bool;
        if let Ok(ref mut input) = self.input.try_lock() {
            processed = input.update(event, window_dimensions);
        } else {
            println!("unable to update_input from mutex lock");
            return false
        }
        if !processed {
            return false;
        }
        if let Ok(ref mut state) = self.state.try_lock() {
            state.update_from_input(self.input.clone())
        } else {
            println!("unable to lock state in update_input");
            false
        }
    }
}

impl Handler for AppDispatcher {
    fn notification(&self, method: &str, params: &Value) {
        println!("AppDispatcher rx method: {} = {}", method, params.to_string());
        if let Some(ref app) = *self.app.lock().unwrap() {
            app.handle_cmd(method, params);
        }
    }
}

pub fn run(title: &str) {
    let events_loop = events::create_event_loop();
    let renderer = RefCell::new(Renderer::new(&events_loop, title));
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();

    let handler = AppDispatcher::new();
    let (xi_peer, rx) = xi_thread::start_xi_thread();
    let core = Core::new(xi_peer, rx, handler.clone());
    let app = App::new(core);

    handler.set_app(&app);
    app.send_notification("client_started", &json!({
        "config_dir": "./config",
        "client_extras_dir": "./extras",
    }));
    app.open_file_in_view(None, screen_dimensions, 20.0, 23.0);

    events_loop.run(move |event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(_event) => {},
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                println!("WindowEvent::Resized to {} x {}", size.width, size.height);
                renderer.borrow_mut().recreate_swap_chain_next_frame();
                screen_dimensions[0] = size.width as f32;
                screen_dimensions[1] = size.height as f32;
                
                if let Ok(ref mut state) = app.state.clone().try_lock() {
                    if state.focused.is_some() {
                        let edit_view = state.get_focused_view();
                        edit_view.poke(EditViewCommands::Resize(screen_dimensions));
                    }
                }
            },
            Event::RedrawRequested(_window_id) => {
                println!("RedrawRequested");
                 
                renderer.borrow_mut().draw_frame();

                println!("Drawn");
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::Focused(_) => {
                    app.update_input(event, screen_dimensions);

                    if let Ok(ref mut state) = app.state.clone().try_lock() {
                        if state.focused.is_some() {
                            let view = state.get_focused_view();
                            if view.dirty() {
                                view.queue_draw(&mut renderer.borrow_mut());
                                view.set_dirty(false);
                                renderer.borrow().request_redraw();
                            }
                        }
                    } else {
                        println!("Unable to obtain state lock to queue_draw after input update");
                    }
                },
                _ => (),
            },
            _ => (),
        }
    });
}
