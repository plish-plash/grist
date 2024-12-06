pub mod button;
pub mod checkbox;
pub mod label;

use crate::{GuiRenderer, PointerState};

pub trait View: 'static {
    fn render(&self, renderer: &mut GuiRenderer);
}

pub trait Control: 'static {
    fn handle_pointer(&mut self, state: PointerState);
}
