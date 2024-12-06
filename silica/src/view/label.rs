use glyph_brush::FontId;
use palette::LinSrgba;

use crate::{GuiRenderer, HorizontalAlign, SimpleColors, Text, VerticalAlign, View};

pub struct Label {
    text: Text,
    color: LinSrgba,
}

impl View for Label {
    fn render(&self, renderer: &mut GuiRenderer) {
        renderer.set_color(self.color);
        renderer.draw_text(&self.text);
    }
}

impl Label {
    pub fn new() -> Self {
        Self::with_text("")
    }
    pub fn with_text(text: &str) -> Self {
        Label {
            text: Text {
                text: text.to_owned(),
                ..Default::default()
            },
            color: SimpleColors::FG_NORMAL,
        }
    }

    pub fn text(&self) -> &str {
        &self.text.text
    }
    pub fn set_text(&mut self, string: String) {
        self.text.text = string;
    }
    pub fn set_font(&mut self, font: FontId) {
        self.text.font = font;
    }
    pub fn set_font_size(&mut self, font_size: f32) {
        self.text.font_size = font_size;
    }
    pub fn set_halign(&mut self, h_align: HorizontalAlign) {
        self.text.h_align = h_align;
    }
    pub fn set_valign(&mut self, v_align: VerticalAlign) {
        self.text.v_align = v_align;
    }
    pub fn set_color(&mut self, color: LinSrgba) {
        self.color = color;
    }
}
