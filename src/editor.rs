use std::cell::RefCell;
use std::str;
use std::time::Instant;

use super::render::Renderer;
use super::events::{
    EditorEventLoop,
    EditorEvent,
    self,
};
use events::state::InputState;

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
    Matrix4,
};

#[derive(Debug)]
pub struct EditorState {
    camera_pos: Point3<f32>,
    camera_direction: Vector3<f32>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            camera_pos: Point3::<f32>::new(2.0, 2.0, 2.0),
            camera_direction: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn move_camera(&mut self, delta: (f32, f32, f32)) {
        let delta_vec = Vector3::<f32>::new(delta.0, delta.1, delta.2);
        self.camera_pos += delta_vec;
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera_pos.z += delta;
    }

    pub fn to_uniform_buffer(&self, dimensions: [f32; 2]) -> UniformBufferObject {
        UniformBufferObject::new(
            self.camera_pos,
            self.camera_direction,
            45.0,
            dimensions,
        )
    }
}

fn draw_info(editor_state: &mut EditorState, renderer: &mut Renderer, fps: f32) {
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

    let pos = editor_state.camera_pos;
    let pos = format_slice("Position: ", &[pos.x, pos.y, pos.z]); 
    renderer.queue_text([20.0, 20.0], 20.0, pos.as_str());

    let pos = editor_state.camera_direction;
    let pos = format_slice("Direction: ", &[pos.x, pos.y, pos.z]); 
    renderer.queue_text([20.0, 40.0], 20.0, pos.as_str());
    
    let mut fps_str = String::from("FPS: ");
    fps_str.push_str(fps.to_string().as_str());
    renderer.queue_text([20.0, 60.0], 20.0, fps_str.as_str());
}

pub fn run(title: &str) {
    let events_loop = &mut events::create_event_loop();
    let renderer = RefCell::from(Renderer::new(&events_loop, title));
    let mut editor_state = EditorState::new();
    let mut done = false;
    let mut last_frame: Instant;

    while !done {
        last_frame = Instant::now();

        events_loop.run_return(|event: Event<'_, EditorEvent>, _, control_flow: &mut ControlFlow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::UserEvent(event) => {
                    match event {
                        EditorEvent::LineUpdate(line_update) => {
                           println!("{:?}", line_update); 
                        },
                    }
                },
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    done = true;
                },
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                    renderer.borrow_mut().recreate_swap_chain_next_frame();
                },
                Event::RedrawEventsCleared => {
                    let dimensions = renderer.borrow().get_screen_dimensions();
                    let ubo = editor_state.to_uniform_buffer(dimensions);
                    renderer.borrow_mut().draw_frame(ubo);

                    let fps_val = 1.0f32 / Instant::from(last_frame).elapsed().as_secs_f32() * 1000.0f32;
                    draw_info(&mut editor_state, &mut renderer.borrow_mut(), fps_val); 

                    last_frame = Instant::now();
                },
                Event::WindowEvent { event, .. } => {
                    events::handle_input(&mut editor_state, event);
                },
                _ => (),
            }
        });
    }
}
