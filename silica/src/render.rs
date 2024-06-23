use glyph_brush::{ab_glyph::PxScale, Extra, Section};
use palette::LinSrgba;
use taffy::{geometry::Point, prelude::*};

pub use glyph_brush::{FontId, HorizontalAlign, VerticalAlign};

#[derive(Clone, Copy, Debug)]
pub enum ThemeColor {
    Background,
    ButtonNormal,
    ButtonOver,
    ButtonPress,
    ButtonDisable,
    Border,
    Foreground,
}

impl From<ThemeColor> for LinSrgba {
    fn from(value: ThemeColor) -> Self {
        match value {
            ThemeColor::Background => LinSrgba::new(0.153, 0.153, 0.153, 1.0),
            ThemeColor::ButtonNormal | ThemeColor::ButtonDisable => {
                LinSrgba::new(0.216, 0.216, 0.216, 1.0)
            }
            ThemeColor::ButtonOver => LinSrgba::new(0.278, 0.278, 0.278, 1.0),
            ThemeColor::ButtonPress => LinSrgba::new(0.341, 0.341, 0.341, 1.0),
            ThemeColor::Border => LinSrgba::new(0.906, 0.906, 0.906, 1.0),
            ThemeColor::Foreground => LinSrgba::new(0.906, 0.906, 0.906, 1.0),
        }
    }
}

#[derive(Clone)]
pub struct Visual {
    pub background: Option<ThemeColor>,
    pub border: Option<ThemeColor>,
    pub foreground: Option<ThemeColor>,
}

impl Visual {
    pub const BUTTON: Visual = Visual {
        background: Some(ThemeColor::ButtonNormal),
        border: None,
        foreground: Some(ThemeColor::Foreground),
    };
}

impl Default for Visual {
    fn default() -> Self {
        Visual {
            background: None,
            border: None,
            foreground: Some(ThemeColor::Foreground),
        }
    }
}

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
    color: LinSrgba,
}

impl<'a> GuiRenderer<'a> {
    pub(crate) fn new(renderer: &'a mut dyn Renderer) -> Self {
        GuiRenderer {
            renderer,
            translation: Point::ZERO,
            translation_stack: Vec::new(),
            color: Default::default(),
        }
    }

    pub fn save(&mut self) {
        self.translation_stack.push(self.translation);
    }
    pub fn restore(&mut self) {
        self.translation = self.translation_stack.pop().unwrap();
    }
    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.translation.x += tx;
        self.translation.y += ty;
    }

    pub fn set_color(&mut self, color: ThemeColor) {
        self.color = color.into();
    }
    pub fn draw_border(&mut self, size: Size<f32>, border: Rect<f32>) {
        let mut pb = PathBuilder::new(self.renderer, self.translation, self.color);
        pb.move_to(0.5, 0.5);
        if border.top > 0.0 {
            pb.set_line_width(border.top);
            pb.line_to(size.width - 0.5, 0.5);
        } else {
            pb.move_to(size.width - 0.5, 0.5);
        }
        if border.right > 0.0 {
            pb.set_line_width(border.right);
            pb.line_to(size.width - 0.5, size.height - 0.5);
        } else {
            pb.move_to(size.width - 0.5, size.height - 0.5);
        }
        if border.bottom > 0.0 {
            pb.set_line_width(border.bottom);
            pb.line_to(0.5, size.height - 0.5);
        } else {
            pb.move_to(0.5, size.height - 0.5);
        }
        if border.left > 0.0 {
            pb.set_line_width(border.left);
            pb.line_to(0.5, 0.5);
        }
    }
    pub fn draw_rect(&mut self, point: Point<f32>, size: Size<f32>) {
        self.renderer
            .queue_rect(self.translation + point, size, self.color);
    }
    pub fn draw_text(&mut self, point: Point<f32>, size: Size<f32>, text: &Text) {
        let layout = glyph_brush::Layout::default_single_line()
            .h_align(text.h_align)
            .v_align(text.v_align);
        let screen_position = (
            self.translation.x
                + point.x
                + match text.h_align {
                    HorizontalAlign::Left => 0.,
                    HorizontalAlign::Center => size.width / 2.,
                    HorizontalAlign::Right => size.width,
                },
            self.translation.y
                + point.y
                + match text.v_align {
                    VerticalAlign::Top => 0.,
                    VerticalAlign::Center => size.height / 2.,
                    VerticalAlign::Bottom => size.height,
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
        let bounds = (size.width, size.height);
        self.renderer.queue_text(Section {
            screen_position,
            bounds,
            layout,
            text: vec![text],
        });
    }
}
