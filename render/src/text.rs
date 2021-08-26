mod shaders;
mod font;
mod unicode;

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::iter;

use self::shaders::{
    Vertex,
    vertex_shader,
    fragment_shader,
};
use uniform::{
    UniformTransform,
    calculate_transform,
};
use super::abstract_renderer::AbstractRenderer;

use winit::window::Window;

use vulkano::device::{
    Device,
    Queue,
}; 
use vulkano::format::{
    Format,
    ClearValue,
};
use vulkano::pipeline::{
    GraphicsPipeline,
    GraphicsPipelineAbstract,
    viewport::Viewport,
};
use vulkano::descriptor_set::{
    PersistentDescriptorSet,  
    DescriptorSet,
};
use vulkano::buffer::{
    BufferAccess,
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
    ImageDimensions,
    MipmapsCount,
    view::ImageView,
};
use vulkano::render_pass::{
    FramebufferAbstract, 
    Framebuffer,
    RenderPass,
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
    PrimaryAutoCommandBuffer,
    DynamicState,
    SubpassContents,
    pool::standard::StandardCommandPoolBuilder,
};  
use glyph_brush::{
    OwnedSection,
    OwnedText,
    Layout,
    GlyphBrush,
    GlyphBrushBuilder,
    BrushAction,
    BrushError,
    GlyphVertex,
    Rectangle,
};
use glyph_brush::ab_glyph::{
    FontArc,
    Rect,
    point,
};
use super::colour::ColourRGBA;
pub use self::font::FontBounds;

pub struct TextGroup {
    section: OwnedSection,
}

pub struct TextContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Option<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>, 
    framebuffers: Option<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    uniform_buffer_pool: CpuBufferPool<UniformTransform>,
    vertex_buffer: Option<Arc<ImmutableBuffer<[Vertex]>>>,
    index_buffer: Option<Arc<dyn TypedBufferAccess<Content=[u16]> + Send + Sync>>,
    
    vertex_shader: vertex_shader::Shader,
    fragment_shader: fragment_shader::Shader,
    
    glyph_brush: RefCell<GlyphBrush<TextVertex>>,
    font_bounds: Arc<Mutex<FontBounds>>,

    descriptor_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,
    texture: TextureCache,
    dimensions: [f32; 2],
}

#[derive(Default, Debug, Clone)]
pub struct TextVertex {
    pub left_top: [f32; 2],
    pub right_bottom: [f32; 2],
    pub depth: f32,
    pub tex_left_top: [f32; 2],
    pub tex_right_bottom: [f32; 2],
    pub colour: [f32; 4],
}

struct TextureCache {
    pub cache_pixel_buffer: Vec<u8>,
    pub cache_dimensions: (usize, usize),
    image: Option<Arc<ImageView<Arc<ImmutableImage>>>>,
    sampler: Arc<Sampler>,
    dirty: bool,
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

#[inline]
pub fn to_verts(text_vertex: &TextVertex) -> [Vertex; 4] {
    [
        // 1  2  5  6
        // 3  4  7  8
        Vertex::new([text_vertex.left_top[0], text_vertex.left_top[1]], text_vertex.tex_left_top, text_vertex.colour),
        Vertex::new([text_vertex.left_top[0], text_vertex.right_bottom[1]], [text_vertex.tex_left_top[0], text_vertex.tex_right_bottom[1]], text_vertex.colour),
        Vertex::new([text_vertex.right_bottom[0], text_vertex.left_top[1]], [text_vertex.tex_right_bottom[0], text_vertex.tex_left_top[1]], text_vertex.colour),
        Vertex::new([text_vertex.right_bottom[0], text_vertex.right_bottom[1]], text_vertex.tex_right_bottom, text_vertex.colour),
    ]
}

impl TextGroup {
    pub fn new() -> Self {
       let section = OwnedSection::default()
           .with_layout(Layout::default_single_line());

       Self {
           section,
       }
    }

    fn get_section(&self) -> &OwnedSection {
        &self.section
    }

    pub fn push(&mut self, text: String, scale: f32, colour: ColourRGBA) {
        let new_text = OwnedText::new(&text)
          .with_color(colour)
          .with_scale(scale);

        self.section.text.push(new_text);
    }

    pub fn clear(&mut self) {
        self.section.text = vec![];
    }

    pub fn screen_position(&self) -> (f32, f32) {
        self.section.screen_position
    }

    pub fn set_screen_position(&mut self, x: f32, y: f32) {
        self.section.screen_position = (x, y);
    }

    pub fn bounds(&self) -> (f32, f32) {
        self.section.bounds
    }

    fn line_string(&self) -> String {
        // TODO: NOT IDEAL, but need a way to simplify two iter loops char_indices with index
        // offset being summed properly...
        let line_string = self.section
            .text.iter()
            .flat_map(|t| t.text.chars())
            .collect();

        line_string
    }
}

impl AbstractRenderer for TextContext {
    fn get_pipeline(&self) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        self.pipeline.clone().expect("Uninitialised pipeline")
    }

    fn get_framebuffers(&self) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        self.framebuffers.clone().expect("Uninitialised framebuffers")
    }
    fn set_swap_chain(&mut self, swapchain: Arc<Swapchain<Window>>, images: &Vec<Arc<SwapchainImage<Window>>>) {
        let device = &self.device;

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
        ).unwrap()) as Arc<RenderPass>;

        let framebuffers = images.iter().map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Arc::new(
                Framebuffer::start(render_pass.clone())
                .add(view)
                .unwrap()
                .build()
                .unwrap()
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
            .expect("Unable to create text pipeline")
        );

        self.pipeline = Some(pipeline);
        self.framebuffers = Some(framebuffers);
    }
}

impl TextContext {
    pub fn new(
        device: Arc<Device>, 
        queue: Arc<Queue>,
        font_size: f32,
    ) -> Self {
        let vertex_shader = vertex_shader::Shader::load(device.clone())
            .expect("unable to load text vertex shader");
        
        let fragment_shader = fragment_shader::Shader::load(device.clone())
            .expect("unable to load fragment shader");

        let font = FontArc::try_from_slice(include_bytes!("../../fonts/Hack-Regular.ttf"))
            .expect("unable to load font");

        let font_bounds = Arc::new(Mutex::new(FontBounds::new(font.clone(), font_size)));
        
        let glyph_brush = RefCell::from(
            GlyphBrushBuilder::using_font(font)
                .cache_glyph_positioning(true)
                .cache_redraws(true)
                .build());

        let cache_dimensions = glyph_brush.borrow().texture_dimensions();
        let cache_dimensions = (cache_dimensions.0 as usize, cache_dimensions.1 as usize);
        let cache_pixel_buffer = vec![0; cache_dimensions.0 * cache_dimensions.1];

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

        TextContext {
            device: device.clone(),
            queue,
            glyph_brush,
            font_bounds,
            texture: TextureCache {
                image: None,
                cache_dimensions,
                cache_pixel_buffer,
                sampler,
                dirty: true,
            },
            pipeline: None,
            framebuffers: None,
            uniform_buffer_pool,
            vertex_buffer: None, 
            index_buffer: None,
            dimensions: [0.0, 0.0],
            descriptor_set: None,
            vertex_shader,
            fragment_shader,
        }
    }

    pub fn queue_text(&self, text: &TextGroup) {
        self.glyph_brush.borrow_mut().queue(text.get_section());
    }

    pub fn get_font_bounds(&self) -> Arc<Mutex<FontBounds>> {
        self.font_bounds.clone()
    }

    fn update_texture(
        cache_dimensions: (usize, usize), 
        cache_pixel_buffer: &mut Vec<u8>, 
        rect: Rectangle<u32>, 
        src_data: &[u8],
    ) {
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
    }

    fn upload_texture(&self) -> Arc<ImageView<Arc<ImmutableImage>>> {
        let buffer = CpuAccessibleBuffer::<[u8]>::from_iter( 
            self.device.clone(),
            BufferUsage::transfer_source(),
            true,
            self.texture.cache_pixel_buffer.iter().cloned(),
        ).expect("unable to upload cache pixel buffer to buffer pool");

        let dimensions = ImageDimensions::Dim2d { 
            width: self.texture.cache_dimensions.0 as u32, 
            height: self.texture.cache_dimensions.1 as u32,
            array_layers: 1,
        };
        // when _future is dropped, it will block the function until completed
        let (cache_tex, _future) = ImmutableImage::from_buffer(
            buffer,
            dimensions,
            MipmapsCount::One,
            Format::R8Unorm,
            self.queue.clone(),
        ).expect("Unable to create unintialised immutable image");

        drop(_future);
        
        ImageView::new(cache_tex).unwrap()
    }

    fn get_max_image_dimension(device: Arc<Device>) -> usize {
        let phys_dev = device.physical_device();
        let props = phys_dev.properties();
        
        props.max_image_dimension2_d as usize
    }

    fn resize_cache(&mut self, width: usize, height: usize) {
        let max_image_dimension = Self::get_max_image_dimension(self.device.clone());
        let glyph_dimensions = self.glyph_brush.borrow().texture_dimensions();
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
        self.glyph_brush.borrow_mut()
            .resize_texture(cache_dimensions.0 as u32, cache_dimensions.1 as u32);
    }

    fn upload_vertices(&mut self, vertices: Vec<TextVertex>) {
        let mut indices = vec!();
        let mut quadrupled_verts = vec!();
        let mut i = 0;

        for v in vertices.iter() {
            let glyph_verts = to_verts(v);
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

        let (vertex_buffer, _vfuture) = ImmutableBuffer::from_iter(
            quadrupled_verts.into_iter(),
            BufferUsage::vertex_buffer(),
            self.queue.clone(),
        ).expect("TextContext: unable to create vertex buffer");

        let (index_buffer, _ifuture) = ImmutableBuffer::from_iter(
            indices.into_iter(),
            BufferUsage::index_buffer(),
            self.queue.clone(),
        ).expect("TextContext: unable to create index buffer");

         
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
    }

    fn check_recreate_descriptor_set(&mut self, image_num: usize) {
        if self.texture.image.is_none() {
            return
        }
        let dimensions = self.get_framebuffers()[image_num].dimensions(); 
        let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
        if !self.texture.dirty && self.dimensions[0] == dimensions[0] && self.dimensions[1] == dimensions[1] {
            return
        }

        let transform = calculate_transform(0.0, dimensions[0], 0.0, dimensions[1], 1.0, -1.0);
        let uniform_buffer = {
            self.uniform_buffer_pool.next(transform).unwrap()
        };
        let cache_tex = self.texture.image.clone().unwrap();
        let pipeline = self.get_pipeline();
        let layout = pipeline.layout().descriptor_set_layouts().get(0)
            .expect("could not retrieve pipeline descriptor set layout 0");
    
        self.dimensions = dimensions;
        self.descriptor_set = Some(Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_sampled_image(cache_tex, self.texture.sampler.clone())
                .expect("could not add sampled image to PersistentDescriptorSet binding 0")
                .add_buffer(uniform_buffer)
                .expect("could not add uniform buffer to PersistentDescriptorSet binding 1")
                .build()
                .expect("TextContext: unable to create PersistentDescriptorSet 0")
        ));
        self.texture.dirty = false;
    }

    #[inline]
    fn draw_internal<'a>(&'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>, 
        image_num: usize,
    ) -> bool {
        self.check_recreate_descriptor_set(image_num);

        if self.vertex_buffer.is_none()
        || self.index_buffer.is_none()
        || self.texture.image.is_none() {
            println!("text draw_internal vertex_buffer, index_buffer or texture image is none");
            return false;
        }
    
        let vbuf: Arc<dyn BufferAccess + Send + Sync> = Arc::new(self.vertex_buffer.clone().unwrap());
        let framebuffers = self.get_framebuffers();
        let pipeline = self.get_pipeline();

        builder 
            .begin_render_pass(
                framebuffers[image_num].clone(), 
                SubpassContents::Inline,
                vec![ClearValue::None],
            ).expect("unable to begin text render pass")

            .draw_indexed(
                pipeline,
                &DynamicState::none(),
                vec![vbuf],
                self.index_buffer.clone().unwrap(),
                self.descriptor_set.clone().unwrap(), 
                (),
            ).expect("unable to draw to command buffer for glyph")

            .end_render_pass()
            .expect("unable to end text render pass");

        true
    }

    pub fn draw_text<'a>(&'a mut self, 
        builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, StandardCommandPoolBuilder>,
        image_num: usize,
    ) -> bool {
        let cache_dimensions = self.texture.cache_dimensions;
        let cache_pixel_buffer = &mut self.texture.cache_pixel_buffer;
        let mut updated_texture = false;
        let glyph_action = self.glyph_brush
            .borrow_mut()
            .process_queued(|rect, tex_data| {
                Self::update_texture(
                    cache_dimensions,
                    cache_pixel_buffer,
                    rect, tex_data);
                updated_texture = true;
            },
            into_vertex,
        );

        match glyph_action {
            Ok(BrushAction::Draw(vertices)) => {
                self.upload_vertices(vertices);
            },
            Ok(BrushAction::ReDraw) => (),
            Err(BrushError::TextureTooSmall { suggested, .. }) => {
                self.resize_cache(suggested.0 as usize, suggested.1 as usize);
            },
        };
        if updated_texture {
            self.texture.image = Some(self.upload_texture());
            self.texture.dirty = true;
        }

        self.draw_internal(builder, image_num)
    }
}

