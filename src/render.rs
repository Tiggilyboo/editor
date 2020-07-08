use std::sync::Arc;
use std::cell::RefCell;

use vulkano::command_buffer::{
    AutoCommandBuffer,
    AutoCommandBufferBuilder,
    DynamicState,
};
use vulkano::device::Device;
use vulkano::framebuffer::{
    RenderPassAbstract,
    FramebufferAbstract,
};
use vulkano::swapchain::{
    SwapchainCreationError,
    AcquireError,
};
use vulkano::sync::{
    self,
    GpuFuture,
};

mod core;
use self::core::RenderCore;

mod text;
use self::text::{
    TextContext,
};

pub mod ui;

use super::events::EditorEventLoop;

pub struct Renderer {
    core: RenderCore,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    swap_chain_frame_buffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    dynamic_state: DynamicState,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,

    text_context: RefCell<TextContext>,
}

impl Renderer {
    pub fn new(events_loop: &EditorEventLoop, title: &str) -> Self {
        let core = RenderCore::new(events_loop, title);
        let render_pass = core.create_render_pass(None);

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

        let text_context = RefCell::<TextContext>::new(TextContext::new(
            device.clone(), 
            graphics_queue.clone(), 
            core.swap_chain.clone(), 
            &core.swap_chain_images)); 

        let device: &Arc<Device> = &device.clone();
        let previous_frame_end = Some(Self::create_sync_objects(device));

        Self {
            core,
            swap_chain_frame_buffers,

            text_context,

            dynamic_state,
            render_pass,

            previous_frame_end,
            recreate_swap_chain: false,
        }
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<dyn GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>
    }

    pub fn queue_text(&mut self, pos: [f32; 2], colour: [f32; 4], font_size: f32, text: &str) {
        self.text_context.borrow_mut()
            .queue_text(pos[0], pos[1], font_size, colour, text)
    }

    pub fn draw_frame(&mut self) { 
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

        let future = self.previous_frame_end.take()
            .expect("unable to take previous_frame_end future");

        let future = future
            .join(acquire_future)
            .then_execute(self.core.get_graphics_queue().clone(), command_buffer).unwrap()
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
        self.core.swap_chain_images = new_images;
        
        self.render_pass = self.core.create_render_pass(None);

        self.swap_chain_frame_buffers = self.core.create_framebuffers(
            &self.render_pass, 
            &mut self.dynamic_state);

        self.text_context = RefCell::from(TextContext::new(
            self.core.get_device().clone(),
            self.core.get_graphics_queue().clone(),
            self.core.swap_chain.clone(),
            &self.core.swap_chain_images,
        ));
    }

    fn create_command_buffer(&mut self, image_index: usize) -> Arc<AutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                self.core.get_device().clone(),
                self.core.get_graphics_queue().family()
        ).expect("unable to create AutoCommandBufferBuilder");

        self.text_context.borrow_mut().draw_text(&mut builder, image_index);

        let command_buffer = builder.build()
            .expect("unable to build command buffer from builder");

        Arc::new(command_buffer)
    }

    pub fn recreate_swap_chain_next_frame(&mut self) {
        self.recreate_swap_chain = true;
    }

    pub fn get_screen_dimensions(&self) -> [f32; 2] {
        self.core.get_window().inner_size().into()
    }
}

