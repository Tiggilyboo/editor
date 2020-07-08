mod shaders;
use shaders::{
    Vertex,
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
    ImmutableBuffer,
    CpuBufferPool,
    TypedBufferAccess,
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
    pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>, 
        Box<dyn PipelineLayoutAbstract + Send + Sync>, 
        Arc<dyn RenderPassAbstract + Send + Sync>>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    uniform_buffer_pool: CpuBufferPool<TextTransform>,
    texts: Vec<TextData>,
    vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    index_buffer: Option<Arc<TypedBufferAccess<Content=[u16]> + Send + Sync>>,
    texture: TextureCache,
    background_colour: [f32; 4],
}

struct TextureCache {
    pub cache_pixel_buffer: Vec<u8>,
    pub cache_dimensions: (usize, usize),
    image: Option<Arc<ImmutableImage<R8Unorm>>>,
    sampler: Arc<Sampler>,
}

#[inline]
pub fn into_vertex(GlyphVertex {
    mut tex_coords,
    pixel_coords,
    bounds,
    extra,
}: GlyphVertex) -> TextVertex {
   
    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    TextVertex {
        left_top: [gl_rect.min.x, gl_rect.max.y],
        right_bottom: [gl_rect.max.x, gl_rect.min.y], 
        depth: extra.z,
        tex_left_top: [tex_coords.min.x, tex_coords.max.y],
        tex_right_bottom: [tex_coords.max.x, tex_coords.min.y],
        colour: [
            extra.color[0],
            extra.color[1],
            extra.color[2],
            extra.color[3]
        ],
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
            .expect("Unable to create pipeline")
        );

        println!("Creating uniform_buffer_pool...");
        let uniform_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());

        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        println!("Loaded TextContext");
        TextContext {
            device: device.clone(),
            queue,
            glyph_brush,
            texture: TextureCache {
                image: None,
                cache_dimensions,
                cache_pixel_buffer,
                sampler,
            },
            pipeline,
            framebuffers,
            texts: vec!(),
            uniform_buffer_pool,
            vertex_buffer: None, 
            index_buffer: None,
            background_colour: [0.1, 0.1, 0.1, 1.0],
        }
    }

    pub fn queue_text(&mut self, index: usize, x: f32, y: f32, font_size: f32, colour: [f32; 4], text: &str) {
        let dimensions = self.framebuffers[0].dimensions();

        let section = if self.texts.len() > index {
            let mut cached = self.texts[index].section.clone();
            cached.screen_position = (x, y);
            cached.bounds = (dimensions[0] as f32, dimensions[1] as f32);
            // TODO: index??
            cached.text[0].text = String::from(text);
            cached.text[0].scale = PxScale::from(font_size);

            cached
        } else {
            Section::default()
                .add_text(Text::new(text)
                    .with_scale(font_size)
                    .with_color(colour))
                .with_bounds((dimensions[0] as f32, dimensions[1] as f32))
                .with_layout(Layout::default())
                .with_screen_position((x, y))
                .to_owned()
        };
        self.glyph_brush.queue(section.to_borrowed());

        let data = TextData {
            section,
            colour,
        };

        self.texts.push(data);
    }

    fn update_texture(
        device: Arc<Device>,
        queue: Arc<Queue>, 
        builder: &mut AutoCommandBufferBuilder,
        cache_dimensions: (usize, usize), 
        cache_pixel_buffer: &mut Vec<u8>, 
        rect: Rectangle<u32>, 
        src_data: &[u8],
    ) -> Arc<ImmutableImage<R8Unorm>> {
        println!("TextContext update_texture at rect: {} {} {} {}", rect.min[0], rect.min[1], rect.max[0], rect.max[1]);

        let w = (rect.max[0] - rect.min[0]) as usize;
        let h = (rect.max[1] - rect.min[1]) as usize;
        let mut dst_id = rect.min[1] as usize * cache_dimensions.0 + rect.min[0] as usize;
        let mut src_id = 0;

        for _ in 0..h {
            let dst = &mut cache_pixel_buffer[dst_id..dst_id+w];
            let src = &src_data[src_id..src_id+w];
            dst.copy_from_slice(src);

            dst_id += cache_dimensions.0 as usize;
            src_id += w;
        }

        let buffer = CpuAccessibleBuffer::<[u8]>::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            cache_pixel_buffer.iter().cloned()
        ).unwrap();

        let (cache_tex, cache_tex_write) = ImmutableImage::uninitialized(
            device.clone(),
            Dimensions::Dim2d { width: cache_dimensions.0 as u32, height: cache_dimensions.1 as u32 },
            R8Unorm,
            1,
            ImageUsage {
                sampled: true,
                transfer_destination: true,
                .. ImageUsage::none()
            },
            ImageLayout::General,
            Some(queue.family())
        ).expect("Unable to create unintialised immutable image");

        builder.copy_buffer_to_image(
            buffer.clone(),
            cache_tex_write)
        .expect("unable to copy cache_pixel_buffer to cache_tex texture");

        cache_tex
    }

    fn get_max_image_dimension(device: Arc<Device>) -> usize {
        let phys_dev = device.physical_device();
        let limits = phys_dev.limits();
        
        limits.max_image_dimension_2d() as usize
    }

    fn resize_cache(&mut self, width: usize, height: usize) {
        let max_image_dimension = Self::get_max_image_dimension(self.device.clone());
        let glyph_dimensions = self.glyph_brush.texture_dimensions();
        let cache_dimensions = if (width > max_image_dimension || height > max_image_dimension)
            && ((glyph_dimensions.0 as usize) < max_image_dimension || (glyph_dimensions.1 as usize) < max_image_dimension)
        {
            (max_image_dimension, max_image_dimension)
        } else {
            (width, height)
        };

        println!("Resizing glyph texture: {}x{}", cache_dimensions.0, cache_dimensions.1);

        self.texture.cache_dimensions = cache_dimensions;
        self.texture.cache_pixel_buffer = vec![0; cache_dimensions.0 * cache_dimensions.1];
        self.glyph_brush.resize_texture(cache_dimensions.0 as u32, cache_dimensions.1 as u32);
    }

    fn upload_vertices(& mut self, vertices: Vec<TextVertex>) {
        let mut indices = vec!();
        let mut quadrupled_verts = vec!();
        let mut i = 0;

        for v in vertices.iter() {
            let glyph_verts = v.to_verts();
            quadrupled_verts.push(glyph_verts[0]);
            quadrupled_verts.push(glyph_verts[1]);
            quadrupled_verts.push(glyph_verts[2]);
            quadrupled_verts.push(glyph_verts[3]);

            let ic = i * 4;
            indices.push(ic);
            indices.push(ic+1);
            indices.push(ic+2);

            indices.push(ic+1);
            indices.push(ic+2);
            indices.push(ic+3);
            i += 1;
        }

        self.vertex_buffer = Some(CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            quadrupled_verts.into_iter()
        ).expect("TextContext: unable to create vertex buffer"));

        let (index_buffer, _future) = ImmutableBuffer::from_iter(
            indices.into_iter(),
            BufferUsage::index_buffer(),
            self.queue.clone(),
        ).expect("TextContext: unable to create index buffer");
         
        self.index_buffer = Some(index_buffer);
    }

    pub fn draw_text<'a>(
        &'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder, 
        image_num: usize
    ) -> &'a mut AutoCommandBufferBuilder {

        let cache_dimensions = self.texture.cache_dimensions;
        let cache_pixel_buffer = &mut self.texture.cache_pixel_buffer; 
        let device = self.device.clone();
        let queue = self.queue.clone();
        let mut updated_texture = self.texture.image.clone();
        
        let glyph_action = self.glyph_brush.process_queued(
            |rect, tex_data| {
                updated_texture = Some(Self::update_texture(
                    device.clone(),
                    queue.clone(),
                    builder, 
                    cache_dimensions, 
                    cache_pixel_buffer, 
                    rect, tex_data));
            },
            into_vertex,
        );
        self.texture.image = updated_texture;
 
        let requires_clear = match glyph_action {
            Ok(BrushAction::Draw(vertices)) => {
                self.upload_vertices(vertices);
                true
            },
            Ok(BrushAction::ReDraw) => false,
            Err(BrushError::TextureTooSmall { suggested, .. }) => {
                self.resize_cache(suggested.0 as usize, suggested.1 as usize);
                true
            },
        };

        let dimensions = self.framebuffers[image_num].dimensions();
        let transform = calculate_transform(0.0, dimensions[0] as f32, 0.0, dimensions[1] as f32, 1.0, -1.0);
        let uniform_buffer = {
            self.uniform_buffer_pool.next(transform).unwrap()
        };

        // Image not loaded yet
        if self.texture.image.is_none() {
            println!("Waiting for text cache image to load...");
            return builder;
        }

        let cache_tex = self.texture.image.clone().unwrap();
        let set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(0).unwrap().clone())
                .add_sampled_image(cache_tex, self.texture.sampler.clone())
                .expect("could not add sampled image to PersistentDescriptorSet binding 0")
                .add_buffer(uniform_buffer)
                .expect("could not add uniform buffer to PersistentDescriptorSet binding 1")
                .build()
                .expect("TextContext: unable to create PersistentDescriptorSet 0")
        );


        if requires_clear {
            // TODO: Clear the framebuffers
        }

        builder 
            .begin_render_pass(
                self.framebuffers[image_num].clone(), 
                false, 
                vec![self.background_colour.into()],
            ).expect("unable to copy buffer to image")

            .draw_indexed(
                self.pipeline.clone(),
                &DynamicState::none(),
                self.vertex_buffer.clone().unwrap(),
                self.index_buffer.clone().unwrap(),
                set.clone(), 
                ()
            ).expect("unable to draw to command buffer for glyph")

            .end_render_pass()
            .expect("unable to end render pass")
    }
}

