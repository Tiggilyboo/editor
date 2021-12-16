use std::sync::Arc;
use vulkano::{
    device::Queue,
    pipeline::GraphicsPipeline,
    command_buffer::SecondaryAutoCommandBuffer,
    render_pass::Subpass,
};

pub trait AbstractRenderer {
    fn new(queue: Arc<Queue>, subpass: Subpass) -> Self;
    fn draw<'a>(&'a mut self, viewport_dimensions: [u32; 2]) -> SecondaryAutoCommandBuffer;
    fn get_pipeline(&self) -> Arc<GraphicsPipeline>;
}

