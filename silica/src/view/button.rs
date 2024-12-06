use glyph_brush::{HorizontalAlign, VerticalAlign};
use grist::{impl_add_event_listener, Event};
use taffy::Rect;

use crate::{Control, GuiRenderer, PointerState, SimpleColors, Text, View};

pub struct ButtonModel {
    pub enabled: bool,
    pub state: PointerState,
    pub toggle: Option<bool>,
    pub label: Text,
}

impl Default for ButtonModel {
    fn default() -> Self {
        Self {
            enabled: true,
            state: PointerState::None,
            toggle: None,
            label: Default::default(),
        }
    }
}

impl ButtonModel {
    pub fn new(label: &str) -> Self {
        ButtonModel {
            label: Text {
                text: label.to_owned(),
                h_align: HorizontalAlign::Center,
                v_align: VerticalAlign::Center,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub trait ButtonView: 'static {
    fn render(&self, renderer: &mut GuiRenderer, model: &ButtonModel);
}

#[derive(Default)]
pub struct SimpleButtonView {
    colors: SimpleColors,
}

impl SimpleButtonView {
    pub fn new(colors: SimpleColors) -> Self {
        SimpleButtonView { colors }
    }
}

impl ButtonView for SimpleButtonView {
    fn render(&self, renderer: &mut GuiRenderer, model: &ButtonModel) {
        renderer.set_color(self.colors.background(model.enabled, model.state));
        renderer.draw_rect();
        renderer.set_color(self.colors.foreground(model.enabled));
        let border = if model.toggle.unwrap_or(false) {
            3.
        } else {
            1.
        };
        renderer.draw_border(Rect::length(border));
        renderer.draw_text(&model.label);
    }
}

pub struct Button {
    model: ButtonModel,
    view: Box<dyn ButtonView>,
    pressed: Event<()>,
}

impl Button {
    pub fn new<V: ButtonView>(model: ButtonModel, view: V) -> Self {
        Button {
            model,
            view: Box::new(view),
            pressed: Event::new(),
        }
    }
    pub fn with_label<V: ButtonView>(label: &str, view: V) -> Self {
        Self::new(ButtonModel::new(label), view)
    }

    pub fn enabled(&self) -> bool {
        self.model.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.model.enabled = enabled;
    }
}

impl_add_event_listener!(Button, pressed, (), add_pressed_listener);

impl View for Button {
    fn render(&self, renderer: &mut GuiRenderer) {
        self.view.render(renderer, &self.model);
    }
}

impl Control for Button {
    fn handle_pointer(&mut self, state: PointerState) {
        if self.model.enabled
            && self.model.state == PointerState::Over
            && state == PointerState::Press
        {
            if let Some(toggle) = self.model.toggle.as_mut() {
                *toggle = !*toggle;
            }
            self.pressed.emit(&());
        }
        self.model.state = state;
    }
}
