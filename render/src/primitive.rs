mod shaders;

use std::sync::Arc;

use vulkano::{
    device::Queue,
    descriptor_set::PersistentDescriptorSet,
    buffer::{
        BufferUsage,
        ImmutableBuffer,
        CpuBufferPool,
    },
    command_buffer::{
        SecondaryAutoCommandBuffer,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
    },
    pipeline::{
        graphics::viewport::{
            ViewportState,
            Viewport,
        },
        graphics::vertex_input::BuffersDefinition,
        graphics::input_assembly::InputAssemblyState,
        GraphicsPipeline,
        Pipeline,
        PipelineBindPoint,
    },
    render_pass::Subpass,
};

use self::shaders::Vertex;
use super::uniform::{
    UniformTransform,
    calculate_transform,
};
use super::abstract_renderer::AbstractRenderer;

#[derive(Debug)]
pub struct Primitive {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
    pub depth: f32,
    pub colour: [f32; 4],
}

pub struct PrimitiveRenderer {
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    
    uniform_buffer_pool: CpuBufferPool<UniformTransform>,
    vertex_buffer: Option<Arc<ImmutableBuffer<[Vertex]>>>,
    index_buffer: Option<Arc<ImmutableBuffer<[u16]>>>,
    indices_len: usize,

    descriptor_set: Option<Arc<PersistentDescriptorSet>>,
    dimensions: [f32; 2],

    primitives: Vec<Primitive>,
    primitives_len: usize,
    pristine: bool,
}

impl AbstractRenderer for PrimitiveRenderer {
    fn new(queue: Arc<Queue>, subpass: Subpass) -> Self {
        
        let vertex_shader = shaders::load_vs(queue.device().clone())
            .expect("unable to load primitive vertex shader");

        let fragment_shader = shaders::load_fs(queue.device().clone())
            .expect("unable to load primitive fragment shader");

        let uniform_buffer_pool = CpuBufferPool::new(queue.device().clone(), BufferUsage::uniform_buffer());
 
        let pipeline = GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new()) // triangle_list
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .render_pass(subpass)
            .blend_alpha_blending()
            .build(queue.device().clone())
            .expect("Unable to create primitive pipeline");

        PrimitiveRenderer {
            queue,
            pipeline,
            uniform_buffer_pool,
            vertex_buffer: None,
            index_buffer: None,
            indices_len: 0,
            descriptor_set: None,
            dimensions: [0.0, 0.0],
            primitives: Vec::new(),
            primitives_len: 0,
            pristine: false,
        }
    }
    
    fn draw<'a>(&'a mut self, viewport_dimensions: [u32; 2]) -> SecondaryAutoCommandBuffer {
        self.process();
        self.check_recreate_descriptor_set(viewport_dimensions);

        let pipeline = self.get_pipeline();
        let indices = self.index_buffer.clone().unwrap();
        let vertices = self.vertex_buffer.clone().unwrap();
        let indices_len = self.indices_len as u32;

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            CommandBufferUsage::MultipleSubmit,
            self.pipeline.subpass().clone()
        ).unwrap();

        builder
            .set_viewport(0,
              [Viewport {
                  origin: [0.0, 0.0],
                  dimensions: self.dimensions,
                  depth_range: 0.0..1.0,
              }],
            )
            .bind_pipeline_graphics(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                self.descriptor_set.clone().unwrap(),
            )
            .bind_vertex_buffers(0, vertices.clone())
            .bind_index_buffer(indices.clone())
            .draw_indexed(indices_len, 1, 0, 0, 0)
            .unwrap();

        let buffer = builder.build().unwrap();

        self.primitives.clear();
        self.pristine = true;

        buffer
    }

    fn get_pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipeline.clone()
    }
}

impl PrimitiveRenderer {
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
        self.indices_len = indices.len();

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

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
    }

    fn process(&mut self) {
        let len_prims = self.primitives.len();
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
    }   
    
    fn check_recreate_descriptor_set(&mut self, dimensions: [u32; 2]) {
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
        let layout = pipeline
            .layout()
            .descriptor_set_layouts().get(0)
            .expect("could not retrieve pipeline descriptor set layout 0");

        let mut descriptor_set_builder = PersistentDescriptorSet::start(layout.clone());

        descriptor_set_builder
                .add_buffer(Arc::new(uniform_buffer))
                .expect("could not add uniform buffer to PersistentDescriptorSet binding 0");

        self.descriptor_set = Some(
            descriptor_set_builder
                .build()
                .expect("PrimitiveRenderer: unable to create PersistentDescriptorSet 0"));

        self.dimensions = dimensions;
    }

}
