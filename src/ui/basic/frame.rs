use crate::datastructure::generic::Vec2i;

use super::{
    boundingbox::BoundingBox,
    coordinate::{Coordinate, Margin, Size},
};

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub anchor: Vec2i,
    pub size: Size,
}

impl Frame {
    pub fn new(anchor: Vec2i, size: Size) -> Frame {
        Frame { anchor, size }
    }

    pub fn to_bb(&self) -> BoundingBox {
        BoundingBox::from_frame(&self)
    }

    pub fn from_bb(bb: &BoundingBox) -> Frame {
        Frame { anchor: Vec2i::new(bb.min.x, bb.max.y), size: Size::new(bb.max.x - bb.min.x, bb.max.y - bb.min.y) }
    }

    pub fn height(&self) -> i32 {
        self.size.height
    }

    pub fn width(&self) -> i32 {
        self.size.width
    }
}

pub fn make_inner_frame(outer_frame: &Frame, margin: i32) -> Frame {
    let size = Size::shrink_axis_aligned(outer_frame.size, Margin::Perpendicular { h: margin, v: margin });
    let anchor = outer_frame.anchor + Vec2i::new(margin, -margin);
    Frame { anchor, size }
}
