use super::{
    coordinate::{Margin, Size},
    frame::Frame,
};
use crate::datastructure::generic::{Vec2f, Vec2i};

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
        // crate::debugger_catch!(min.x <= max.x && min.y <= max.y, crate::DebuggerCatch::Handle(format!("Assertion failed for {:?} <= {:?}", min, max)));
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

    /// Aligns this bounding box on a position, on the center of the bounding box
    /// * `pos` - the position which the bounding box's center point should snap to
    pub fn center_align_around(&mut self, pos: Vec2i) {
        let min = pos + Vec2i::new(self.width() / -2, self.height() / -2);
        let max = pos + Vec2i::new(self.width() / 2, self.height() / 2);
        self.min = min;
        self.max = max;
    }

    pub fn center_horizontal_align(&mut self, x: i32) {
        let min = Vec2i::new(x, self.min.y) + Vec2i::new(self.width() / -2, 0);
        let max = Vec2i::new(x, self.max.y) + Vec2i::new(self.width() / 2, 0);
        self.min = min;
        self.max = max;
    }

    pub fn center_vertical_align(&mut self, y: i32) {
        let min = Vec2i::new(self.min.x, y) + Vec2i::new(0, self.height() / -2);
        let max = Vec2i::new(self.max.x, y) + Vec2i::new(0, self.height() / 2);
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

    pub fn size_f32(&self) -> Vec2f {
        Vec2f::new(self.width() as f32, self.height() as f32)
    }

    pub fn height(&self) -> i32 {
        self.max.y - self.min.y
    }
    pub fn width(&self) -> i32 {
        self.max.x - self.min.x
    }

    pub fn center_pos(&self) -> Vec2i {
        let sz = self.size();
        let add = Vec2i::new(sz.width / 2, sz.height / 2);
        self.min + add
    }

    pub fn translate(&self, vec: Vec2i) -> BoundingBox {
        let mut bb = self.clone();
        bb.min += vec;
        bb.max += vec;
        bb
    }

    pub fn translate_mut(mut self, vec: Vec2i) -> BoundingBox {
        self.min += vec;
        self.max += vec;
        self
    }
}

impl From<(Vec2i, Size)> for BoundingBox {
    #[inline(always)]
    fn from(tuple: (Vec2i, Size)) -> Self {
        let (Vec2i { x, y }, size) = tuple;
        BoundingBox::new(Vec2i::new(x, y - size.height), Vec2i::new(x + size.width, y))
    }
}
