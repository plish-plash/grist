mod render;
pub mod widget;

use grist::{obj_upcast, Obj};
use std::collections::HashMap;
use taffy::{geometry::Point, prelude::*};

use widget::Widget;

pub use render::*;
pub use taffy::{self, NodeId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerState {
    None,
    Over,
    Press,
}

pub struct Node {
    visual: Option<Visual>,
}

impl Node {
    fn can_highlight(&self) -> bool {
        self.visual
            .as_ref()
            .map(|vis| vis.background.is_some())
            .unwrap_or(false)
    }
}

#[derive(Default)]
pub struct GuiState {
    highlight: Option<NodeId>,
    pointer_down: bool,
}

pub struct Gui {
    state: GuiState,
    layout: TaffyTree<()>,
    root: NodeId,
    nodes: HashMap<NodeId, Node>,
    widgets: HashMap<NodeId, Obj<dyn Widget>>,
}

impl Gui {
    pub fn new() -> Self {
        let mut layout = TaffyTree::new();
        let root = layout.new_leaf(Style::DEFAULT).unwrap();
        Gui {
            state: Default::default(),
            layout,
            root,
            nodes: HashMap::new(),
            widgets: HashMap::new(),
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn create_node(&mut self, style: Style, visual: Option<Visual>) -> NodeId {
        let node = self.layout.new_leaf(style).unwrap();
        self.nodes.insert(node, Node { visual });
        node
    }
    pub fn create_widget<W, F>(&mut self, style: Style, visual: Option<Visual>, f: F) -> Obj<W>
    where
        W: Widget,
        F: FnOnce(NodeId) -> W,
    {
        let node = self.create_node(style, visual);
        let widget = Obj::new(f(node));
        self.widgets.insert(node, obj_upcast!(widget).upgrade());
        widget
    }
    pub fn destroy_node(&mut self, node: NodeId) {
        self.layout.remove(node).unwrap();
        self.nodes.remove(&node);
        self.widgets.remove(&node);
        if self.state.highlight == Some(node) {
            self.state.highlight = None;
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

    pub fn add_widget<W, F>(&mut self, parent: NodeId, widget: Obj<W>, f: F)
    where
        W: Widget,
        F: FnOnce(&mut W, Style) -> Style,
    {
        let mut widget_guard = widget.get_mut();
        let widget_node = widget_guard.node();
        let style = self.layout.style(widget_node).unwrap().clone();
        let style = f(&mut widget_guard, style);
        self.set_style(widget_node, style);
        self.add_child(parent, widget_node);
    }

    pub fn set_style(&mut self, node: NodeId, style: Style) {
        self.layout.set_style(node, style).unwrap();
    }
    pub fn set_visual(&mut self, node: NodeId, visual: Option<Visual>) {
        let node = self.nodes.get_mut(&node).unwrap();
        node.visual = visual;
    }

    pub fn render(&mut self, renderer: &mut dyn Renderer) {
        let mut renderer = GuiRenderer::new(renderer);
        self.render_node(&mut renderer, self.root);
    }
    pub fn layout(&mut self, width: f32, height: f32) {
        let mut root_style = self.layout.style(self.root).unwrap().clone();
        root_style.size = Size::from_lengths(width, height);
        self.layout.set_style(self.root, root_style).unwrap();
        self.layout
            .compute_layout(
                self.root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
            )
            .unwrap();
    }
    pub fn handle_pointer_motion(&mut self, x: f32, y: f32) {
        let highlight = self.hit_highlightable_node(self.root, x, y);
        if highlight != self.state.highlight {
            if let Some(node) = self.state.highlight {
                if let Some(widget) = self.widgets.get(&node).cloned() {
                    widget.get_mut().handle_pointer(self, PointerState::None);
                }
            }
            if let Some(node) = highlight {
                if let Some(widget) = self.widgets.get(&node).cloned() {
                    widget.get_mut().handle_pointer(
                        self,
                        if self.state.pointer_down {
                            PointerState::Press
                        } else {
                            PointerState::Over
                        },
                    );
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
            if let Some(widget) = self.widgets.get(&node).cloned() {
                widget.get_mut().handle_pointer(
                    self,
                    if pressed {
                        PointerState::Press
                    } else {
                        PointerState::Over
                    },
                );
            }
        }
        self.state.pointer_down = pressed;
    }

    fn render_node(&self, renderer: &mut GuiRenderer, node: NodeId) {
        let layout = self.layout.layout(node).unwrap();
        renderer.save();
        renderer.translate(layout.location.x, layout.location.y);

        if let Some(visual) = self.nodes.get(&node).and_then(|n| n.visual.as_ref()) {
            if let Some(background) = visual.background {
                renderer.set_color(background);
                renderer.draw_rect(Point::ZERO, layout.size);
            }
            if let Some(border) = visual.border {
                let border_size = self.layout.style(node).unwrap().border;
                let border_size = border_size.map(|val| match val {
                    LengthPercentage::Length(length) => length,
                    LengthPercentage::Percent(_) => 0.,
                });
                if border_size.left > 0.
                    || border_size.right > 0.
                    || border_size.top > 0.
                    || border_size.bottom > 0.
                {
                    renderer.set_color(border);
                    renderer.draw_border(layout.size, border_size);
                }
            }
            if let Some(foreground) = visual.foreground {
                if let Some(widget) = self.widgets.get(&node) {
                    renderer.set_color(foreground);
                    widget.get().render(renderer, layout.size);
                }
            }
        }

        for child in self.layout.child_ids(node) {
            self.render_node(renderer, child);
        }

        renderer.restore();
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
            if self
                .nodes
                .get(&node)
                .map(|n| n.can_highlight())
                .unwrap_or(false)
            {
                return Some(node);
            }
        }
        None
    }
}
