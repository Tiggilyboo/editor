mod shaders;
use shaders::{
    TextVertex,
    vertex_shader,
    fragment_shader,
};

use std::sync::Arc;
use std::iter;
use std::cell::RefCell;

use rusttype::{
    Font,
    PositionedGlyph,
    Scale,
    Rect,
    point,
};
use rusttype::gpu_cache::{
    Cache,
};  
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

const CACHE_WIDTH: usize = 1024;
const CACHE_HEIGHT: usize = 1024;

#[derive(Debug)]
struct TextData {
    glyphs: Vec<PositionedGlyph<'static>>,
    colour: [f32; 4],
}

pub struct TextContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    font: Font<'static>,
    cache: Cache<'static>,
    cache_pixel_buffer: Vec<u8>,
    pipeline: Arc<GraphicsPipeline<SingleBufferDefinition<TextVertex>, 
        Box<dyn PipelineLayoutAbstract + Send + Sync>, 
        Arc<dyn RenderPassAbstract + Send + Sync>>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    texts: Vec<TextData>,
}

pub trait DrawsText {
    fn draw_text(self, ctx: &mut TextContext, image_num: usize) -> AutoCommandBufferBuilder; 
}

impl DrawsText for AutoCommandBufferBuilder {
    fn draw_text(self, ctx: &mut TextContext, image_num: usize) -> AutoCommandBufferBuilder {
        ctx.draw_text(self, image_num)
    }
}

impl TextContext {
    pub fn new<W>(
        device: Arc<Device>, 
        queue: Arc<Queue>,
        swapchain: Arc<Swapchain<W>>,
        images: &[Arc<SwapchainImage<W>>]
    ) -> Self where W: Send + Sync + 'static {

        let font_data = include_bytes!("../../fonts/Hack-Regular.ttf") as &[u8];
        //let font_data = include_bytes!("../../fonts/DejaVuSans.ttf") as &[u8];
        let font = Font::from_bytes(font_data)
            .expect("unable to load font from data");

        let vertex_shader = vertex_shader::Shader::load(device.clone())
            .expect("unable to load text vertex shader");
        let fragment_shader = fragment_shader::Shader::load(device.clone())
            .expect("unable to load fragment shader");

        let cache = Cache::builder()
            .dimensions(CACHE_WIDTH as u32, CACHE_HEIGHT as u32)
            .build();
        let cache_pixel_buffer = vec![0; CACHE_WIDTH * CACHE_HEIGHT];

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

        let framebuffers = images.iter().map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer()
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

        TextContext {
            device: device.clone(),
            queue,
            font,
            cache,
            cache_pixel_buffer,
            pipeline,
            framebuffers,
            texts: vec!(),
        }
    }

    pub fn queue_text(&mut self, x: f32, y: f32, size: f32, colour: [f32; 4], text: &str) {
        let glyphs: Vec<PositionedGlyph> = self.font
            .layout(text, Scale::uniform(size), point(x,y))
            .map(|g| g.standalone())
            .collect();

        for g in &glyphs {
            self.cache.queue_glyph(0, g.clone());
        }
        let data = TextData {
            glyphs: glyphs.clone(),
            colour: colour,
        };

        self.texts.push(data);
    }

    pub fn draw_text(
        &mut self, 
        command_buffer: AutoCommandBufferBuilder, 
        image_num: usize) -> AutoCommandBufferBuilder {

        let dimensions = self.framebuffers[image_num].dimensions();
        let scr_w = dimensions[0];
        let scr_h = dimensions[1];
        let cache_pixel_buffer = &mut self.cache_pixel_buffer;
        let cache = &mut self.cache;

        cache.cache_queued(
            |rect, src_data| {
                let w = (rect.max.x - rect.min.x) as usize;
                let h = (rect.max.y - rect.min.y) as usize;
                let mut dst_id = rect.min.y as usize * CACHE_WIDTH + rect.min.x as usize;
                let mut src_id = 0;

                for _ in 0..h {
                    let dst = &mut cache_pixel_buffer[dst_id..dst_id+w];
                    let src = &src_data[src_id..src_id+w];
                    dst.copy_from_slice(src);

                    dst_id += CACHE_WIDTH;
                    src_id += w;
                }
            }
        ).unwrap();

        let buffer = CpuAccessibleBuffer::<[u8]>::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            false,
            cache_pixel_buffer.iter().cloned()
        ).unwrap();

        let (cache_tex, cache_tex_write) = ImmutableImage::uninitialized(
            self.device.clone(),
            Dimensions::Dim2d { width: CACHE_WIDTH as u32, height: CACHE_HEIGHT as u32 },
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
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(0).unwrap().clone())
            .add_sampled_image(cache_tex.clone(), sampler).unwrap()
            .build()
            .expect("unable to create PersistentDescriptorSet")
        );

        let mut command_buffer = command_buffer
            .copy_buffer_to_image(buffer.clone(), cache_tex_write).unwrap()
            .begin_render_pass(
                self.framebuffers[image_num].clone(), 
                false, 
                vec!(ClearValue::None),
            ).unwrap();

        for text in &mut self.texts.drain(..) {
            let verts: Vec<TextVertex> = text.glyphs.iter().flat_map(|g| {
                if let Ok(Some((uv, scr))) = cache.rect_for(0, g) {
                    let rect = Rect {
                        min: point(
                            (scr.min.x as f32 / scr_w as f32 - 0.5) * 2.0,
                            (scr.min.y as f32 / scr_h as f32 - 0.5) * 2.0
                        ),
                        max: point(
                            (scr.max.x as f32 / scr_w as f32 - 0.5) * 2.0,
                            (scr.max.y as f32 / scr_h as f32 - 0.5) * 2.0
                        )
                    };
                    vec!(
                        TextVertex::new([rect.min.x, rect.max.y], [uv.min.x, uv.max.y],text.colour),
                        TextVertex::new([rect.min.x, rect.min.y], [uv.min.x, uv.min.y], text.colour),
                        TextVertex::new([rect.max.x, rect.min.y], [uv.max.x, uv.min.y],text.colour),
                        
                        TextVertex::new([rect.max.x, rect.min.y], [uv.max.x, uv.min.y],text.colour),
                        TextVertex::new([rect.max.x, rect.max.y], [uv.max.x, uv.max.y], text.colour),
                        TextVertex::new([rect.min.x, rect.max.y], [uv.min.x, uv.max.y], text.colour),
                    ).into_iter()
                } else {
                    vec!().into_iter()
                }
            }).collect();

            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                self.device.clone(),
                BufferUsage::vertex_buffer(),
                false,
                verts.into_iter()
            ).expect("unable to create vertex buffer for glyph");

            command_buffer = command_buffer.draw(
                self.pipeline.clone(),
                &DynamicState::none(),
                vertex_buffer.clone(),
                set.clone(),
                ()
            ).expect("unable to draw to command buffer for glyph");
        }

        command_buffer.end_render_pass().unwrap()
    }
}

