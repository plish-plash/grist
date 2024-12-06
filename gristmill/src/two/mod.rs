mod quad;
mod sprite;

use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

pub use quad::*;
pub use sprite::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Copy, Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        position: Vec2::ZERO,
        size: Vec2::ZERO,
    };
    pub const ONE: Rect = Rect {
        position: Vec2::ZERO,
        size: Vec2::ONE,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }
    pub fn from_size(size: Vec2) -> Self {
        Rect {
            position: Vec2::ZERO,
            size,
        }
    }
    pub fn from_anchor(size: Vec2, anchor: Anchor, anchor_pos: Vec2) -> Rect {
        match anchor {
            Anchor::TopLeft => Rect {
                position: anchor_pos,
                size,
            },
            Anchor::TopCenter => Rect {
                position: Vec2::new(anchor_pos.x - (size.x / 2.), anchor_pos.y),
                size,
            },
            Anchor::TopRight => Rect {
                position: Vec2::new(anchor_pos.x - size.x, anchor_pos.y),
                size,
            },
            Anchor::MiddleLeft => Rect {
                position: Vec2::new(anchor_pos.x, anchor_pos.y - (size.y / 2.)),
                size,
            },
            Anchor::Center => Rect {
                position: Vec2::new(anchor_pos.x - (size.x / 2.), anchor_pos.y - (size.y / 2.)),
                size,
            },
            Anchor::MiddleRight => Rect {
                position: Vec2::new(anchor_pos.x - size.x, anchor_pos.y - (size.y / 2.)),
                size,
            },
            Anchor::BottomLeft => Rect {
                position: Vec2::new(anchor_pos.x, anchor_pos.y - size.y),
                size,
            },
            Anchor::BottomCenter => Rect {
                position: Vec2::new(anchor_pos.x - (size.x / 2.), anchor_pos.y - size.y),
                size,
            },
            Anchor::BottomRight => Rect {
                position: Vec2::new(anchor_pos.x - size.x, anchor_pos.y - size.y),
                size,
            },
        }
    }

    pub fn x(&self) -> f32 {
        self.position.x
    }
    pub fn y(&self) -> f32 {
        self.position.y
    }
    pub fn width(&self) -> f32 {
        self.size.x
    }
    pub fn height(&self) -> f32 {
        self.size.y
    }
    pub fn aspect(&self) -> f32 {
        self.width() / self.height()
    }
    pub fn get_anchor(&self, anchor: Anchor) -> Vec2 {
        match anchor {
            Anchor::TopLeft => self.position,
            Anchor::TopCenter => Vec2::new(self.position.x + (self.size.x / 2.), self.position.y),
            Anchor::TopRight => Vec2::new(self.position.x + self.size.x, self.position.y),
            Anchor::MiddleLeft => Vec2::new(self.position.x, self.position.y + (self.size.y / 2.)),
            Anchor::Center => self.position + (self.size / 2.0),
            Anchor::MiddleRight => Vec2::new(
                self.position.x + self.size.x,
                self.position.y + (self.size.y / 2.),
            ),
            Anchor::BottomLeft => Vec2::new(self.position.x, self.position.y + self.size.y),
            Anchor::BottomCenter => Vec2::new(
                self.position.x + (self.size.x / 2.),
                self.position.y + self.size.y,
            ),
            Anchor::BottomRight => {
                Vec2::new(self.position.x + self.size.x, self.position.y + self.size.y)
            }
        }
    }

    pub fn contains(&self, position: Vec2) -> bool {
        position.x >= self.position.x
            && position.x < self.position.x + self.size.x
            && position.y >= self.position.y
            && position.y < self.position.y + self.size.y
    }

    pub fn grow(mut self, amount: f32) -> Rect {
        self.position.x -= amount;
        self.position.y -= amount;
        self.size.x += amount * 2.;
        self.size.y += amount * 2.;
        self
    }
    pub fn shrink(self, amount: f32) -> Rect {
        self.grow(-amount)
    }

    pub fn as_irect(&self) -> IRect {
        IRect {
            position: self.position.as_ivec2(),
            size: self.size.as_ivec2(),
        }
    }
}

impl From<[f32; 4]> for Rect {
    fn from(rect: [f32; 4]) -> Self {
        Rect {
            position: Vec2::new(rect[0], rect[1]),
            size: Vec2::new(rect[2], rect[3]),
        }
    }
}
impl From<glyph_brush::ab_glyph::Rect> for Rect {
    fn from(rect: glyph_brush::ab_glyph::Rect) -> Self {
        Rect {
            position: Vec2::new(rect.min.x, rect.min.y),
            size: Vec2::new(rect.width(), rect.height()),
        }
    }
}
impl From<Rect> for [f32; 4] {
    fn from(rect: Rect) -> Self {
        [rect.position.x, rect.position.y, rect.size.x, rect.size.y]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct IRect {
    pub position: IVec2,
    pub size: IVec2,
}

impl IRect {
    pub const ZERO: IRect = IRect {
        position: IVec2::ZERO,
        size: IVec2::ZERO,
    };

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        IRect {
            position: IVec2::new(x, y),
            size: IVec2::new(width, height),
        }
    }
    pub fn from_size(size: IVec2) -> Self {
        IRect {
            position: IVec2::ZERO,
            size,
        }
    }

    pub fn x(&self) -> i32 {
        self.position.x
    }
    pub fn y(&self) -> i32 {
        self.position.y
    }
    pub fn width(&self) -> i32 {
        self.size.x
    }
    pub fn height(&self) -> i32 {
        self.size.y
    }
    pub fn center(&self) -> IVec2 {
        self.position + (self.size / 2)
    }

    pub fn contains(&self, point: IVec2) -> bool {
        self.position.x <= point.x
            && self.position.y <= point.y
            && self.position.x + self.size.x > point.x
            && self.position.y + self.size.y > point.y
    }

    pub fn inset(&self, insets: EdgeRect) -> IRect {
        let inset_width = insets.left + insets.right;
        let inset_height = insets.top + insets.bottom;
        IRect {
            position: IVec2 {
                x: self.position.x + insets.left,
                y: self.position.y + insets.top,
            },
            size: IVec2 {
                x: self.size.x - inset_width,
                y: self.size.y - inset_height,
            },
        }
    }

    pub fn add_components(&self, other: IRect) -> Self {
        IRect {
            position: self.position + other.position,
            size: self.size + other.size,
        }
    }

    pub fn as_rect(&self) -> Rect {
        Rect {
            position: self.position.as_vec2(),
            size: self.size.as_vec2(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct EdgeRect {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl EdgeRect {
    pub const ZERO: EdgeRect = EdgeRect {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        EdgeRect {
            top,
            right,
            bottom,
            left,
        }
    }
    pub fn splat(v: i32) -> Self {
        EdgeRect {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
}
