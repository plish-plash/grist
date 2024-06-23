use glyph_brush::FontId;
use grist::Obj;
use taffy::{
    geometry::{Point, Size},
    Style,
};

use crate::{widget::Widget, Gui, GuiRenderer, HorizontalAlign, NodeId, Text, VerticalAlign};

pub struct Label {
    node: NodeId,
    text: Text,
}

impl Widget for Label {
    fn node(&self) -> NodeId {
        self.node
    }
    fn render(&self, renderer: &mut GuiRenderer, size: Size<f32>) {
        renderer.draw_text(Point::ZERO, size, &self.text);
    }
}

impl Label {
    pub fn new(gui: &mut Gui) -> Obj<Self> {
        Self::with_text(gui, "")
    }
    pub fn with_text(gui: &mut Gui, text: &str) -> Obj<Self> {
        gui.create_widget(Style::DEFAULT, Some(Default::default()), move |node| {
            Label {
                node,
                text: Text {
                    text: text.to_owned(),
                    ..Default::default()
                },
            }
        })
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
}
