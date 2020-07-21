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
    format::ClearValue,
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
    top_left: [f32; 2],
    bottom_right: [f32; 2],
    depth: f32,
    colour: [f32; 4],   
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

    primitives: Vec<Primitive>,
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
            primitives: vec!(),
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

    pub fn queue_primitive(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    pub fn draw_primitives<'a>(&'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder,
        image_num: usize,
    ) -> bool {
        
        let dimensions = self.framebuffers[image_num].dimensions();
        
        // process the Primitives to vertices and indices...
        let verts = vec!();
        for prim in self.primitives {
            verts.append([
                prim.top_left[0], prim.top_left[1],
                prim.bottom_right[0], prim.bottom_right[1],
            ]);
        }

        builder
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                false,
                vec![ClearValue::None],
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

        true
    }

}
