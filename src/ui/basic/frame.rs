use crate::datastructure::generic::Vec2i;

use super::{
    boundingbox::BoundingBox,
    coordinate::{Anchor, Margin, PointArithmetic, Size},
};

#[derive(Copy, Clone, Debug)]
pub struct Frame {
    pub anchor: Anchor,
    pub size: Size,
}

impl Frame {
    pub fn new(anchor: Anchor, size: Size) -> Frame {
        Frame { anchor, size }
    }

    pub fn to_bb(&self) -> BoundingBox {
        BoundingBox::from_frame(&self)
    }
}

pub fn make_inner_frame(outer_frame: &Frame, margin: i32) -> Frame {
    let size = Size::shrink_axis_aligned(outer_frame.size, Margin::Perpendicular { h: margin, v: margin });
    let anchor = Anchor::vector_add(outer_frame.anchor, Vec2i::new(margin, -margin));
    Frame { anchor, size }
}
