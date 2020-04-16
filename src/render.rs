use std::sync::Arc;
use std::cell::RefCell;
use cgmath::{
    Point3,
    Vector2,
};
use vulkano::buffer::{
    BufferAccess,
    TypedBufferAccess,
    CpuBufferPool,
    BufferUsage,
};

use vulkano::command_buffer::{
    AutoCommandBuffer,
    AutoCommandBufferBuilder,
    DynamicState,
};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{
    Device,
};
use vulkano::framebuffer::{
    RenderPassAbstract,
    Subpass,
    FramebufferAbstract,
};
use vulkano::swapchain::{
    SwapchainCreationError,
    AcquireError,
};
use vulkano::pipeline::{
    GraphicsPipeline,
    GraphicsPipelineAbstract,
};
use vulkano::sync::{
    self,
    GpuFuture,
};
use vulkano::format::ClearValue;

use winit::event::{ 
    Event, 
    WindowEvent,
    DeviceEvent,
};
use winit::event_loop::ControlFlow;
use winit::platform::desktop::EventLoopExtDesktop;

mod buffers;
use buffers::Vertex;

mod uniform_buffer_object;
use uniform_buffer_object::UniformBufferObject;

mod shaders;
use shaders::vertex_shader;
use shaders::fragment_shader;

mod core;
use self::core::RenderCore;

mod text;
use self::text::{
    TextContext,
    DrawsText,
};

pub struct EditorApplication {
    core: RenderCore,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    swap_chain_frame_buffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    text_context: RefCell<TextContext>,

    dynamic_state: DynamicState,
    command_buffers: Vec<Arc<AutoCommandBuffer>>,

    vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    index_buffer: Arc<dyn TypedBufferAccess<Content=[u16]> + Send + Sync>,
    //uniform_buffer_pool: CpuBufferPool<UniformBufferObject>,

    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,
}

impl EditorApplication {
    pub fn new(title: &str) -> Self {
        let core = RenderCore::new(title);
        let render_pass = core.create_render_pass(None);
        let graphics_pipeline = Self::create_graphics_pipeline(&core.get_device(), &render_pass);

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };
        let swap_chain_frame_buffers = core.create_framebuffers(&render_pass, &mut dynamic_state);

        let device = core.get_device();
        let graphics_queue = core.get_graphics_queue();

        let mut text_context = RefCell::<TextContext>::new(TextContext::new(
            device.clone(), 
            graphics_queue.clone(), 
            core.swap_chain.clone(), 
            &core.swap_chain_images)); 

        let device: &Arc<Device> = &device.clone();
        let graphics_queue = &graphics_queue.clone();
        let vertex_buffer = buffers::create_vertex_buffer(graphics_queue);
        let index_buffer = buffers::create_index_buffer(graphics_queue);
        //let uniform_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());

        let previous_frame_end = Some(Self::create_sync_objects(device));

        let mut app = Self {
            core,
            graphics_pipeline,
            swap_chain_frame_buffers,

            text_context,

            dynamic_state,
            render_pass,

            vertex_buffer,
            index_buffer,
            //uniform_buffer_pool,

            command_buffers: vec![],

            previous_frame_end,
            recreate_swap_chain: false,
        };

        app.create_command_buffers();
        app
    }

    pub fn run(&mut self) {

        let mut done = false;
        let mut recreate = false;
        let mut draw = false;
        let mut mouse_delta = Vector2::<f64>::new(0.0, 0.0);

        while !done {
            draw = false;
            recreate = false;

            self.core.get_events_loop().run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                
                match event {
                    Event::UserEvent(event) => println!("user event: {:?}", event),
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        done = true;
                    },
                    Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                        recreate = true;
                    },
                    Event::MainEventsCleared => {
                        *control_flow = ControlFlow::Exit;
                    },
                    Event::RedrawEventsCleared => {
                        draw = true;
                    },
                    Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                        mouse_delta += Vector2 { x: delta.0, y: delta.1 };
                    }
                    _ => (),
                }
            });

            if recreate {
                self.recreate_swap_chain = recreate;
            }
            if draw {
                // self.mouse_position += Vector2 { x: mouse_delta.x as f32, y: mouse_delta.y as f32 };
                self.draw_frame();
            }
        }
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<dyn GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>
    }

    fn draw_frame(&mut self) { 
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swap_chain {
            self.recreate_swap_chain();
            self.recreate_swap_chain = false;
        }

        self.text_context.borrow_mut().queue_text(
            200.0, 100.0, 100.0, [1.0, 1.0, 1.0, 1.0], 
            "Chicken");

        let (image_index, suboptimal, acquire_future) = match self.core.get_next_swap_chain_image() {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                println!("Setting recreate swap chain to true");
                self.recreate_swap_chain = true;
                return
            },
            Err(err) => panic!("Failed to acquire next image: {:?}", err)
        };

        if suboptimal {
            println!("Printing suboptimal image, recreating next frame");
            self.recreate_swap_chain = true;
        }

        let command_buffer = self.command_buffers[image_index].clone();

        let future = self.previous_frame_end.take()
            .expect("unable to take previous_frame_end future");

        let future = future
            .join(acquire_future)
            .then_execute(self.core.get_graphics_queue().clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.core.get_present_queue().clone(), self.core.swap_chain.clone(), image_index)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                println!("FlushError::OutOfDate: recreating swap_chain next frame");
                self.recreate_swap_chain = true;
                self.previous_frame_end = Some(self.core.get_previous_frame_end() as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end = Some(self.core.get_previous_frame_end() as Box<_>);
            }
        }
    }

    fn recreate_swap_chain(&mut self) {
        let dimensions: [u32; 2] = self.core.get_window().inner_size().into();
        let (new_swapchain, new_images) = match self.core.swap_chain.recreate_with_dimensions(dimensions) {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => return,
            Err(err) => panic!("Failed to recreate swapchain: {:?}", err)
        };

        self.core.swap_chain = new_swapchain;
        self.swap_chain_frame_buffers = self.core.create_framebuffers(
            &self.render_pass, 
            &mut self.dynamic_state);

        self.text_context = RefCell::<TextContext>::new(TextContext::new(
            self.core.get_device().clone(), 
            self.core.get_graphics_queue().clone(), 
            self.core.swap_chain.clone(), 
            &new_images,
        ));
        self.core.swap_chain_images = new_images;
    }

    fn create_graphics_pipeline(
        device: &Arc<Device>, 
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {

        let _vert_shader_mod = vertex_shader::Shader::load(device.clone())
            .expect("failed to create vertex shader module");
        let _frag_shader_mod = fragment_shader::Shader::load(device.clone())
            .expect("failed to create fragment shader module");
       
        Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(_vert_shader_mod.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(_frag_shader_mod.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
        )
    }

    fn create_command_buffers(&mut self) {
        let q_family = self.core.get_graphics_queue().family();

        let dimensions = self.core.swap_chain_images[0].dimensions();

/*
        let layout = self.graphics_pipeline.descriptor_set_layout(0).unwrap();
        let uniform_buffer = {
            let uniform_subbuffer = UniformBufferObject::from_dimensions(
                Point3::<f32>::new(0.0, 0.0, 0.0), // TODO MOUSE 
                [dimensions[0] as f32, dimensions[1] as f32]);
            
            self.uniform_buffer_pool.next(uniform_subbuffer).unwrap()
        };
        let set = Arc::new(PersistentDescriptorSet::start(layout.clone())
            .add_buffer(uniform_buffer).unwrap()
            .build().unwrap());

        self.command_buffers = self.swap_chain_frame_buffers
            .iter()
            .enumerate()
            .map(|(image_index, framebuffer)| {
                Arc::new(
                    AutoCommandBufferBuilder::primary_one_time_submit(
                            self.core.get_device().clone(), q_family).unwrap()
                        .begin_render_pass(
                            framebuffer.clone(), false, vec!(ClearValue::from([0.0, 0.0, 0.0, 1.0]))).unwrap()
                        .draw_indexed(
                            self.graphics_pipeline.clone(), 
                            &self.dynamic_state, 
                            vec![self.vertex_buffer.clone()], 
                            self.index_buffer.clone(), (), ()
                        ).unwrap()
                         .end_render_pass().unwrap()
                         //.draw_text(&mut self.text_context.borrow_mut(), image_index)
                         .build().unwrap()
                )
            }).collect();
    }
}

