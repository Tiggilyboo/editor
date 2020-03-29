use std::sync::Arc;
use std::collections::HashSet;

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    PhysicalDevice,
};
use vulkano::device::{
    Device,
    DeviceExtensions,
    Queue,
    Features,
};
use vulkano::swapchain::{
    Surface,
    Capabilities,
    ColorSpace,
    SupportedPresentModes,
    PresentMode,
    Swapchain,
    CompositeAlpha,
    FullscreenExclusive,
};
use vulkano::format::Format;
use vulkano::image::{
    ImageUsage,
    swapchain::SwapchainImage,
};
use vulkano::sync::SharingMode;
use vulkano_win::VkSurfaceBuild;

use winit::event_loop::{ EventLoop ,ControlFlow };
use winit::event::{ Event, WindowEvent };
use winit::window::{ WindowBuilder, Window };
use winit::dpi::LogicalSize;

fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        .. vulkano::device::DeviceExtensions::none()
    }
}

pub struct EditorApplication {
    instance: Arc<Instance>,
    events_loop: EventLoop<()>,
    surface: Arc<Surface<Window>>,

    physical_device_id: usize,
    device: Arc<Device>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
}

impl EditorApplication {
    pub fn new(title: &str) -> Self {
        let _instance = Self::create_instance(); 
        let (_events_loop, _surface) = Self::create_surface(title, &_instance);
        
        let _physical_device_id = Self::find_physical_device(&_instance, &_surface);
        let (_device, graphics_queue, present_queue) = Self::create_logical_device(&_instance, &_surface, _physical_device_id);

        let (swap_chain, swap_chain_images) = Self::create_swap_chain(
            &_instance, &_surface, _physical_device_id, 
            &_device, &graphics_queue, &present_queue);
    
        Self {
            events_loop: _events_loop,
            surface: _surface,
            instance: _instance,
            physical_device_id: _physical_device_id,
            device: _device, 
            graphics_queue: graphics_queue,
            present_queue: present_queue, 

            swap_chain: swap_chain,
            swap_chain_images,
        }
    }

    pub fn run_events(self) -> ! {
        self.events_loop.run(move |event, _, control_flow| {

            *control_flow = ControlFlow::Poll;
            
            match event {
                Event::UserEvent(event) => println!("user event: {:?}", event),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }

    fn create_instance() -> Arc<Instance> {
        let supported_exts = InstanceExtensions::supported_by_core()
            .expect("unable to retrive supported extensions");

        println!("Supported Extensions: {:?}", supported_exts);

        let app_info = vulkano::app_info_from_cargo_toml!();
        let req_extensions = vulkano_win::required_extensions();

        return Instance::new(Some(&app_info), &req_extensions, None) 
            .expect("unable to create new instance");
    }

    fn is_device_suitable(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> bool {
        let indices = Self::find_queue_families(surface, device);
        let extensions_supported = Self::check_device_extensions_supported(device);

        let swap_chain_adequate = if extensions_supported {
            let caps = surface.capabilities(*device)
                .expect("failed to get surface capabilities");

            println!("Suppored formats: {:?}, Present Modes: {:?}", caps.supported_formats, caps.present_modes);

            !caps.supported_formats.is_empty() &&
                caps.present_modes.iter().next().is_some()
        } else {
            false
        };

        println!("{:?}: complete: {:?}, supported: {:?}, adequate: {:?}", device.name(), indices.is_complete(), extensions_supported, swap_chain_adequate);

        indices.is_complete() && extensions_supported && swap_chain_adequate
    }

    fn check_device_extensions_supported(device: &PhysicalDevice) -> bool {
        let avail_ext = DeviceExtensions::supported_by_device(*device);
    
        avail_ext.khr_swapchain
    }

    fn find_queue_families(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> QueueFamilyIndices {
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

    fn find_physical_device(instance: &Arc<Instance>, surface: &Arc<Surface<Window>>) -> usize {
        return PhysicalDevice::enumerate(instance)
            .position(|device| Self::is_device_suitable(surface, &device))
            .expect("failed to find suitable physical device");
    }

    fn create_logical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_id: usize,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        use std::iter::FromIterator;

        let physical_device = PhysicalDevice::from_index(&instance, physical_device_id).unwrap();
        let indices = Self::find_queue_families(surface, &physical_device);

        let families = [indices.graphics_family, indices.present_family];
        let unique_queue_families: HashSet<&i32> = HashSet::from_iter(families.iter());

        let queue_families = unique_queue_families.iter().map(|i| {
            (physical_device.queue_families().nth(**i as usize).unwrap(), 1.0)
        });
        let (_device, mut queues) = Device::new(physical_device, &Features::none(), &device_extensions(), queue_families)
            .expect("failed to create logical device");

        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next()
            .unwrap_or_else(|| graphics_queue.clone());

        (_device, graphics_queue, present_queue) 
    }

    fn create_surface(title: &str, instance: &Arc<Instance>) -> (EventLoop<()>, Arc<Surface<Window>>) {
        let _events_loop = EventLoop::new();
        let _surface = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(800.0, 600.0))
            .build_vk_surface(&_events_loop, instance.clone())
            .expect("Unable to create window with events loop");
        
        (_events_loop, _surface)
    }

    fn choose_swap_surface_format(avail_formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
        *avail_formats.iter()
            .find(|(format, color_space)|
                  *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
            ).unwrap_or_else(|| &avail_formats[0])
    }

    fn choose_swap_present_mode(avail_modes: SupportedPresentModes) -> PresentMode {
        if avail_modes.mailbox {
            PresentMode::Mailbox
        } else if avail_modes.immediate {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        }
    }

    fn choose_swap_extent(caps: &Capabilities) -> [u32; 2] {
        if let Some(current_extent) = caps.current_extent {
            return current_extent
        }
        let mut actual_extent = [800, 600];
        actual_extent[0] = caps.min_image_extent[0]
            .max(caps.max_image_extent[0].min(actual_extent[0]));
        actual_extent[1] = caps.min_image_extent[1]
            .max(caps.max_image_extent[1].min(actual_extent[1]));

        actual_extent
    }

    fn create_swap_chain(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_id: usize,
        device: &Arc<Device>,
        graphics_queue: &Arc<Queue>,
        present_queue: &Arc<Queue>,
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let physical_device = PhysicalDevice::from_index(&instance, physical_device_id).unwrap();
        let caps = surface.capabilities(physical_device)
            .expect("failed to get surface capabilities");

        let surface_format = Self::choose_swap_surface_format(&caps.supported_formats);
        let present_mode = Self::choose_swap_present_mode(caps.present_modes);
        let extent = Self::choose_swap_extent(&caps);

        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count.is_some() && image_count > caps.max_image_count.unwrap() {
            image_count = caps.max_image_count.unwrap();
        }
        
        let image_usage = ImageUsage {
            color_attachment: true,
            .. ImageUsage::none()
        };

        let indices = Self::find_queue_families(&surface, &physical_device);
        let sharing: SharingMode = if indices.graphics_family != indices.present_family {
            vec![graphics_queue, present_queue].as_slice().into()
        } else {
            graphics_queue.into()
        };

        let (swap_chain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            image_count,
            surface_format.0,
            extent,
            1, // Layers
            image_usage,
            sharing,
            caps.current_transform,
            CompositeAlpha::Opaque,
            present_mode,
            FullscreenExclusive::Allowed,
            true, // Clipped
            surface_format.1,
        ).expect("failed to create swap chain");

        (swap_chain, images)
    }
}

struct QueueFamilyIndices {
  graphics_family: i32,
  present_family: i32,
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
}


