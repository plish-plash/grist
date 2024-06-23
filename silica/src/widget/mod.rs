mod button;
mod label;

pub use button::{Button, Checkbox};
pub use label::Label;

use crate::{Gui, GuiRenderer, NodeId, PointerState};

pub trait Widget: 'static {
    fn node(&self) -> NodeId;
    fn render(&self, renderer: &mut GuiRenderer, size: taffy::Size<f32>);
    fn handle_pointer(&mut self, _gui: &mut Gui, _state: PointerState) {}
}
