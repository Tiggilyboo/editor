mod shaders;

use std::sync::Arc;
use std::iter;

use abstract_renderer::AbstractRenderer;

use winit::window::Window;

use vulkano::{
    device::{
        Device,
        Queue,
    },
    pipeline::viewport::Viewport,
    descriptor_set::{
        PersistentDescriptorSet,
        DescriptorSet,
    },
    buffer::{
        BufferUsage,
        ImmutableBuffer,
        CpuBufferPool,
        TypedBufferAccess,
        BufferAccess,
    },
    command_buffer::{
        pool::standard::StandardCommandPoolBuilder,
        PrimaryAutoCommandBuffer,
        AutoCommandBufferBuilder,
        DynamicState,
        SubpassContents,
    },
    pipeline::{
        GraphicsPipeline,
        GraphicsPipelineAbstract,
    },
    render_pass::{
        Framebuffer,
        FramebufferAbstract,
        Subpass,
    },
    swapchain::Swapchain,
    image::{
        SwapchainImage,
        view::ImageView,
    },
};

use self::shaders::{
    Vertex,
    vertex_shader,
    fragment_shader,
};
use uniform::{
    UniformTransform,
    calculate_transform,
};

#[derive(Debug)]
pub struct Primitive {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
    pub depth: f32,
    pub colour: [f32; 4],
}

pub struct PrimitiveContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Option<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    framebuffers: Option<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    
    uniform_buffer_pool: CpuBufferPool<UniformTransform>,
    vertex_buffer: Option<Arc<dyn TypedBufferAccess<Content=[Vertex]> + Send + Sync>>,
    index_buffer: Option<Arc<dyn TypedBufferAccess<Content=[u16]> + Send + Sync>>,

    vertex_shader: vertex_shader::Shader,
    fragment_shader: fragment_shader::Shader,

    descriptor_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,
    dimensions: [f32; 2],

    primitives: Vec<Primitive>,
    primitives_len: usize,
    pristine: bool,
}

impl AbstractRenderer for PrimitiveContext {
    fn get_pipeline(&self) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline.clone().expect("Uninitialised pipeline")
    }

    fn get_framebuffers(&self) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        self.framebuffers.clone()
            .expect("Uninitialised frame buffers")
    }

    fn set_swap_chain(&mut self, swapchain: Arc<Swapchain<Window>>, images: &Vec<Arc<SwapchainImage<Window>>>) {
        let device = &self.device;

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            ).unwrap());

        let framebuffers = images.iter().map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Arc::new(
                Framebuffer::start(render_pass.clone())
                .add(view).unwrap()
                .build().unwrap()
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .triangle_list()
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                depth_range: 0.0..1.0,
                dimensions: [
                    images[0].dimensions()[0] as f32,
                    images[0].dimensions()[1] as f32
                ],
            }))
            .fragment_shader(self.fragment_shader.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .expect("Unable to create primitive pipeline")
        );

        self.pipeline = Some(pipeline);
        self.framebuffers = Some(framebuffers);
    }
}

impl PrimitiveContext {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {

        let vertex_shader = vertex_shader::Shader::load(device.clone())
            .expect("unable to load primitive vertex shader");

        let fragment_shader = fragment_shader::Shader::load(device.clone())
            .expect("unable to load primitive fragment shader");

        let uniform_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());
 
        PrimitiveContext {
            device: device.clone(),
            queue,
            fragment_shader,
            vertex_shader,
            pipeline: None,
            framebuffers: None,
            uniform_buffer_pool,
            primitives: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            descriptor_set: None,
            dimensions: [0.0, 0.0],
            primitives_len: 0,
            pristine: false,
        }
    }

    pub fn queue_primitive(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
        self.pristine = false;
    }

    #[inline]
    fn primitive_to_buffer(offset: u16, primitive: &Primitive) -> ([Vertex; 4], [u16; 6]) {
        let offset = offset * 4;
        let top_left = primitive.top_left;
        let bottom_right = primitive.bottom_right;
        let depth = primitive.depth;
        let colour = primitive.colour;

        ([
            Vertex {
                position: [top_left[0], top_left[1], depth],
                colour,
            },
            Vertex {
                position: [bottom_right[0], top_left[1], depth],
                colour,
            },
            Vertex {
                position: [bottom_right[0], bottom_right[1], depth],
                colour,
            },
            Vertex {
                position: [top_left[0], bottom_right[1], depth],
                colour,
            },
        ],
            [
                offset + 0, offset + 1, offset + 2, 
                offset + 2, offset + 3, offset + 0, 
            ]
        )
    }

    #[inline]
    fn upload_buffers(&mut self, verts: Vec<Vertex>, indices: Vec<u16>) {
        let (vertex_buffer, _future) = ImmutableBuffer::from_iter(
            verts.into_iter(),
            BufferUsage::vertex_buffer(),
            self.queue.clone(),
        ).expect("unable to create primitive vertex buffer");

        let (index_buffer, _future) = ImmutableBuffer::from_iter(
            indices.into_iter(),
            BufferUsage::index_buffer(),
            self.queue.clone(),
        ).expect("unable to create primitive index buffer");

        self.vertex_buffer = Some(Arc::new(vertex_buffer));
        self.index_buffer = Some(Arc::new(index_buffer));
    }

    fn draw_internal<'a>(&'a mut self,
        builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>,
        image_num: usize,
    ) {
        self.check_recreate_descriptor_set(image_num);
    
        if self.vertex_buffer.is_none()
        || self.index_buffer.is_none() 
        || self.descriptor_set.is_none() {
            return;
        }

        let framebuffers = self.get_framebuffers();

        builder
            .begin_render_pass(
                framebuffers[image_num].clone(),
                SubpassContents::Inline,
                vec![[0.0, 0.0, 0.0, 1.0].into()],
            ).expect("unable to begin primitive render pass");

        let vbuf: Arc<dyn BufferAccess + Send + Sync> = Arc::new(self.vertex_buffer.clone().unwrap());

        let pipeline = self.get_pipeline();

        builder.draw_indexed(
            pipeline.clone(),
            &DynamicState::none(),
            vec![vbuf],
            self.index_buffer.clone().unwrap(),
            self.descriptor_set.clone().unwrap(),
            (),
        ).expect("unable to draw to command buffer for primitive");

        builder
            .end_render_pass()
            .expect("unable to end primitive render pass");
    }

    fn check_recreate_descriptor_set(&mut self, image_num: usize) {
        let fbs = self.get_framebuffers();
        let dimensions = fbs[image_num].dimensions(); 
        let dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        if self.dimensions[0] == dimensions[0]
        && self.dimensions[1] == dimensions[1] {
            return; 
        }

        let transform = calculate_transform(0.0, dimensions[0], 0.0, dimensions[1], 1.0, -1.0);
        let uniform_buffer = {
            self.uniform_buffer_pool.next(transform).unwrap()
        };
        let pipeline = self.get_pipeline();
        let layout = pipeline.layout().descriptor_set_layouts().get(0)
            .expect("could not retrieve pipeline descriptor set layout 0");

        self.descriptor_set = Some(
            Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                .add_buffer(uniform_buffer)
                .expect("could not add uniform buffer to PersistentDescriptorSet binding 0")
                .build()
                .expect("PrimitiveContext: unable to create PersistentDescriptorSet 0")
        ));

        self.dimensions = dimensions;
    }

    pub fn draw_primitives<'a>(&'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>,
        image_num: usize,
    ) -> bool {
        if self.pristine {
            self.draw_internal(builder, image_num);
            return true;
        }

        let len_prims = self.primitives.len();
        if len_prims == 0 {
            return true;
        }
        self.primitives_len = len_prims;

        let mut verts: Vec<Vertex> = Vec::with_capacity(len_prims * 4);
        let mut indices: Vec<u16> = Vec::with_capacity(len_prims * 6);
        let mut i: u16 = 0;

        // process the Primitives to vertices and indices...
        for prim in self.primitives.iter() {
            let (prim_verts, prim_indices) = Self::primitive_to_buffer(i, prim);

            verts.extend(prim_verts.iter());
            indices.extend(prim_indices.iter());
            i += 1;
        }

        self.upload_buffers(verts, indices); 

        self.draw_internal(builder, image_num);

        self.primitives.clear();
        self.pristine = true;

        true
    }

}
