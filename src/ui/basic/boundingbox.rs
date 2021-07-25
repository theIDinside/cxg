use super::{
    coordinate::{Margin, Size},
    frame::Frame,
};
use crate::datastructure::generic::Vec2i;

#[derive(Debug, Clone)]
pub struct BoundingBox {
    /// Bottom left corner
    pub min: Vec2i,
    /// Top right corner
    pub max: Vec2i,
}

impl BoundingBox {
    /// Create a new bounding box. No assertions or checks is done to verify that min <= max. User must take responsibility for that
    pub fn new(min: Vec2i, max: Vec2i) -> BoundingBox {
        debug_assert!(min.x <= max.x && min.y <= max.y, "Assertion failed for {:?} <= {:?}", min, max);
        BoundingBox { min, max }
    }

    /// Checks if parameter pos, is inside the bounding box coordinate space
    pub fn box_hit_check(&self, pos: Vec2i) -> bool {
        pos.x >= self.min.x && pos.y >= self.min.y && pos.x <= self.max.x && pos.y <= self.max.y
    }

    pub fn from_info(anchor: Vec2i, size: Size) -> BoundingBox {
        BoundingBox::from((anchor, size))
    }

    #[inline(always)]
    pub fn from_frame(frame: &Frame) -> BoundingBox {
        let (Vec2i { x, y }, Size { width, height }) = (frame.anchor, frame.size);
        BoundingBox::new(Vec2i::new(x, y - height), Vec2i::new(x + width, y))
    }

    pub fn center_align_around(&mut self, pos: Vec2i) {
        let min = pos + Vec2i::new(self.width() / -2, self.height() / -2);
        let max = pos + Vec2i::new(self.width() / 2, self.height() / 2);
        self.min = min;
        self.max = max;
    }

    pub fn shrink(bounding_box: &BoundingBox, margin: Margin) -> BoundingBox {
        let mut b = bounding_box.clone();
        match margin {
            Margin::Vertical(margin) => {
                b.min.y += margin;
                b.max.y -= margin;
            }
            Margin::Horizontal(margin) => {
                b.min.x += margin;
                b.max.x -= margin;
            }
            Margin::Perpendicular { h: horizontal, v: vertical } => {
                b.min.y += vertical;
                b.max.y -= vertical;
                b.min.x += horizontal;
                b.max.x -= horizontal;
            }
        }
        b
    }

    pub fn expand(bounding_box: &BoundingBox, margin: Margin) -> BoundingBox {
        let mut b = bounding_box.clone();
        match margin {
            Margin::Vertical(margin) => {
                b.min.y -= margin;
                b.max.y += margin;
            }
            Margin::Horizontal(margin) => {
                b.min.x -= margin;
                b.max.x += margin;
            }
            Margin::Perpendicular { h: horizontal, v: vertical } => {
                b.min.y -= vertical;
                b.max.y += vertical;
                b.min.x -= horizontal;
                b.max.x += horizontal;
            }
        }
        b
    }

    pub fn size(&self) -> Size {
        Size { width: self.width(), height: self.height() }
    }

    pub fn height(&self) -> i32 {
        self.max.y - self.min.y
    }
    pub fn width(&self) -> i32 {
        self.max.x - self.min.x
    }
}

impl From<(Vec2i, Size)> for BoundingBox {
    #[inline(always)]
    fn from(tuple: (Vec2i, Size)) -> Self {
        let (Vec2i { x, y }, size) = tuple;
        BoundingBox::new(Vec2i::new(x, y - size.height), Vec2i::new(x + size.width, y))
    }
}
