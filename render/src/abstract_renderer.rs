use std::sync::Arc;
use winit::window::Window;
use vulkano::swapchain::Swapchain;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::Framebuffer;

pub trait AbstractRenderer {
    fn set_swap_chain(&mut self, swap_chain: Arc<Swapchain<Window>>, swap_chain_images: &Vec<Arc<SwapchainImage<Window>>>);
    fn get_pipeline(&self) -> Arc<GraphicsPipeline>;
    fn get_framebuffers(&self) -> Vec<Arc<Framebuffer>>;
}

