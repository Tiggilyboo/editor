extern crate vulkano;
extern crate vulkano_win;
extern crate vulkano_shaders;
extern crate winit;
extern crate glyph_brush;

mod core;

pub mod text;
pub mod primitive;
pub mod uniform;
pub mod colour;
mod abstract_renderer;

use std::sync::Arc;
use std::cell::RefCell;

use abstract_renderer::AbstractRenderer;

use vulkano::command_buffer::{
    PrimaryAutoCommandBuffer,
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    SubpassContents,
};
use vulkano::device::Device;
use vulkano::render_pass::{
    RenderPass,
    Framebuffer,
    Subpass,
};
use vulkano::swapchain::{
    SwapchainCreationError,
    AcquireError,
};
use vulkano::sync::{
    self,
    GpuFuture,
};
use winit::event_loop::EventLoop;

use self::core::RenderCore;
use self::text::TextRenderer;
use self::primitive::PrimitiveRenderer;

pub struct Renderer {
    core: RenderCore,
    render_pass: Arc<RenderPass>,
    frame_buffers: Vec<Arc<Framebuffer>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,

    text_renderer: Arc<RefCell<TextRenderer>>,
    primitive_renderer: Arc<RefCell<PrimitiveRenderer>>,
}

impl Renderer {
    pub fn new<L>(events_loop: &EventLoop<L>, title: &str) -> Self {
        let core = RenderCore::new(events_loop, title);
        let render_pass = core.create_render_pass(None);
        let frame_buffers = core.create_framebuffers(&render_pass);

        let device = core.get_device();
        let graphics_queue = core.get_graphics_queue();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let text_renderer = Arc::new(RefCell::new(
                TextRenderer::new(graphics_queue.clone(), subpass.clone())
        )); 

        let primitive_renderer = Arc::new(RefCell::new(
                PrimitiveRenderer::new(graphics_queue.clone(), subpass.clone())
        ));

        let device: &Arc<Device> = &device.clone();
        let previous_frame_end = Some(Self::create_sync_objects(device));

        Self {
            core,
            frame_buffers,

            text_renderer,
            primitive_renderer,

            render_pass,

            previous_frame_end,
            recreate_swap_chain: false,
        }
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<dyn GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>
    }

    pub fn get_text_renderer(&mut self) -> Arc<RefCell<TextRenderer>> {
        self.text_renderer.clone()
    }

    pub fn get_primitive_renderer(&mut self) -> Arc<RefCell<PrimitiveRenderer>> {
        self.primitive_renderer.clone()
    }

    pub fn draw_frame(&mut self) { 
        //println!("Drawing frame...");
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swap_chain {
            self.recreate_swap_chain();
            self.recreate_swap_chain = false;
            return
        }

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
       
        let command_buffer = self.create_command_buffer(image_index); 
        if command_buffer.is_none() {
            println!("Command buffer was empty for image_index: {:?}", image_index);
            return
        }

        let future = self.previous_frame_end.take()
            .expect("unable to take previous_frame_end future");

        let future = future
            .join(acquire_future)
            .then_execute(
                self.core.get_graphics_queue().clone(), 
                command_buffer.unwrap()
            ).unwrap()
            .then_swapchain_present(
                self.core.get_present_queue().clone(), 
                self.core.swap_chain.clone(), 
                image_index)
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
        let dimensions = self.get_window_dimensions();
        let (new_swapchain, new_images) = match self.core.swap_chain
            .recreate()
            .dimensions(dimensions)
            .build() {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => return,
            Err(err) => panic!("Failed to recreate swapchain: {:?}", err)
        };

        self.core.swap_chain = new_swapchain;
        self.core.swap_chain_images = new_images;
        
        self.render_pass = self.core.create_render_pass(None);
        self.frame_buffers = self.core.create_framebuffers(&self.render_pass);
    }

    fn create_command_buffer(&mut self, image_idx: usize) -> Option<Arc<PrimaryAutoCommandBuffer>> {
        let mut builder = AutoCommandBufferBuilder::primary(
                self.core.get_device().clone(),
                self.core.get_graphics_queue().family(),
                CommandBufferUsage::OneTimeSubmit,
        ).expect("unable to create AutoCommandBufferBuilder");

        builder.begin_render_pass(
            self.frame_buffers[image_idx].clone(),
            SubpassContents::SecondaryCommandBuffers,
            vec![[0.0; 4].into()],
        ).unwrap();

        let dimensions = self.get_window_dimensions();
        
        let primitive_buffer = self.primitive_renderer
            .borrow_mut()
            .draw(dimensions);

        let text_buffer = self.text_renderer
            .borrow_mut()
            .draw(dimensions);

        builder.execute_commands(primitive_buffer).unwrap();
        builder.execute_commands(text_buffer).unwrap();

        builder.end_render_pass().unwrap();

        let command_buffer = builder
            .build()
            .expect("unable to build command buffer from builder");

        Some(Arc::new(command_buffer))
    }

    pub fn recreate_swap_chain_next_frame(&mut self) {
        self.recreate_swap_chain = true;
    }

    pub fn get_window_dimensions(&self) -> [u32; 2] {
        self.core.get_window().inner_size().into()
    }


    pub fn get_screen_dimensions(&self) -> [f32; 2] {
        self.core.get_window().inner_size().into()
    }

    pub fn request_redraw(&self) {
        self.core.get_window().request_redraw();
    }
}

