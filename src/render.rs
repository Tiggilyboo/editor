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
use vulkano::command_buffer::{
    AutoCommandBuffer,
    AutoCommandBufferBuilder,
    DynamicState,
};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::{
    Device,
    DeviceExtensions,
    Queue,
    Features,
};
use vulkano::framebuffer::{
    RenderPassAbstract,
    Subpass,
    FramebufferAbstract,
    Framebuffer,
};
use vulkano::swapchain::{
    Surface,
    Capabilities,
    ColorSpace,
    SupportedPresentModes,
    SwapchainCreationError,
    PresentMode,
    Swapchain,
    CompositeAlpha,
    FullscreenExclusive,
    acquire_next_image,
    AcquireError,
};
use vulkano::format::Format;
use vulkano::image::{
    ImageUsage,
    swapchain::SwapchainImage,
};
use vulkano::pipeline::{
    GraphicsPipeline,
    vertex::BufferlessDefinition,
    vertex::BufferlessVertices,
    viewport::Viewport,
};
use vulkano::sync::{
    self,
    SharingMode,
    GpuFuture,
};
use vulkano_win::VkSurfaceBuild;

use winit::event_loop::{ EventLoop ,ControlFlow };
use winit::platform::desktop::EventLoopExtDesktop;
use winit::event::{ Event, WindowEvent };
use winit::window::{ WindowBuilder, Window };
use winit::dpi::LogicalSize;

type ConcreteGraphicsPipeline = 
    GraphicsPipeline<BufferlessDefinition, Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>, 
        Arc<dyn RenderPassAbstract + Send + Sync + 'static>>;

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

pub struct EditorApplication {
    instance: Arc<Instance>,
    debug_callback: Option<DebugCallback>,
    events_loop: EventLoop<()>,
    surface: Arc<Surface<Window>>,

    physical_device_id: usize,
    device: Arc<Device>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,

    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: DynamicState,
    graphics_pipeline: Arc<ConcreteGraphicsPipeline>,

    swap_chain_frame_buffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    command_buffers: Vec<Arc<AutoCommandBuffer>>,

    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swap_chain: bool,
}

impl EditorApplication {
    pub fn new(title: &str) -> Self {
        let _instance = Self::create_instance(); 
        let debug_callback = Self::create_debug_callback(&_instance);
        let (_events_loop, _surface) = Self::create_surface(title, &_instance);
        
        let _physical_device_id = Self::find_physical_device(&_instance, &_surface);
        let (_device, graphics_queue, present_queue) = Self::create_logical_device(&_instance, &_surface, _physical_device_id);

        let (swap_chain, swap_chain_images) = Self::create_swap_chain(
            &_instance, &_surface, _physical_device_id, 
            &_device, &graphics_queue, &present_queue, None);
    
        let render_pass = Self::create_render_pass(&_device, swap_chain.format());
        let graphics_pipeline = Self::create_graphics_pipeline(&_device, swap_chain.dimensions(), &render_pass);
        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };
        let swap_chain_frame_buffers = Self::create_framebuffers(&swap_chain_images, &render_pass, &mut dynamic_state);

        let previous_frame_end = Some(Self::create_sync_objects(&_device));

        let mut app = Self {
            events_loop: _events_loop,
            surface: _surface,
            instance: _instance,
            debug_callback: debug_callback,

            physical_device_id: _physical_device_id,
            device: _device, 
            graphics_queue: graphics_queue,
            present_queue: present_queue, 

            swap_chain: swap_chain,
            swap_chain_images,
            dynamic_state,

            render_pass: render_pass,
            graphics_pipeline,

            swap_chain_frame_buffers,
            command_buffers: vec![],

            previous_frame_end,
            recreate_swap_chain: false,
        };

        app.create_command_buffers();
        app
    }

    pub fn run(&mut self) {

        let mut done = false;
        let mut recreate = false;

        while !done {
            self.events_loop.run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                
                match event {
                    Event::UserEvent(event) => println!("user event: {:?}", event),
                    Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                        done = true;
                    },
                    Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                        recreate = true;
                    }
                    Event::MainEventsCleared => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            });

            if recreate {
                self.recreate_swap_chain = recreate;
            }
            self.draw_frame();
        }
    }

    fn create_sync_objects(device: &Arc<Device>) -> Box<dyn GpuFuture> {
        Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>
    }

    fn draw_frame(&mut self) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swap_chain {
            self.recreate_swap_chain();
            self.recreate_swap_chain = false;
        }

        let (image_index, suboptimal, acquire_future) = match acquire_next_image(self.swap_chain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                println!("Setting recreate swap chain to true");
                self.recreate_swap_chain = true;
                return
            },
            Err(err) => panic!("Failed to acquire next image: {:?}", err)
        };

        if suboptimal {
            self.recreate_swap_chain = true;
            return
        }

        if image_index > self.command_buffers.len() - 1 {
            println!("Bad image index: {:?}", image_index);
            return;
        }   
        let command_buffer = self.command_buffers[image_index].clone();

        let future = self.previous_frame_end.take().unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.present_queue.clone(), self.swap_chain.clone(), image_index)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                self.recreate_swap_chain = true;
                self.previous_frame_end = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
        }
    }

    fn recreate_swap_chain(&mut self) {
        let dimensions: [u32; 2] = self.surface.window().inner_size().into();
        let (new_swapchain, new_images) = match self.swap_chain.recreate_with_dimensions(dimensions) {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => return,
            Err(err) => panic!("Failed to recreate swapchain: {:?}", err)
        };

        self.swap_chain = new_swapchain;
        self.swap_chain_images = new_images;
        self.swap_chain_frame_buffers = Self::create_framebuffers(&self.swap_chain_images, &self.render_pass, &mut self.dynamic_state);
        self.create_command_buffers();
    }

    fn get_required_extensions() -> InstanceExtensions {
        let mut extensions = vulkano_win::required_extensions();

        if ENABLE_VALIDATION_LAYERS {
            extensions.ext_debug_utils = true;
        }

        extensions
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list().unwrap()
            .map(|l| l.name().to_owned())
            .collect();

        VALIDATION_LAYERS.iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
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
            .with_resizable(true)
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

        let indices = Self::find_queue_families(&surface, &physical_device);
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

    fn create_render_pass(device: &Arc<Device>, color_fmt: Format) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_fmt,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap())
    }

    fn create_graphics_pipeline(
        device: &Arc<Device>, 
        swap_chain_extent: [u32; 2],
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> Arc<ConcreteGraphicsPipeline> {
        mod vertex_shader {
            vulkano_shaders::shader! {
                ty: "vertex",
                path: "shaders/shader.vert"
            }
        }
        mod fragment_shader {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: "shaders/shader.frag"
            }
        }

        let _vert_shader_mod = vertex_shader::Shader::load(device.clone())
            .expect("failed to create vertex shader module");
        let _frag_shader_mod = fragment_shader::Shader::load(device.clone())
            .expect("failed to create fragment shader module");
       
        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0 .. 1.0,
        };

        Arc::new(GraphicsPipeline::start()
            .vertex_input(BufferlessDefinition {})
            .vertex_shader(_vert_shader_mod.main_entry_point(), ())
            .triangle_list()
            .primitive_restart(false)
            .viewports(vec![viewport])
            .fragment_shader(_frag_shader_mod.main_entry_point(), ())
            .depth_clamp(false)
            .polygon_mode_fill()
            .line_width(1.0)
            .cull_mode_back()
            .front_face_clockwise()
            .blend_pass_through()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
        )
    }

    fn create_framebuffers(
        swap_chain_images: &[Arc<SwapchainImage<Window>>],
        render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = swap_chain_images[0].dimensions();
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        swap_chain_images.iter()
            .map(|image| {
                let fba: Arc<dyn FramebufferAbstract + Send + Sync> = 
                    Arc::new(Framebuffer::start(render_pass.clone())
                        .add(image.clone()).unwrap()
                        .build().unwrap());
                fba
            }).collect::<Vec<_>>()
    }

    fn create_command_buffers(&mut self) {
        let q_family = self.graphics_queue.family();

        self.command_buffers = self.swap_chain_frame_buffers.iter()
            .map(|framebuffer| {
                let verts = BufferlessVertices { vertices: 3, instances: 1 };

                Arc::new(
                    AutoCommandBufferBuilder::primary_simultaneous_use(self.device.clone(), q_family)
                         .unwrap()
                         .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
                         .unwrap()
                         .draw(self.graphics_pipeline.clone(), &DynamicState::none(), verts, (), ())
                         .unwrap()
                         .end_render_pass()
                         .unwrap()
                         .build()
                         .unwrap()
                )
            }).collect();
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

