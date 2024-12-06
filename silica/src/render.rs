use glyph_brush::{ab_glyph::PxScale, Extra, Section};
use palette::LinSrgba;
use taffy::{geometry::Point, prelude::*};

use crate::PointerState;

pub use glyph_brush::{FontId, HorizontalAlign, VerticalAlign};

#[derive(Debug, Clone)]
pub struct Text {
    pub font: FontId,
    pub font_size: f32,
    pub text: String,
    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            font: Default::default(),
            font_size: 14.0,
            text: String::new(),
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
        }
    }
}

pub trait Renderer {
    fn queue_rect(&mut self, point: Point<f32>, size: Size<f32>, color: LinSrgba);
    fn queue_text(&mut self, section: Section);
    fn pt_to_px_scale(&self, font: FontId, pt_size: f32) -> PxScale;
}

struct PathBuilder<'a> {
    renderer: &'a mut dyn Renderer,
    translation: Point<f32>,
    start: Point<f32>,
    line_width: f32,
    color: LinSrgba,
}

impl<'a> PathBuilder<'a> {
    fn new(renderer: &'a mut dyn Renderer, translation: Point<f32>, color: LinSrgba) -> Self {
        PathBuilder {
            renderer,
            translation,
            start: Point::ZERO,
            line_width: 0.0,
            color,
        }
    }
    fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.start.x = x;
        self.start.y = y;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let line_extent = self.line_width / 2.0;
        if (x - self.start.x).abs() > (y - self.start.y).abs() {
            // Horizontal
            let rect_x = x.min(self.start.x) - line_extent;
            let rect_y = self.start.y - line_extent;
            let rect_width = (x.max(self.start.x) + line_extent) - rect_x;
            let rect_height = self.line_width;
            self.renderer.queue_rect(
                Point {
                    x: self.translation.x + rect_x,
                    y: self.translation.y + rect_y,
                },
                Size {
                    width: rect_width,
                    height: rect_height,
                },
                self.color,
            );
        } else {
            // Vertical
            let rect_x = self.start.x - line_extent;
            let rect_y = y.min(self.start.y) - line_extent;
            let rect_width = self.line_width;
            let rect_height = (y.max(self.start.y) + line_extent) - rect_y;
            self.renderer.queue_rect(
                Point {
                    x: self.translation.x + rect_x,
                    y: self.translation.y + rect_y,
                },
                Size {
                    width: rect_width,
                    height: rect_height,
                },
                self.color,
            );
        }
        self.move_to(x, y);
    }
}

pub struct GuiRenderer<'a> {
    renderer: &'a mut dyn Renderer,
    translation: Point<f32>,
    translation_stack: Vec<Point<f32>>,
    size: Size<f32>,
    color: LinSrgba,
}

impl<'a> GuiRenderer<'a> {
    pub(crate) fn new(renderer: &'a mut dyn Renderer) -> Self {
        GuiRenderer {
            renderer,
            translation: Point::ZERO,
            translation_stack: Vec::new(),
            size: Size::ZERO,
            color: Default::default(),
        }
    }

    pub fn push_translation(&mut self) {
        self.translation_stack.push(self.translation);
    }
    pub fn pop_translation(&mut self) {
        self.translation = self.translation_stack.pop().unwrap();
    }
    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.translation.x += tx;
        self.translation.y += ty;
    }
    pub fn position(&self) -> Point<f32> {
        self.translation
    }
    pub fn size(&self) -> Size<f32> {
        self.size
    }
    pub fn set_size(&mut self, size: Size<f32>) {
        self.size = size;
    }

    pub fn set_color(&mut self, color: LinSrgba) {
        self.color = color;
    }
    pub fn draw_border(&mut self, border: Rect<f32>) {
        let mut pb = PathBuilder::new(self.renderer, self.translation, self.color);
        pb.move_to(0.5, 0.5);
        if border.top > 0.0 {
            pb.set_line_width(border.top);
            pb.line_to(self.size.width - 0.5, 0.5);
        } else {
            pb.move_to(self.size.width - 0.5, 0.5);
        }
        if border.right > 0.0 {
            pb.set_line_width(border.right);
            pb.line_to(self.size.width - 0.5, self.size.height - 0.5);
        } else {
            pb.move_to(self.size.width - 0.5, self.size.height - 0.5);
        }
        if border.bottom > 0.0 {
            pb.set_line_width(border.bottom);
            pb.line_to(0.5, self.size.height - 0.5);
        } else {
            pb.move_to(0.5, self.size.height - 0.5);
        }
        if border.left > 0.0 {
            pb.set_line_width(border.left);
            pb.line_to(0.5, 0.5);
        }
    }
    pub fn draw_rect(&mut self) {
        self.renderer
            .queue_rect(self.translation, self.size, self.color);
    }
    pub fn draw_rect_at(&mut self, point: Point<f32>, size: Size<f32>) {
        self.renderer
            .queue_rect(self.translation + point, size, self.color);
    }
    pub fn draw_text(&mut self, text: &Text) {
        let mut layout = if text.text.contains('\n') {
            glyph_brush::Layout::default_wrap()
        } else {
            glyph_brush::Layout::default_single_line()
        };
        layout = layout.h_align(text.h_align).v_align(text.v_align);
        let screen_position = (
            self.translation.x
                + match text.h_align {
                    HorizontalAlign::Left => 0.,
                    HorizontalAlign::Center => self.size.width / 2.,
                    HorizontalAlign::Right => self.size.width,
                },
            self.translation.y
                + match text.v_align {
                    VerticalAlign::Top => 0.,
                    VerticalAlign::Center => self.size.height / 2.,
                    VerticalAlign::Bottom => self.size.height,
                },
        );
        let text = glyph_brush::Text {
            text: &text.text,
            scale: self.renderer.pt_to_px_scale(text.font, text.font_size),
            font_id: text.font,
            extra: Extra {
                color: self.color.into(),
                z: 0.,
            },
        };
        let bounds = (self.size.width, self.size.height);
        self.renderer.queue_text(Section {
            screen_position,
            bounds,
            layout,
            text: vec![text],
        });
    }
}

pub struct SimpleColors {
    pub bg_normal: LinSrgba,
    pub bg_hover: LinSrgba,
    pub bg_press: LinSrgba,
    pub bg_disable: LinSrgba,
    pub fg_normal: LinSrgba,
    pub fg_disable: LinSrgba,
}

impl SimpleColors {
    pub const BG_NORMAL: LinSrgba = LinSrgba::new(0.216, 0.216, 0.216, 1.0);
    pub const BG_HOVER: LinSrgba = LinSrgba::new(0.278, 0.278, 0.278, 1.0);
    pub const BG_PRESS: LinSrgba = LinSrgba::new(0.341, 0.341, 0.341, 1.0);
    pub const BG_DISABLE: LinSrgba = LinSrgba::new(0.216, 0.216, 0.216, 0.5);
    pub const FG_NORMAL: LinSrgba = LinSrgba::new(0.906, 0.906, 0.906, 1.0);
    pub const FG_DISABLE: LinSrgba = LinSrgba::new(0.906, 0.906, 0.906, 0.5);

    pub fn background(&self, enabled: bool, state: PointerState) -> LinSrgba {
        if enabled {
            match state {
                PointerState::None => self.bg_normal,
                PointerState::Over => self.bg_hover,
                PointerState::Press => self.bg_press,
            }
        } else {
            self.bg_disable
        }
    }
    pub fn foreground(&self, enabled: bool) -> LinSrgba {
        if enabled {
            self.fg_normal
        } else {
            self.fg_disable
        }
    }
}

impl Default for SimpleColors {
    fn default() -> Self {
        Self {
            bg_normal: Self::BG_NORMAL,
            bg_hover: Self::BG_HOVER,
            bg_press: Self::BG_PRESS,
            bg_disable: Self::BG_DISABLE,
            fg_normal: Self::FG_NORMAL,
            fg_disable: Self::FG_DISABLE,
        }
    }
}
