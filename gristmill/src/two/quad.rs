use glyph_brush::{
    ab_glyph::{Font, FontArc, PxScale},
    BrushAction, BrushError, GlyphBrushBuilder, GlyphCruncher, Section,
};
use grist::WeakObj;
use miniquad::*;
use palette::LinSrgba;
use silica::taffy::{prelude::*, Point};

use crate::{
    math::{BVec2, Vec2},
    two::Rect,
    RenderingContext, Texture,
};

#[derive(Clone)]
#[repr(C)]
struct Quad {
    rect: [f32; 4],
    uv: [f32; 4],
    color: [f32; 4],
}

type GlyphBrush = glyph_brush::GlyphBrush<(Quad, usize)>;

mod shader {
    use crate::math::Vec2;
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    const vec2 FLIP_Y = vec2(1.0, -1.0);

    attribute vec2 vert_pos;

    attribute vec4 inst_rect;
    attribute vec4 inst_uv;
    attribute vec4 inst_color;

    uniform vec2 screen_size;

    varying lowp vec2 texcoord;
    varying lowp vec4 color;

    void main() {
        vec2 pos = inst_rect.xy + (vert_pos * inst_rect.zw);
        gl_Position = vec4((pos / screen_size * 2.0 - 1.0) * FLIP_Y, 0.0, 1.0);
        texcoord = inst_uv.xy + (vert_pos * inst_uv.zw);
        color = inst_color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    varying lowp vec4 color;

    uniform sampler2D tex;
    uniform sampler2D mask;

    void main() {
        mediump float alpha = texture2D(mask, texcoord).r;
        if (alpha <= 0.0) {
            discard;
        }
        gl_FragColor = texture2D(tex, texcoord) * color * vec4(1.0, 1.0, 1.0, alpha);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string(), "mask".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("screen_size", UniformType::Float2)],
            },
        }
    }

    pub fn attributes() -> [VertexAttribute; 4] {
        [
            VertexAttribute::with_buffer("vert_pos", VertexFormat::Float2, 0),
            VertexAttribute::with_buffer("inst_rect", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("inst_uv", VertexFormat::Float4, 1),
            VertexAttribute::with_buffer("inst_color", VertexFormat::Float4, 1),
        ]
    }

    #[repr(C)]
    pub struct Uniforms {
        pub screen_size: Vec2,
    }
}

pub trait Renderable {
    fn render(&self, renderer: &mut QuadRenderer);
}

pub struct RenderList(Vec<WeakObj<dyn Renderable>>);

impl RenderList {
    pub fn new() -> Self {
        RenderList(Vec::new())
    }
    pub fn add(&mut self, renderable: WeakObj<dyn Renderable>) {
        self.0.push(renderable);
    }
}

pub struct RenderQuad<'a> {
    pub texture: Option<&'a Texture>,
    pub color: LinSrgba,
    pub rect: Rect,
    pub uv_rect: Rect,
    pub flip: BVec2,
    pub scroll: bool,
}

impl<'a> Default for RenderQuad<'a> {
    fn default() -> Self {
        const WHITE: LinSrgba = LinSrgba::new(1., 1., 1., 1.);
        Self {
            texture: None,
            color: WHITE,
            rect: Rect::ZERO,
            uv_rect: Rect::ONE,
            flip: BVec2::FALSE,
            scroll: true,
        }
    }
}

struct GlyphLayer {
    layer: usize,
    combine: bool,
}

impl GlyphLayer {
    fn new() -> Self {
        GlyphLayer {
            layer: 0,
            combine: true,
        }
    }
    fn next(&mut self) -> usize {
        if !self.combine {
            self.layer += 1;
            self.combine = true;
        }
        self.layer
    }
    fn finish(&mut self) {
        self.combine = false;
    }
    fn reset(&mut self) {
        self.combine = true;
        self.layer = 0;
    }
}

enum InstanceRange {
    Instances(TextureId, std::ops::Range<usize>),
    Text(usize),
}

pub struct QuadRenderer {
    pixel_perfect: bool,
    instances: Vec<Quad>,
    instance_ranges: Vec<InstanceRange>,
    screen_size: Vec2,
    scroll_offset: Vec2,
    scale: f32,
    pipeline: Pipeline,
    vertex_buffer: BufferId,
    instance_buffer: BufferId,
    index_buffer: BufferId,
    white_pixel: Texture,
    glyph_brush: GlyphBrush,
    glyph_texture: TextureId,
    glyph_layer: GlyphLayer,
    glyph_instances: Vec<Vec<Quad>>,
}

impl QuadRenderer {
    fn create_glyph_texture(
        context: &mut RenderingContext,
        (width, height): (u32, u32),
    ) -> TextureId {
        context.new_texture(
            TextureAccess::Static,
            TextureSource::Empty,
            TextureParams {
                kind: TextureKind::Texture2D,
                format: TextureFormat::Alpha,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width,
                height,
                allocate_mipmaps: false,
            },
        )
    }

    pub fn new(context: &mut RenderingContext, fonts: Vec<FontArc>, pixel_perfect: bool) -> Self {
        let vertices: [Vec2; 4] = [
            Vec2 { x: 0., y: 0. },
            Vec2 { x: 1., y: 0. },
            Vec2 { x: 1., y: 1. },
            Vec2 { x: 0., y: 1. },
        ];
        let vertex_buffer = context.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = context.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let instance_buffer = context.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Quad>(1024),
        );

        let shader = context
            .new_shader(
                ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                shader::meta(),
            )
            .unwrap();
        let pipeline = context.new_pipeline(
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &shader::attributes(),
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );
        let white_pixel = Texture::new_rgba8(context, 1, 1, &[255; 4]);
        let glyph_brush = GlyphBrushBuilder::using_fonts(fonts).build();
        let glyph_texture = Self::create_glyph_texture(context, glyph_brush.texture_dimensions());

        QuadRenderer {
            pixel_perfect,
            instances: Vec::new(),
            instance_ranges: Vec::new(),
            screen_size: Vec2::ONE,
            scroll_offset: Vec2::ZERO,
            scale: 1.,
            pipeline,
            vertex_buffer,
            instance_buffer,
            index_buffer,
            white_pixel,
            glyph_brush,
            glyph_texture,
            glyph_layer: GlyphLayer::new(),
            glyph_instances: Vec::new(),
        }
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
    }

    pub fn scroll_offset(&self) -> Vec2 {
        self.scroll_offset
    }
    pub fn set_scroll_offset(&mut self, scroll: Vec2) {
        self.scroll_offset = scroll;
        if self.pixel_perfect {
            self.scroll_offset.x = self.scroll_offset.x.round();
            self.scroll_offset.y = self.scroll_offset.y.round();
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    fn transform(&self, mut rect: Rect) -> Rect {
        rect.position *= self.scale;
        rect.size *= self.scale;
        if self.pixel_perfect {
            rect.position.x = rect.position.x.round();
            rect.position.y = rect.position.y.round();
            rect.size.x = rect.size.x.floor();
            rect.size.y = rect.size.y.floor();
        }
        rect
    }
    fn process_queued_text(&mut self, context: &mut RenderingContext) {
        let mut brush_action;
        loop {
            brush_action = self.glyph_brush.process_queued(
                |rect, tex_data| {
                    // Update part of gpu texture with new glyph alpha values
                    context.texture_update_part(
                        self.glyph_texture,
                        rect.min[0] as i32,
                        rect.min[1] as i32,
                        rect.width() as i32,
                        rect.height() as i32,
                        tex_data,
                    );
                },
                |glyph_vertex| {
                    let pos: Rect = glyph_vertex.pixel_coords.into();
                    let uv: Rect = glyph_vertex.tex_coords.into();
                    let color = glyph_vertex.extra.color;
                    (
                        Quad {
                            rect: pos.into(),
                            uv: uv.into(),
                            color,
                        },
                        glyph_vertex.extra.z as usize,
                    )
                },
            );

            // If the cache texture is too small to fit all the glyphs, resize and try again
            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    // Recreate texture as a larger size to fit more
                    println!("Resizing glyph texture to {}x{}", suggested.0, suggested.1);
                    self.glyph_texture = Self::create_glyph_texture(context, suggested);
                    self.glyph_brush.resize_texture(suggested.0, suggested.1);
                }
            }
        }

        // If the text has changed from what was last drawn, store new instances
        match brush_action.unwrap() {
            BrushAction::Draw(instances) => {
                self.glyph_instances.clear();
                self.glyph_instances
                    .resize(self.glyph_layer.layer + 1, Vec::new());
                for instance in instances {
                    self.glyph_instances[instance.1].push(instance.0);
                }
            }
            BrushAction::ReDraw => {}
        }
    }

    pub fn queue(&mut self, quad: RenderQuad) {
        let texture = quad.texture.unwrap_or(&self.white_pixel).id();
        let mut rect = self.transform(quad.rect);
        if rect.width() <= 0. || rect.height() <= 0. {
            return;
        }
        if quad.scroll {
            rect.position -= self.scroll_offset;
        }
        if rect.x() + rect.width() < 0.
            || rect.y() + rect.height() < 0.
            || rect.x() >= self.screen_size.x
            || rect.y() >= self.screen_size.y
        {
            // rect is outside of the screen
            return;
        }
        let mut uv: [f32; 4] = quad.uv_rect.into();
        if quad.flip.x {
            uv[0] += uv[2];
            uv[2] *= -1.;
        }
        if quad.flip.y {
            uv[1] += uv[3];
            uv[3] *= -1.;
        }

        self.glyph_layer.finish();
        self.instances.push(Quad {
            rect: rect.into(),
            uv,
            color: quad.color.into(),
        });
        let end = self.instances.len();
        let mut appended = false;
        if let Some(InstanceRange::Instances(instance_texture, range)) =
            self.instance_ranges.last_mut()
        {
            if *instance_texture == texture {
                range.end = end;
                appended = true;
            }
        }
        if !appended {
            let start = end - 1;
            self.instance_ranges
                .push(InstanceRange::Instances(texture, start..end));
        }
    }
    pub fn queue_color(&mut self, rect: Rect, color: LinSrgba) {
        self.queue(RenderQuad {
            color,
            rect,
            ..Default::default()
        });
    }
    pub fn queue_texture(&mut self, rect: Rect, texture: &Texture) {
        self.queue(RenderQuad {
            texture: Some(texture),
            rect,
            ..Default::default()
        });
    }
    pub fn queue_all(&mut self, render_list: &mut RenderList) {
        render_list.0.retain(|renderable| {
            if let Some(renderable) = renderable.try_upgrade() {
                renderable.get().render(self);
                true
            } else {
                false
            }
        });
    }
    pub fn render(&mut self, context: &mut RenderingContext) {
        self.process_queued_text(context);
        context.apply_pipeline(&self.pipeline);
        context.apply_uniforms(UniformsSource::table(&shader::Uniforms {
            screen_size: self.screen_size,
        }));
        let white_pixel = self.white_pixel.id();
        let mut bindings = Bindings {
            vertex_buffers: vec![self.vertex_buffer, self.instance_buffer],
            index_buffer: self.index_buffer,
            images: vec![white_pixel, white_pixel],
        };
        for instance_range in self.instance_ranges.drain(..) {
            let num_instances = match instance_range {
                InstanceRange::Instances(texture, range) => {
                    bindings.images[0] = texture;
                    bindings.images[1] = white_pixel;
                    let len = range.len();
                    context.buffer_update(
                        self.instance_buffer,
                        BufferSource::slice(&self.instances[range]),
                    );
                    len
                }
                InstanceRange::Text(layer) => {
                    bindings.images[0] = white_pixel;
                    bindings.images[1] = self.glyph_texture;
                    let instances = &self.glyph_instances[layer];
                    context.buffer_update(self.instance_buffer, BufferSource::slice(instances));
                    instances.len()
                }
            };
            context.apply_bindings(&bindings);
            context.draw(0, 6, num_instances.try_into().unwrap());
        }
        self.instances.clear();
        self.glyph_layer.reset();
    }
    pub fn render_pass(&mut self, context: &mut RenderingContext) {
        context.begin_default_pass(Default::default());
        self.render(context);
        context.end_render_pass();
    }
}

impl silica::Renderer for QuadRenderer {
    fn queue_rect(&mut self, point: Point<f32>, size: Size<f32>, color: LinSrgba) {
        self.queue(RenderQuad {
            color,
            rect: Rect::new(point.x, point.y, size.width, size.height),
            scroll: false,
            ..Default::default()
        });
    }
    fn queue_text(&mut self, mut section: Section) {
        let layer = self.glyph_layer.next();
        for text in section.text.iter_mut() {
            text.extra.z = layer as f32;
        }
        self.glyph_brush.queue(section);
        if !matches!(self.instance_ranges.last(), Some(InstanceRange::Text(_))) {
            self.instance_ranges.push(InstanceRange::Text(layer));
        }
    }
    fn pt_to_px_scale(&self, font: silica::FontId, pt_size: f32) -> PxScale {
        let font = self
            .glyph_brush
            .fonts()
            .get(font.0)
            .expect("invalid FontId");
        font.pt_to_px_scale(pt_size).unwrap()
    }
}
