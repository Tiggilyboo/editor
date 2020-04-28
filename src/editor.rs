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
use super::render::ui::update_ui;
use super::events::state::InputState;
use super::render::camera::Camera;

pub struct EditorState {
    camera: Camera,
    show_info: bool,
    time: Instant,
    
    pub widgets: Vec<WidgetKind>, 
}

impl EditorState {
    pub fn new() -> Self {
        let camera = Camera::default();
        Self {
            camera,
            show_info: true,
            widgets: vec!(),
            time: Instant::now(),
        }
    }
    
    pub fn time_elapsed(&self) -> Duration {
        self.time.elapsed()
    }

    pub fn translate_camera(&mut self, delta: (f32, f32), time_delta: f32) {
        self.camera.move_camera(delta, time_delta);
    }

    pub fn get_camera_position(&self) -> Point3<f32> {
        self.camera.position
    }

    pub fn get_camera_direction(&self) -> Vector3<f32> {
        self.camera.front
    }

    pub fn zoom(&mut self, delta: f32) {
        self.camera.zoom(delta)
    }

    pub fn camera_direction(&mut self, mouse_delta: (f32, f32)) {
        self.camera.direction(mouse_delta);
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

    pub fn to_uniform_buffer(&self, dimensions: [f32; 2]) -> UniformBufferObject {
        UniformBufferObject::new(
            &self.camera,
            dimensions,
        )
    }
}

pub fn run(title: &str) {
    let events_loop = &mut events::create_event_loop();
    let renderer = RefCell::from(Renderer::new(&events_loop, title));
    let mut editor_state = EditorState::new();
    let mut input_state = InputState::new();
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
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::ModifiersChanged(_) => {
                        input_state.update(event, screen_dimensions);
                        events::handle_input(&mut editor_state, &input_state, screen_dimensions);
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
