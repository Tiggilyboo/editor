mod shaders;
use shaders::{
    TextVertex,
    TextTransform,
    vertex_shader,
    fragment_shader,
};

use std::sync::Arc;
use std::iter;

use vulkano::device::{
    Device,
    Queue,
}; 
use vulkano::format::{
    R8Unorm,
    ClearValue,
};
use vulkano::pipeline::{
    GraphicsPipeline,
    viewport::Viewport,
    vertex::SingleBufferDefinition,
};
use vulkano::descriptor::descriptor_set::{
    PersistentDescriptorSet,  
};
use vulkano::descriptor::pipeline_layout::{
    PipelineLayoutAbstract,
};
use vulkano::buffer::{
    BufferUsage,
    CpuAccessibleBuffer,
    CpuBufferPool,
};
use vulkano::swapchain::Swapchain;
use vulkano::image::{
    SwapchainImage,
    ImmutableImage,
    ImageLayout,
    ImageUsage,
    Dimensions,
};
use vulkano::framebuffer::{
    FramebufferAbstract, 
    Framebuffer,
    RenderPassAbstract,
    Subpass,
};
use vulkano::sampler::{
    Sampler,
    Filter,
    MipmapMode,
    SamplerAddressMode,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    DynamicState,
};  
use ab_glyph::{
    *
};
use glyph_brush::{ * };

#[derive(Debug)]
struct TextData {
    section: OwnedSection,
    colour: [f32; 4],
}

pub struct TextContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    glyph_brush: GlyphBrush<TextVertex>,
    pub cache_pixel_buffer: Vec<u8>,
    pub cache_dimensions: (usize, usize),
    pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<TextVertex>, 
        Box<dyn PipelineLayoutAbstract + Send + Sync>, 
        Arc<dyn RenderPassAbstract + Send + Sync>>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    texts: Vec<TextData>,
    uniform_buffer_pool: CpuBufferPool<TextTransform>,
}

#[inline]
pub fn into_vertex(GlyphVertex {
    mut tex_coords,
    pixel_coords,
    bounds,
    extra,
}: GlyphVertex) -> TextVertex {
    let mut rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y),
    };

    // handle overlapping bounds, preserve texture aspect
    if rect.max.x > bounds.max.x {
        let old_w = rect.width();
        rect.max.x = bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * rect.width() / old_w;
    }
    if rect.min.x < bounds.min.x {
        let old_w = rect.width();
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * rect.width() / old_w;
    }
    if rect.max.y > bounds.max.y {
        let old_h = rect.height();
        rect.max.y = bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * rect.height() / old_h;
    }
    if rect.min.y < bounds.min.y {
        let old_h = rect.width();
        rect.min.y = bounds.min.y;
        tex_coords.max.y = tex_coords.max.y - tex_coords.height() * rect.height() / old_h;
    }

    TextVertex {
        left_top: [rect.min.x, rect.max.y, extra.z],
        right_bottom: [rect.max.x, rect.min.y],
        tex_left_top: [tex_coords.min.x, tex_coords.max.y],
        tex_right_bottom: [tex_coords.max.x, tex_coords.min.y],
        colour: extra.color,
    }
}

fn calculate_transform(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> TextTransform {
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);

    TextTransform {
        transform: cgmath::Matrix4::new(
            2.0 / (right - left), 0.0, 0.0, 0.0,
            0.0, 2.0 / (top - bottom), 0.0, 0.0,
            0.0, 0.0, -2.0 / (far - near), 0.0,
            tx, ty, tz, 1.0,
        ),
    }
}

impl TextContext {

    pub fn new<W>(
        device: Arc<Device>, 
        queue: Arc<Queue>,
        swapchain: Arc<Swapchain<W>>,
        images: &[Arc<SwapchainImage<W>>]
    ) -> Self where W: Send + Sync + 'static {

        println!("Creating TextContext");

        println!("Loading TextContext vertex_shader...");
        let vertex_shader = vertex_shader::Shader::load(device.clone())
            .expect("unable to load text vertex shader");
        
        println!("Loading TextContext fragment_shader...");
        let fragment_shader = fragment_shader::Shader::load(device.clone())
            .expect("unable to load fragment shader");

        println!("Loading TextContext font...");
        let font = FontArc::try_from_slice(include_bytes!("../../fonts/Hack-Regular.ttf"))
            .expect("unable to load font");

        println!("Loading GlyphBrushBuilder...");
        let glyph_brush = GlyphBrushBuilder::using_font(font)
            .build();

        let cache_dimensions = glyph_brush.texture_dimensions();
        let cache_dimensions = (cache_dimensions.0 as usize, cache_dimensions.1 as usize);
        let cache_pixel_buffer = vec![0; cache_dimensions.0 * cache_dimensions.1];

        println!("Creating render_pass...");
        let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Load,
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

        println!("Creating framebuffer...");
        let framebuffers = images.iter().map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        println!("Creating pipeline...");
        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<TextVertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .triangle_strip()
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
            .expect("Unable to create pipeline")
        );

        println!("Creating uniform_buffer_pool...");
        let uniform_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());

        println!("Loaded TextContext");
        TextContext {
            device: device.clone(),
            queue,
            glyph_brush,
            cache_dimensions,
            cache_pixel_buffer,
            pipeline,
            framebuffers,
            texts: vec!(),
            uniform_buffer_pool,
        }
    }

    pub fn queue_text(&mut self, x: f32, y: f32, font_size: f32, colour: [f32; 4], text: &str) {
        let dimensions = self.framebuffers[0].dimensions();
        let layout = Layout::default();
        let section = Section::default()
            .add_text(
                Text::new(text)
                    .with_scale(font_size * 10.0)
                    .with_color(colour),
                )
            .with_bounds((dimensions[0] as f32, dimensions[1] as f32))
            .with_layout(layout)
           .with_screen_position((0.0, 0.0))
           .to_owned();

        self.glyph_brush.queue(section.to_borrowed());

        let data = TextData {
            section,
            colour,
        };

        self.texts.push(data);
    }

    fn update_texture(cache_dimensions: (usize, usize), cache_pixel_buffer: &mut Vec<u8>, rect: Rectangle<u32>, src_data: &[u8]) {
        println!("TextContext update_texture at rect: {} {} {} {}", rect.min[0], rect.min[1], rect.max[0], rect.max[1]);

        let w = (rect.max[0] - rect.min[0]) as usize;
        let h = (rect.max[1] - rect.min[1]) as usize;
        let mut dst_id = rect.min[1] as usize * cache_dimensions.0 + rect.min[0] as usize;
        let mut src_id = 0;

        for _ in 0..h {
            let dst = &mut cache_pixel_buffer[dst_id..dst_id+w];
            let src = &src_data[src_id..src_id+w];
            dst.copy_from_slice(src);

            dst_id += cache_dimensions.0;
            src_id += w;
        }
    }

    fn get_max_image_demension(device: Arc<Device>) -> usize {
        let phys_dev = device.physical_device();
        let limits = phys_dev.limits();
        
        limits.max_image_dimension_2d() as usize
    }

    fn resize_cache(&mut self, width: usize, height: usize) {
        let max_image_dimension = Self::get_max_image_demension(self.device.clone());
        let glyph_dimensions = self.glyph_brush.texture_dimensions();
        let cache_dimensions = if (width > max_image_dimension || height > max_image_dimension)
            && ((glyph_dimensions.0 as usize) < max_image_dimension || (glyph_dimensions.1 as usize) < max_image_dimension)
        {
            (max_image_dimension, max_image_dimension)
        } else {
            (width, height)
        };

        println!("Resizing glyph texture: {}x{}", cache_dimensions.0, cache_dimensions.1);

        self.cache_dimensions = cache_dimensions;
        self.cache_pixel_buffer = vec![0; cache_dimensions.0 * cache_dimensions.1];
        self.glyph_brush.resize_texture(cache_dimensions.0 as u32, cache_dimensions.1 as u32);
    }
    
    fn upload_vertices<'a>(&'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder, 
        vertices: Vec<TextVertex>,
        image_num: usize,
    ) -> &'a mut AutoCommandBufferBuilder {
        let cache_pixel_buffer = &mut self.cache_pixel_buffer;

        let buffer = CpuAccessibleBuffer::<[u8]>::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            false,
            cache_pixel_buffer.iter().cloned()
        ).unwrap();

        let cache_dimensions = self.cache_dimensions;
        let (cache_tex, cache_tex_write) = ImmutableImage::uninitialized(
            self.device.clone(),
            Dimensions::Dim2d { width: cache_dimensions.0 as u32, height: cache_dimensions.1 as u32 },
            R8Unorm,
            1,
            ImageUsage {
                sampled: true,
                transfer_destination: true,
                .. ImageUsage::none()
            },
            ImageLayout::General,
            Some(self.queue.family())
        ).expect("Unable to create unintialised immutable image");

        let sampler = Sampler::new(
            self.device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Linear,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let dimensions = self.framebuffers[0].dimensions();
        let transform = calculate_transform(0.0, dimensions[0] as f32, 0.0, dimensions[1] as f32, 1.0, -1.0);
        let uniform_buffer = {
            self.uniform_buffer_pool.next(transform).unwrap()
        };

        let image_set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(0).unwrap().clone())
            .add_sampled_image(cache_tex.clone(), sampler)
            .expect("could not add sampled image to PersistentDescriptorSet 0")
            .build()
            .expect("TextContext: unable to create PersistentDescriptorSet 0")
        );
        let uniform_set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(1).unwrap().clone())
            .add_buffer(uniform_buffer)
            .expect("could not add uniform buffer to PersistentDescriptorSet 1")
            .build()
            .expect("TextContext: unable to create PersistentDescriptorSet 1")
        );

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            vertices.into_iter()
        ).expect("TextContext: unable to create vertex buffer");

        builder 
            .copy_buffer_to_image(buffer.clone(), cache_tex_write).unwrap()
            .begin_render_pass(
                self.framebuffers[image_num].clone(), 
                false, 
                vec!(ClearValue::None),
            ).expect("unable to copy buffer to image")

            .draw(
                self.pipeline.clone(),
                &DynamicState::none(),
                vertex_buffer.clone(),
                (image_set, uniform_set),
                ()
            ).expect("unable to draw to command buffer for glyph")

            .end_render_pass()
            .expect("unable to end render pass") 
    }

    pub fn draw_text<'a>(
        &'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder, 
        image_num: usize
    ) -> &'a mut AutoCommandBufferBuilder {

        let cache_dimensions = self.cache_dimensions;
        let cache_pixel_buffer = &mut self.cache_pixel_buffer; 
        
        let glyph_action = self.glyph_brush.process_queued(
            |rect, tex_data| Self::update_texture(cache_dimensions, cache_pixel_buffer, rect, tex_data),
            into_vertex,
        );
 
        match glyph_action {
            Ok(BrushAction::Draw(vertices)) => {
                self.upload_vertices(builder, vertices, image_num)
            },
            Ok(BrushAction::ReDraw) => {
                builder
            },
            Err(BrushError::TextureTooSmall { suggested, .. }) => {
                self.resize_cache(suggested.0 as usize, suggested.1 as usize);
                builder
            },
        }
    }
}

