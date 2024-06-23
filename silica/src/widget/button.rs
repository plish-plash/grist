use grist::{impl_add_event_listener, Event, Obj};
use taffy::{geometry::Point, prelude::*};

use crate::{
    widget::{Label, Widget},
    Gui, GuiRenderer, HorizontalAlign, PointerState, ThemeColor, VerticalAlign, Visual,
};

pub struct BaseButton {
    was_pressed: bool,
}

impl BaseButton {
    fn new() -> Self {
        BaseButton { was_pressed: false }
    }
    fn set_pointer_state<F>(&mut self, state: PointerState, on_activate: F) -> Visual
    where
        F: FnOnce(),
    {
        if state == PointerState::Over && self.was_pressed {
            on_activate();
        }
        self.was_pressed = state == PointerState::Press;

        let mut visual = Visual::BUTTON;
        match state {
            PointerState::None => visual.background = Some(ThemeColor::ButtonNormal),
            PointerState::Over => visual.background = Some(ThemeColor::ButtonOver),
            PointerState::Press => visual.background = Some(ThemeColor::ButtonPress),
        }
        visual
    }
}

pub struct Button {
    node: NodeId,
    base: BaseButton,
    label: Obj<Label>,
    toggle: Option<bool>,
    pressed: Event<()>,
}

impl Button {
    pub fn new(gui: &mut Gui, label_text: &str, toggle: Option<bool>) -> Obj<Self> {
        let label = Label::with_text(gui, label_text);
        let mut label_guard = label.get_mut();
        label_guard.set_halign(HorizontalAlign::Center);
        label_guard.set_valign(VerticalAlign::Center);
        gui.set_style(
            label_guard.node(),
            Style {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        let style = Style {
            min_size: Size {
                width: Dimension::Length(128.),
                height: Dimension::Length(32.),
            },
            align_items: Some(AlignItems::Stretch),
            justify_items: Some(JustifyItems::Stretch),
            ..Default::default()
        };
        let label_node = label_guard.node();
        std::mem::drop(label_guard);
        let widget = gui.create_widget(style, Some(Visual::BUTTON), move |node| Button {
            node,
            base: BaseButton::new(),
            label,
            toggle,
            pressed: Event::new(),
        });
        let node = widget.get().node;
        gui.add_child(node, label_node);
        widget
    }

    pub fn label(&self) -> Obj<Label> {
        self.label.clone()
    }
}

impl Widget for Button {
    fn node(&self) -> NodeId {
        self.node
    }
    fn render(&self, renderer: &mut GuiRenderer, size: taffy::Size<f32>) {
        let mut border_width = 1.0;
        if self.toggle.unwrap_or(false) {
            border_width = 3.0;
        }
        renderer.draw_border(size, Rect::length(border_width));
    }
    fn handle_pointer(&mut self, gui: &mut Gui, state: PointerState) {
        let visual = self.base.set_pointer_state(state, || {
            if let Some(model) = self.toggle.as_mut() {
                *model = !*model;
            }
            self.pressed.emit(&());
        });
        gui.set_visual(self.node, Some(visual));
    }
}

impl_add_event_listener!(Button, pressed, (), add_pressed_listener);

pub struct Checkbox {
    node: NodeId,
    base: BaseButton,
    state: bool,
    rocker: bool,
    changed: Event<bool>,
}

impl Checkbox {
    pub fn new(gui: &mut Gui, state: bool) -> Obj<Self> {
        let style = Style {
            min_size: Size {
                width: Dimension::Length(24.),
                height: Dimension::Length(24.),
            },
            ..Default::default()
        };
        gui.create_widget(style, Some(Visual::BUTTON), move |node| Checkbox {
            node,
            base: BaseButton::new(),
            state,
            rocker: false,
            changed: Event::new(),
        })
    }
    pub fn new_rocker(gui: &mut Gui, state: bool) -> Obj<Self> {
        let style = Style {
            min_size: Size {
                width: Dimension::Length(48.),
                height: Dimension::Length(24.),
            },
            ..Default::default()
        };
        gui.create_widget(style, Some(Visual::BUTTON), move |node| Checkbox {
            node,
            base: BaseButton::new(),
            state,
            rocker: true,
            changed: Event::new(),
        })
    }
    pub fn state(&self) -> bool {
        self.state
    }
}

impl Widget for Checkbox {
    fn node(&self) -> NodeId {
        self.node
    }
    fn render(&self, renderer: &mut GuiRenderer, size: taffy::Size<f32>) {
        renderer.draw_border(size, Rect::length(1.0));
        if self.state {
            if self.rocker {
                let point = Point {
                    x: (size.width * 0.75) - (size.height / 4.0),
                    y: size.height / 4.0,
                };
                renderer.draw_rect(
                    point,
                    Size {
                        width: size.height / 2.0,
                        height: size.height / 2.0,
                    },
                );
            } else {
                let point = Point {
                    x: size.width / 4.0,
                    y: size.height / 4.0,
                };
                renderer.draw_rect(point, size.map(|x| x / 2.0));
            }
        } else if self.rocker {
            let point = Point {
                x: (size.width * 0.25) - (size.height / 4.0),
                y: size.height / 4.0,
            };
            renderer.draw_rect(
                point,
                Size {
                    width: size.height / 2.0,
                    height: size.height / 2.0,
                },
            );
        }
    }
    fn handle_pointer(&mut self, gui: &mut Gui, state: PointerState) {
        let visual = self.base.set_pointer_state(state, || {
            self.state = !self.state;
            self.changed.emit(&self.state);
        });
        gui.set_visual(self.node, Some(visual));
    }
}

impl_add_event_listener!(Checkbox, changed, bool, add_changed_listener);
