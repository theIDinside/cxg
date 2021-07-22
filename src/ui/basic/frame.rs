use crate::datastructure::generic::Vec2i;

use super::coordinate::{Anchor, Margin, PointArithmetic, Size};

#[derive(Copy, Clone)]
pub struct Frame {
    pub anchor: Anchor,
    pub size: Size,
}

pub fn make_inner_frame(outer_frame: &Frame, margin: i32) -> Frame {
    let size = Size::shrink_axis_aligned(outer_frame.size, Margin::Perpendicular { h: margin, v: margin });
    let anchor = Anchor::vector_add(outer_frame.anchor, Vec2i::new(margin, -margin));
    Frame { anchor, size }
}
