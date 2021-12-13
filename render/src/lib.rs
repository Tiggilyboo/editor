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
};
use vulkano::device::Device;
use vulkano::render_pass::{
    RenderPass,
    Framebuffer,
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
use self::text::TextContext;
use self::primitive::PrimitiveContext;

pub struct Renderer {
    core: RenderCore,
    render_pass: Arc<RenderPass>,
    swap_chain_frame_buffers: Vec<Arc<Framebuffer>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,

    text_context: Arc<RefCell<TextContext>>,
    primitive_context: Arc<RefCell<PrimitiveContext>>,
}

impl Renderer {
    pub fn new<L>(events_loop: &EventLoop<L>, title: &str, font_size: f32) -> Self {
        let core = RenderCore::new(events_loop, title);
        let render_pass = core.create_render_pass(None);
        let swap_chain_frame_buffers = core.create_framebuffers(&render_pass);

        let device = core.get_device();
        let graphics_queue = core.get_graphics_queue();

        let mut text_context = TextContext::new(device.clone(), graphics_queue.clone(), font_size); 
        text_context.set_swap_chain(core.swap_chain.clone(), &core.swap_chain_images);
        let text_context = Arc::new(RefCell::new(text_context));

        let mut primitive_context = PrimitiveContext::new(device.clone(), graphics_queue.clone());
        primitive_context.set_swap_chain(core.swap_chain.clone(), &core.swap_chain_images);
        let primitive_context = Arc::new(RefCell::new(primitive_context));

        let device: &Arc<Device> = &device.clone();
        let previous_frame_end = Some(Self::create_sync_objects(device));

        Self {
            core,
            swap_chain_frame_buffers,

            text_context,
            primitive_context,

            render_pass,

            previous_frame_end,
            recreate_swap_chain: false,
        }
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<dyn GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>
    }

    pub fn get_text_context(&mut self) -> Arc<RefCell<TextContext>> {
        self.text_context.clone()
    }

    pub fn get_primitive_context(&mut self) -> Arc<RefCell<PrimitiveContext>> {
        self.primitive_context.clone()
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
        let dimensions: [u32; 2] = self.core.get_window().inner_size().into();
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

        self.swap_chain_frame_buffers = self.core.create_framebuffers(&self.render_pass);

        println!("Setting text_context swap chain");
        self.text_context
            .borrow_mut()
            .set_swap_chain(
                self.core.swap_chain.clone(),
                &self.core.swap_chain_images);

        println!("Setting primitive_context swap chain");
        self.primitive_context
            .borrow_mut()
            .set_swap_chain(
                self.core.swap_chain.clone(),
                &self.core.swap_chain_images);
    }

    #[inline]
    fn create_command_buffer(&mut self, image_index: usize) -> Option<Arc<PrimaryAutoCommandBuffer>> {
        let mut builder = AutoCommandBufferBuilder::primary(
                self.core.get_device().clone(),
                self.core.get_graphics_queue().family(),
                CommandBufferUsage::OneTimeSubmit,
        ).expect("unable to create AutoCommandBufferBuilder");

        self.primitive_context
            .borrow_mut()
            .draw_primitives(&mut builder, image_index);

        self.text_context
            .borrow_mut()
            .draw_text(&mut builder, image_index);
            
        let command_buffer = builder.build()
            .expect("unable to build command buffer from builder");

        Some(Arc::new(command_buffer))
    }

    pub fn recreate_swap_chain_next_frame(&mut self) {
        self.recreate_swap_chain = true;
    }

    pub fn get_screen_dimensions(&self) -> [f32; 2] {
        self.core.get_window().inner_size().into()
    }

    pub fn request_redraw(&self) {
        self.core.get_window().request_redraw();
    }
}

