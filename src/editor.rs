use std::cell::RefCell;
use std::str;
use std::time::Instant;

use super::render::Renderer;
use super::events::{
    EditorEvent,
    self,
};

use winit::event_loop::ControlFlow;
use winit::platform::desktop::EventLoopExtDesktop;
use winit::event::{
    WindowEvent,
    Event,
};
use super::render::uniform_buffer_object::UniformBufferObject;
use cgmath::{
    Point3, 
    Vector3,
};
use super::render::ui::widget::{
    Widget,
    WidgetKind,
};

pub struct EditorState {
    camera_pos: Point3<f32>,
    camera_direction: Vector3<f32>,
    camera_pitch: f32,
    show_info: bool,
    
    widgets: Vec<WidgetKind>, 
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            camera_pos: Point3::<f32>::new(2.0, 2.0, 2.0),
            camera_direction: Vector3::new(-1.0, -1.0, -1.0),
            camera_pitch: 45.0,
            show_info: true,
            widgets: vec!(),
        }
    }

    pub fn move_camera(&mut self, delta: (f32, f32, f32)) {
        let delta_vec = Vector3::<f32>::new(delta.0, delta.1, delta.2);
        self.camera_pos += delta_vec;
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera_pos.z += delta;
    }

    pub fn pitch(&mut self, delta: f32) {
        self.camera_pitch += delta;
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

    pub fn get_widget(&mut self, index: usize) -> &mut WidgetKind {
        let widget = self.widgets.get_mut(index)
            .expect("unable to get widget from state");

        widget
    }

    pub fn to_uniform_buffer(&self, dimensions: [f32; 2]) -> UniformBufferObject {
        UniformBufferObject::new(
            self.camera_pos,
            self.camera_direction,
            self.camera_pitch,
            dimensions,
        )
    }
}

fn update_ui(editor_state: &mut EditorState, renderer: &mut Renderer, fps: f32) {
    fn format_slice(prefix: &str, vec: &[f32]) -> String {
        let mut text = String::from(prefix);
        let len = vec.len();
        text.push('[');

        for (i, v) in vec.iter().enumerate() {
            text.push_str(v.to_string().as_str());
            if i != len - 1 {
                text.push_str(", ");
            }
        }
        text.push(']');

        text
    }
 
    let mut new_content: [String; 3] = [
        String::with_capacity(32),
        String::with_capacity(32),
        String::with_capacity(32),
    ]; 

    let pos = editor_state.camera_pos;
    let pos = format_slice("Position: ", &[pos.x, pos.y, pos.z]); 
    new_content[0] = pos;
    
    let pos = editor_state.camera_direction;
    let pos = format_slice("Direction: ", &[pos.x, pos.y, pos.z]); 
    new_content[1] = pos;
    
    let mut fps_str = String::from("FPS: ");
    fps_str.push_str(fps.to_string().as_str());
    new_content[2] = fps_str;

    let mut i = 0;
    for w in editor_state.widgets.iter_mut() {
        match w {
            WidgetKind::Text(text_widget) => {
                text_widget.set_content(new_content[i].as_str());
                text_widget.draw(renderer);
            },
            _ => (),
        }

        i += 1;
    }
}

pub fn run(title: &str) {
    let events_loop = &mut events::create_event_loop();
    let renderer = RefCell::from(Renderer::new(&events_loop, title));
    let mut editor_state = EditorState::new();
    let mut last_frame = Instant::now();
    let mut done = false;
    let mut screen_dimensions: [f32; 2] = renderer.borrow().get_screen_dimensions();

    let mut initial_widgets = super::render::ui::create_initial_ui_state(
        screen_dimensions,
    );
    for w in initial_widgets.drain(..) {
        editor_state.add_widget(w);
    }

    let mut frames: usize = 0;
    while !done {
        events_loop.run_return(|event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::UserEvent(event) => {
                    match event {
                        EditorEvent::OpenWidget(widget) => {
                        },
                    }
                },
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    done = true;
                },
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    renderer.borrow_mut().recreate_swap_chain_next_frame();
                    screen_dimensions[0] = size.width as f32;
                    screen_dimensions[1] = size.height as f32;
                },
                Event::RedrawEventsCleared => {
                    let ubo = editor_state.to_uniform_buffer(screen_dimensions);
                    renderer.borrow_mut().draw_frame(ubo);
                    
                    let fps_val = frames as f32 / last_frame.elapsed().as_secs_f32();
                    frames = 0;

                    if editor_state.show_info {
                        update_ui(&mut editor_state, &mut renderer.borrow_mut(), fps_val); 
                    }
                },
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { .. }
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::MouseWheel { .. }
                    | WindowEvent::ModifiersChanged(_) => {
                        events::handle_input(&mut editor_state, event, screen_dimensions);
                    },
                    _ => (),
                },
                _ => (),
            }
        });
        
        last_frame = Instant::now();
        frames += 1;
    }
}
