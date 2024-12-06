mod render;
pub mod view;

use grist::{obj_upcast, Obj};
use palette::LinSrgba;
use std::collections::HashMap;
use taffy::{prelude::*, Point};

use view::{Control, View};

pub use render::*;
pub use taffy::{self, NodeId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerState {
    None,
    Over,
    Press,
}

#[derive(Default)]
pub struct GuiState {
    screen_size: Size<f32>,
    pointer: Point<f32>,
    pointer_down: bool,
    highlight: Option<NodeId>,
}

pub struct Gui {
    state: GuiState,
    layout: TaffyTree<()>,
    root: NodeId,
    views: HashMap<NodeId, Obj<dyn View>>,
    controls: HashMap<NodeId, Obj<dyn Control>>,
}

impl Gui {
    pub fn new() -> Self {
        let mut layout = TaffyTree::new();
        let root = layout.new_leaf(Style::DEFAULT).unwrap();
        Gui {
            state: Default::default(),
            layout,
            root,
            views: HashMap::new(),
            controls: HashMap::new(),
        }
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.state.screen_size = Size { width, height };
        self.layout();
    }
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn add_root(&mut self) -> NodeId {
        self.layout.new_leaf(Style::DEFAULT).unwrap()
    }
    pub fn set_root(&mut self, root: NodeId) {
        if self.root != root {
            self.root = root;
            self.layout();
            self.handle_pointer_motion(self.state.pointer.x, self.state.pointer.y);
        }
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.layout.add_child(parent, child).unwrap();
    }
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        self.layout.remove_child(parent, child).unwrap();
        if self.state.highlight == Some(child) {
            self.state.highlight = None;
        }
    }

    pub fn add_node(&mut self, parent: NodeId, style: Style) -> NodeId {
        let node = self.layout.new_leaf(style).unwrap();
        self.layout.add_child(parent, node).unwrap();
        node
    }
    pub fn add_view<V: View>(&mut self, parent: NodeId, view: Obj<V>, style: Style) -> NodeId {
        let node = self.add_node(parent, style);
        self.views.insert(node, obj_upcast!(view).upgrade());
        node
    }
    pub fn add_view_control<C: View + Control>(
        &mut self,
        parent: NodeId,
        control: Obj<C>,
        style: Style,
    ) -> NodeId {
        let node = self.add_node(parent, style);
        self.views.insert(node, obj_upcast!(control).upgrade());
        self.controls.insert(node, obj_upcast!(control).upgrade());
        node
    }
    pub fn destroy(&mut self, node: NodeId) {
        self.layout.remove(node).unwrap();
        self.views.remove(&node);
        self.controls.remove(&node);
        if self.state.highlight == Some(node) {
            self.state.highlight = None;
        }
    }

    pub fn set_style(&mut self, node: NodeId, style: Style) {
        self.layout.set_style(node, style).unwrap();
    }

    pub fn render(&self, renderer: &mut dyn Renderer) {
        let mut renderer = GuiRenderer::new(renderer);
        self.render_node(&mut renderer, self.root);
    }
    pub fn layout(&mut self) {
        let screen_size = self.state.screen_size;
        let mut root_style = self.layout.style(self.root).unwrap().clone();
        root_style.size = screen_size.map(Dimension::Length);
        self.layout.set_style(self.root, root_style).unwrap();
        self.layout
            .compute_layout(self.root, screen_size.map(AvailableSpace::Definite))
            .unwrap();
    }
    pub fn handle_pointer_motion(&mut self, x: f32, y: f32) {
        self.state.pointer = Point { x, y };
        let highlight = self.hit_highlightable_node(self.root, x, y);
        if highlight != self.state.highlight {
            if let Some(node) = self.state.highlight {
                if let Some(widget) = self.controls.get(&node) {
                    widget.get_mut().handle_pointer(PointerState::None);
                }
            }
            if let Some(node) = highlight {
                if let Some(widget) = self.controls.get(&node) {
                    widget.get_mut().handle_pointer(if self.state.pointer_down {
                        PointerState::Press
                    } else {
                        PointerState::Over
                    });
                }
            }
            self.state.highlight = highlight;
        }
    }
    pub fn handle_pointer_button(&mut self, pressed: bool) {
        if self.state.pointer_down == pressed {
            return;
        }
        if let Some(node) = self.state.highlight {
            if let Some(widget) = self.controls.get(&node) {
                widget.get_mut().handle_pointer(if pressed {
                    PointerState::Press
                } else {
                    PointerState::Over
                });
            }
        }
        self.state.pointer_down = pressed;
    }

    fn render_node(&self, renderer: &mut GuiRenderer, node: NodeId) {
        let layout = self.layout.layout(node).unwrap();
        renderer.push_translation();
        renderer.translate(layout.location.x, layout.location.y);

        if let Some(view) = self.views.get(&node) {
            renderer.set_size(layout.size);
            renderer.set_color(LinSrgba::new(1., 1., 1., 1.));
            view.get().render(renderer);
        }

        for child in self.layout.child_ids(node) {
            self.render_node(renderer, child);
        }

        renderer.pop_translation();
    }

    fn hit_highlightable_node(&self, node: NodeId, mut x: f32, mut y: f32) -> Option<NodeId> {
        let layout = self.layout.layout(node).unwrap();
        x -= layout.location.x;
        y -= layout.location.y;
        if x >= 0.0 && y >= 0.0 && x < layout.size.width && y < layout.size.height {
            for child in self.layout.children(node).unwrap().into_iter().rev() {
                if let Some(hit_node) = self.hit_highlightable_node(child, x, y) {
                    return Some(hit_node);
                }
            }
            if self.controls.contains_key(&node) {
                return Some(node);
            }
        }
        None
    }
}
