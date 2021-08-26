use std::sync::Arc;
use winit::window::Window;
use vulkano::swapchain::Swapchain;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::render_pass::FramebufferAbstract;

pub trait AbstractRenderer {
    fn set_swap_chain(&mut self, swap_chain: Arc<Swapchain<Window>>, swap_chain_images: &Vec<Arc<SwapchainImage<Window>>>);
    fn get_pipeline(&self) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync>;
    fn get_framebuffers(&self) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>>;
}
