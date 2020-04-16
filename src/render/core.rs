use std::sync::Arc;

use std::collections::HashSet;
use vulkano::instance::{
    Instance,
    InstanceExtensions,
    PhysicalDevice,
    layers_list,
    debug::DebugCallback,
    debug::MessageType,
    debug::MessageSeverity,
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
    SwapchainAcquireFuture,
    acquire_next_image,
    AcquireError,

};
use vulkano::format::Format;
use vulkano::framebuffer::{
    RenderPassAbstract,
    FramebufferAbstract,
    Framebuffer,
};
use vulkano::image::{
    ImageUsage,
    swapchain::SwapchainImage,
};
use vulkano::sync::{
    NowFuture,
    SharingMode,
};
use vulkano::command_buffer::{
    DynamicState,
};
use vulkano::pipeline::viewport::Viewport;

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::{ EventLoop };

use winit::window::{ WindowBuilder, Window };
use winit::dpi::LogicalSize;

mod queue_indices;
use queue_indices::QueueFamilyIndices;

const VALIDATION_LAYERS: &[&str] = &[
    "VK_LAYER_LUNARG_standard_validation"
];
#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        .. vulkano::device::DeviceExtensions::none()
    }
}

pub struct RenderCore {
    instance: Arc<Instance>,
    events_loop: EventLoop<()>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
   
    // TODO This is dirty, need to pull out bits from render still in recreate_swap_chain
    
    pub swap_chain: Arc<Swapchain<Window>>,
    pub swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    
    debug_callback: Option<DebugCallback>,
}

impl RenderCore {
    pub fn new(title: &str) -> Self {
        let instance = Self::create_instance(); 
        let debug_callback = Self::create_debug_callback(&instance);
        let (events_loop, surface) = Self::create_surface(title, &instance);

        let _physical_device_id = Self::find_physical_device(&instance, &surface);
        let (device, graphics_queue, present_queue) = Self::create_logical_device(&instance, &surface, _physical_device_id);

        let (swap_chain, swap_chain_images) = Self::create_swap_chain(
            &instance, &surface, _physical_device_id, 
            &device, &graphics_queue, &present_queue, None);
    
        Self {
            instance,
            debug_callback,
            events_loop,
            surface,
            device,
            graphics_queue,
            present_queue,
            swap_chain,
            swap_chain_images,
        }
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list().unwrap()
            .map(|l| l.name().to_owned())
            .collect();

        VALIDATION_LAYERS.iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
    }
    
    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = vulkano_win::required_extensions();

        if ENABLE_VALIDATION_LAYERS {
            extensions.ext_debug_utils = true;
        }

        extensions
    }

    fn create_instance() -> Arc<Instance> {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
            println!("Validation layers enabled, but not supported");
        }

        let supported_exts = InstanceExtensions::supported_by_core()
            .expect("unable to retrieve supported extensions");

        println!("Supported Extensions: {:?}", supported_exts);

        let app_info = vulkano::app_info_from_cargo_toml!();
        let req_extensions = Self::get_required_extensions();

        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(Some(&app_info), &req_extensions, VALIDATION_LAYERS.iter().cloned())
                .expect("unable to create new vulkan instance")
        } else {
           Instance::new(Some(&app_info), &req_extensions, None) 
                .expect("unable to create new vulkan instance")
        }
    }

    fn create_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }
        
        let msg_types = MessageType::general();
        let msg_severity = MessageSeverity::errors();
        
        DebugCallback::new(&instance, msg_severity, msg_types, |msg| {
            println!("validation layers: {:?}", msg.description);
        }).ok() 
    }

    fn is_device_suitable(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> bool {
        let indices = QueueFamilyIndices::find_queue_families(surface, device);
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

    fn find_physical_device(instance: &Arc<Instance>, surface: &Arc<Surface<Window>>) -> usize {
        return PhysicalDevice::enumerate(instance)
            .position(|device| Self::is_device_suitable(surface, &device))
            .expect("failed to find suitable physical device");
    }
    
    fn create_surface(title: &str, instance: &Arc<Instance>) -> (EventLoop<()>, Arc<Surface<Window>>) {
        let _events_loop = EventLoop::new();
        let _surface = WindowBuilder::new()
            .with_title(title)
            .with_resizable(true)
            .with_inner_size(LogicalSize::new(1024.0, 768.0))
            .build_vk_surface(&_events_loop, instance.clone())
            .expect("Unable to create window with events loop");
        
        (_events_loop, _surface)
    }

    fn create_logical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_id: usize,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        use std::iter::FromIterator;

        let physical_device = PhysicalDevice::from_index(&instance, physical_device_id).unwrap();
        let indices = QueueFamilyIndices::find_queue_families(surface, &physical_device);

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
        old_swapchain: Option<Arc<Swapchain<Window>>>,
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

        let indices = QueueFamilyIndices::find_queue_families(&surface, &physical_device);
        let sharing: SharingMode = if indices.graphics_family != indices.present_family {
            vec![graphics_queue, present_queue].as_slice().into()
        } else {
            graphics_queue.into()
        };

        if old_swapchain.is_some() {
            return Swapchain::with_old_swapchain(
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
                old_swapchain.unwrap(),
            ).expect("failed to create swap chain");

        }
        
        return Swapchain::new(
            device.clone(),
            surface.clone(),
            image_count,
            surface_format.0,
            extent,
            1,
            image_usage,
            sharing,
            caps.current_transform,
            CompositeAlpha::Opaque,
            present_mode,
            FullscreenExclusive::Allowed,
            true,
            surface_format.1
        ).expect("failed to create swap chain");
    }

    pub fn create_render_pass(&self, color_fmt: Option<Format>) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        Arc::new(vulkano::single_pass_renderpass!(self.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_fmt.unwrap_or(self.swap_chain.format()),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap())
    }

    pub fn create_framebuffers(
        &self,
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = self.swap_chain_images[0].dimensions();
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        self.swap_chain_images.iter()
            .map(|image| {
                Arc::new(Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .build().unwrap()
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            }).collect::<Vec<_>>()
    }

    pub fn get_next_swap_chain_image(&self) -> Result<(usize, bool, SwapchainAcquireFuture<Window>), AcquireError> {
        acquire_next_image(self.swap_chain.clone(), None)
    }

    pub fn get_device(&self) -> &Arc<Device> {
        &self.device 
    }

    pub fn get_graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }

    pub fn get_present_queue(&self) -> &Arc<Queue> {
        &self.present_queue
    }

    pub fn get_events_loop(&mut self) -> &mut EventLoop<()> {
        &mut self.events_loop
    }

    pub fn get_window(&self) -> &Window {
        self.surface.window()
    }

    pub fn get_previous_frame_end(&self) -> Box<NowFuture> {
        Box::new(vulkano::sync::now(self.device.clone()))
    }
}
