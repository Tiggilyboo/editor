use std::sync::Arc;
use vulkano::instance::PhysicalDevice;
use winit::window::Window;
use vulkano::swapchain::Surface;

pub struct QueueFamilyIndices {
  pub graphics_family: i32,
  pub present_family: i32,
}

impl QueueFamilyIndices {
    pub fn new() -> Self {
        return Self { 
            graphics_family: -1,
            present_family: -1, 
        };
    }
    pub fn is_complete(&self) -> bool {
        return self.graphics_family >= 0 && self.present_family >= 0;
    }

    pub fn find_queue_families(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();

        for (index, q_family) in device.queue_families().enumerate() {
            if q_family.supports_graphics() {
                indices.graphics_family = index as i32;
            }
            if surface.is_supported(q_family).unwrap() {
                indices.present_family = index as i32;
            }
            if indices.is_complete() {
                break;
            }
        }

        return indices;
    }
}

