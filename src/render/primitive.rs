mod shaders;

use std::sync::Arc;
use std::iter;
use std::collections::HashMap;

use vulkano::{
    device::{
        Device,
        Queue,
    },
    pipeline::{
        GraphicsPipeline,
        viewport::Viewport,
        vertex::SingleBufferDefinition,
    },
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
        ImmutableBuffer,
        TypedBufferAccess,
    },
    command_buffer::{
        AutoCommandBufferBuilder,
        DynamicState,
    },
    descriptor::pipeline_layout::PipelineLayoutAbstract,
    framebuffer::{
        RenderPassAbstract,
        Framebuffer,
        FramebufferAbstract,
        Subpass,
    },
    swapchain::Swapchain,
    image::SwapchainImage,
};
use shaders::{
    Vertex,
    vertex_shader,
    fragment_shader,
};

pub struct Primitive {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
    pub depth: f32,
    pub colour: [f32; 4],   
}

pub struct PrimitiveContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, 
        Box<dyn PipelineLayoutAbstract + Send + Sync>, 
        Arc<dyn RenderPassAbstract + Send + Sync>>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    
    vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    index_buffer: Option<Arc<dyn TypedBufferAccess<Content=[u16]> + Send + Sync>>,

    primitives: HashMap<usize, Primitive>,
}

impl PrimitiveContext {
    pub fn new<W>(
        device: Arc<Device>,
        queue: Arc<Queue>,
        swapchain: Arc<Swapchain<W>>,
        images: &[Arc<SwapchainImage<W>>]
    ) -> Self where W: Send + Sync + 'static {
        
        let vertex_shader = vertex_shader::Shader::load(device.clone())
            .expect("unable to load primitive vertex shader");

        let fragment_shader = fragment_shader::Shader::load(device.clone())
            .expect("unable to load primitive fragment shader");

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
            ).unwrap()) as Arc<dyn RenderPassAbstract + Send + Sync>;

        let framebuffers = images.iter().map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .triangle_list()
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                depth_range: 0.0..1.0,
                dimensions: [
                    images[0].dimensions()[0] as f32,
                    images[0].dimensions()[1] as f32
                ],
            }))
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .expect("Unable to create primitive pipeline")
        );

        PrimitiveContext {
            device: device.clone(),
            queue,
            pipeline,
            framebuffers,
            vertex_buffer: None,
            index_buffer: None,
            primitives: HashMap::new(),
        }
    }

    fn upload_vertices(&mut self, vertices: Vec<Vertex>, indices: Vec<u16>) {
        
        self.vertex_buffer = Some(CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            vertices.into_iter()
        ).expect("unable to create primitive vertex buffer"));

        let (index_buffer, _future) = ImmutableBuffer::from_iter(
            indices.into_iter(),
            BufferUsage::index_buffer(),
            self.queue.clone(),
        ).expect("unable to create primitive index buffer");

        self.index_buffer = Some(index_buffer);
    }

    pub fn queue_primitive(&mut self, index: usize, primitive: Primitive) {
        self.primitives.insert(index, primitive);
    }

    #[inline]
    fn primitive_to_buffer(offset: u16, primitive: &Primitive, dimensions: [f32; 2]) -> ([Vertex; 4], [u16; 6]) {
        let x = 2.0 / dimensions[0];
        let y = 2.0 / dimensions[1];
        let w = dimensions[0] / 2.0;
        let h = dimensions[1] / 2.0;
        let offset = offset * 3;

        let mut top_left = [primitive.top_left[0] - w, primitive.top_left[1] - h];
        let mut bottom_right = [primitive.bottom_right[0] - w, primitive.bottom_right[1] - h];

        top_left[0] *= x;
        bottom_right[0] *= x;

        top_left[1] *= y;
        bottom_right[1] *= y;
    
        ([
            Vertex {
                position: [top_left[0], top_left[1], primitive.depth],
                colour: primitive.colour,
            },
            Vertex {
                position: [bottom_right[0], top_left[1], primitive.depth],
                colour: primitive.colour,
            },
            Vertex {
                position: [top_left[0], bottom_right[1], primitive.depth],
                colour: primitive.colour,
            },
            Vertex {
                position: [bottom_right[0], bottom_right[1], primitive.depth],
                colour: primitive.colour,
            },
        ],
            [
                // 0 1 4 5
                // 2 3 6 7
                offset + 0, offset + 1, offset + 2, 
                offset + 1, offset + 2, offset + 3, 
            ]
        )
    }

    pub fn draw_primitives<'a>(&'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder,
        image_num: usize,
    ) -> bool {
        let dimensions = self.framebuffers[image_num].dimensions(); 
        let dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        // process the Primitives to vertices and indices...
        let len_prims = self.primitives.len();
        let mut verts: Vec<Vertex> = Vec::with_capacity(len_prims * 4);
        let mut indices: Vec<u16> = Vec::with_capacity(len_prims * 6);
        let mut i: u16 = 0;
        for (_, prim) in self.primitives.iter() {
            let (prim_verts, prim_indices) = Self::primitive_to_buffer(i, prim, dimensions);
        
            verts.extend(prim_verts.iter());
            indices.extend(prim_indices.iter());
            i += 1;
        }

        // Do we actuall have anything to draw?
        if i > 0 {
            self.upload_vertices(verts, indices);

            builder
                .begin_render_pass(
                    self.framebuffers[image_num].clone(),
                    false,
                    vec![[0.0, 0.0, 0.0, 1.0].into()],
                ).expect("unable to begin primitive render pass")

                .draw_indexed(
                    self.pipeline.clone(),
                    &DynamicState::none(),
                    self.vertex_buffer.clone().unwrap(),
                    self.index_buffer.clone().unwrap(),
                    (),
                    ()
                ).expect("unable to draw to command buffer for primitive")

                .end_render_pass()
                .expect("unable to end primitive render pass");
        }

        true
    }

}
