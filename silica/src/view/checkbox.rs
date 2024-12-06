use grist::{impl_add_event_listener, Event};
use taffy::{Point, Rect, Size};

use crate::{Control, GuiRenderer, PointerState, SimpleColors, View};

pub struct CheckboxModel {
    pub enabled: bool,
    pub state: PointerState,
    pub value: bool,
}

impl Default for CheckboxModel {
    fn default() -> Self {
        Self {
            enabled: true,
            state: PointerState::None,
            value: false,
        }
    }
}

impl CheckboxModel {
    pub fn new(value: bool) -> Self {
        CheckboxModel {
            value,
            ..Default::default()
        }
    }
}

pub trait CheckboxView: 'static {
    fn render(&self, renderer: &mut GuiRenderer, model: &CheckboxModel);
}

#[derive(Default)]
pub struct SimpleCheckboxView {
    colors: SimpleColors,
}

impl SimpleCheckboxView {
    pub fn new(colors: SimpleColors) -> Self {
        SimpleCheckboxView { colors }
    }
}

impl CheckboxView for SimpleCheckboxView {
    fn render(&self, renderer: &mut GuiRenderer, model: &CheckboxModel) {
        renderer.set_color(self.colors.background(model.enabled, model.state));
        renderer.draw_rect();
        renderer.set_color(self.colors.foreground(model.enabled));
        renderer.draw_border(Rect::length(1.));
        if model.value {
            let size = renderer.size();
            let point = Point {
                x: size.width / 4.,
                y: size.height / 4.,
            };
            renderer.draw_rect_at(point, size.map(|x| x / 2.));
        }
    }
}

#[derive(Default)]
pub struct SimpleRockerView {
    colors: SimpleColors,
}

impl SimpleRockerView {
    pub fn new(colors: SimpleColors) -> Self {
        SimpleRockerView { colors }
    }
}

impl CheckboxView for SimpleRockerView {
    fn render(&self, renderer: &mut GuiRenderer, model: &CheckboxModel) {
        renderer.set_color(self.colors.background(model.enabled, model.state));
        renderer.draw_rect();
        renderer.set_color(self.colors.foreground(model.enabled));
        renderer.draw_border(Rect::length(1.));
        let size = renderer.size();
        let fill_size = Size {
            width: size.height / 2.,
            height: size.height / 2.,
        };
        if model.value {
            let point = Point {
                x: (size.width * 0.75) - (size.height / 4.),
                y: size.height / 4.,
            };
            renderer.draw_rect_at(point, fill_size);
        } else {
            let point = Point {
                x: (size.width * 0.25) - (size.height / 4.),
                y: size.height / 4.,
            };
            renderer.draw_rect_at(point, fill_size);
        }
    }
}

pub struct Checkbox {
    model: CheckboxModel,
    view: Box<dyn CheckboxView>,
    changed: Event<bool>,
}

impl Checkbox {
    pub fn new<V: CheckboxView>(model: CheckboxModel, view: V) -> Self {
        Checkbox {
            model,
            view: Box::new(view),
            changed: Event::new(),
        }
    }

    pub fn value(&self) -> bool {
        self.model.value
    }
    pub fn set_value(&mut self, value: bool) {
        self.model.value = value;
    }

    pub fn enabled(&self) -> bool {
        self.model.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.model.enabled = enabled;
    }
}

impl_add_event_listener!(Checkbox, changed, bool, add_changed_listener);

impl View for Checkbox {
    fn render(&self, renderer: &mut GuiRenderer) {
        self.view.render(renderer, &self.model);
    }
}

impl Control for Checkbox {
    fn handle_pointer(&mut self, state: PointerState) {
        if self.model.enabled
            && self.model.state == PointerState::Over
            && state == PointerState::Press
        {
            self.model.value = !self.model.value;
            self.changed.emit(&self.model.value);
        }
        self.model.state = state;
    }
}

// impl Widget for Checkbox {
//     fn node(&self) -> NodeId {
//         self.node
//     }
//     fn render(&self, renderer: &mut GuiRenderer, size: taffy::Size<f32>) {
//         renderer.draw_border(size, Rect::length(1.0));
//         if self.state {
//             if self.rocker {
//                 let point = Point {
//                     x: (size.width * 0.75) - (size.height / 4.0),
//                     y: size.height / 4.0,
//                 };
//                 renderer.draw_rect(
//                     point,
//                     Size {
//                         width: size.height / 2.0,
//                         height: size.height / 2.0,
//                     },
//                 );
//             } else {
//                 let point = Point {
//                     x: size.width / 4.0,
//                     y: size.height / 4.0,
//                 };
//                 renderer.draw_rect(point, size.map(|x| x / 2.0));
//             }
//         } else if self.rocker {
//             let point = Point {
//                 x: (size.width * 0.25) - (size.height / 4.0),
//                 y: size.height / 4.0,
//             };
//             renderer.draw_rect(
//                 point,
//                 Size {
//                     width: size.height / 2.0,
//                     height: size.height / 2.0,
//                 },
//             );
//         }
//     }
//     fn handle_pointer(&mut self, gui: &mut Gui, state: PointerState) {
//         let visual = self.base.set_pointer_state(state, || {
//             self.state = !self.state;
//             self.changed.emit(&self.state);
//         });
//         gui.set_visual(self.node, Some(visual));
//     }
// }
