mod queue_indices;

use std::sync::Arc;

use std::collections::HashSet;
use vulkano::instance::{
    Instance,
    InstanceExtensions,
    layers_list,
};
use vulkano::device::{
    physical::PhysicalDevice,
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
use vulkano::render_pass::{
    RenderPass,
    FramebufferAbstract,
    Framebuffer,
};
use vulkano::image::{
    ImageUsage,
    view::ImageView,
    swapchain::SwapchainImage,
};
use vulkano::sync::{
    NowFuture,
    SharingMode,
};
use vulkano::command_buffer::{
    DynamicState,
    PrimaryAutoCommandBuffer,
    AutoCommandBufferBuilder,
    pool::standard::StandardCommandPoolBuilder,
};
use vulkano::pipeline::viewport::Viewport;
use vulkano::Version;

use vulkano_win::VkSurfaceBuild;

use winit::window::{ WindowBuilder, Window };
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;

use self::queue_indices::QueueFamilyIndices;

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
    _instance: Arc<Instance>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
   
    // TODO This is dirty, need to pull out bits from render still in recreate_swap_chain
    
    pub swap_chain: Arc<Swapchain<Window>>,
    pub swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
}


impl RenderCore {
    pub fn new<L>(events_loop: &EventLoop<L>, title: &str) -> Self {
        let instance = Self::create_instance(); 
        let surface = Self::create_surface(title, events_loop, &instance);

        let _physical_device_id = Self::find_physical_device(&instance, &surface);
        let (device, graphics_queue, present_queue) = Self::create_logical_device(
            &instance, &surface, _physical_device_id);

        let (swap_chain, swap_chain_images) = Self::create_swap_chain(
            &instance, &surface, _physical_device_id, 
            &device, &graphics_queue, &present_queue, None);
    
        Self {
            _instance: instance,
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

        let app_info = vulkano::app_info_from_cargo_toml!();
        let req_extensions = Self::get_required_extensions();

        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(Some(&app_info), Version::V1_1, &req_extensions, VALIDATION_LAYERS.iter().cloned())
                .expect("unable to create new vulkan instance")
        } else {
           Instance::new(Some(&app_info), Version::V1_1, &req_extensions, None) 
                .expect("unable to create new vulkan instance")
        }
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

        println!("{:?}: complete: {:?}, supported: {:?}, adequate: {:?}", device.properties().device_name, indices.is_complete(), extensions_supported, swap_chain_adequate);

        indices.is_complete() && extensions_supported && swap_chain_adequate
    }

    fn check_device_extensions_supported(device: &PhysicalDevice) -> bool {
        let avail_ext = device.supported_extensions();
    
        avail_ext.khr_swapchain
    }

    fn find_physical_device(instance: &Arc<Instance>, surface: &Arc<Surface<Window>>) -> usize {
        return PhysicalDevice::enumerate(instance)
            .position(|device| Self::is_device_suitable(surface, &device))
            .expect("failed to find suitable physical device");
    }
    
    fn create_surface<L>(title: &str, events_loop: &EventLoop<L>, instance: &Arc<Instance>) -> Arc<Surface<Window>> {
        WindowBuilder::new()
            .with_title(title)
            .with_resizable(true)
            .with_inner_size(LogicalSize::new(1024.0, 768.0))
            .build_vk_surface(events_loop, instance.clone())
            .expect("Unable to create window with events loop")
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
        let features = Features {
            .. Features::none()
        };
        let (_device, mut queues) = Device::new(physical_device, &features, &device_extensions(), queue_families)
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

        if let Some(swapchain) = old_swapchain {
            return swapchain
                .recreate()
                .dimensions(extent)
                .build().unwrap();
        }
        
        return Swapchain::start(device.clone(), surface.clone())
            .num_images(image_count)
            .format(surface_format.0)
            .color_space(surface_format.1)
            .usage(image_usage)
            .sharing_mode(sharing)
            .transform(caps.current_transform)
            .composite_alpha(CompositeAlpha::Opaque)
            .present_mode(present_mode)
            .fullscreen_exclusive(FullscreenExclusive::Allowed)
            .dimensions(extent)
            .clipped(true)
            .build()
        .expect("failed to create swap chain");
    }

    pub fn create_render_pass(&self, color_fmt: Option<Format>) -> Arc<RenderPass> {
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
        render_pass: &Arc<RenderPass>,
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
                let view = ImageView::new(image.clone()).unwrap();
                Arc::new(Framebuffer::start(render_pass.clone())
                    .add(view).unwrap()
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

    pub fn get_window(&self) -> &Window {
        self.surface.window()
    }

    pub fn get_previous_frame_end(&self) -> Box<NowFuture> {
        Box::new(vulkano::sync::now(self.device.clone()))
    }
}
