use crate::datastructure::generic::Vec2i;

use super::basic::frame::Frame;

pub enum ScrollBarLayout {
    Horizontal,
    Vertical,
}

/// Scroll bar UI Element
pub struct ScrollBar {
    /// The visual frame of this UI Element
    pub frame: Frame,
    /// The actual sliding block
    pub slider: Frame,
    /// The range of values this slider slides beween
    pub max: usize,
    /// The layout of the slider/scroll bar
    pub layout: ScrollBarLayout,

    pub scroll_value: usize,
}

impl ScrollBar {
    pub fn ui_update(&mut self) {
        let percent = self.scroll_value as f32 / self.max as f32;
        match self.layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                self.slider.size.height = std::cmp::max(15, self.frame.size.height / self.max as i32);
                let block_center_y = self.frame.anchor.y - (percent as f32 * self.frame.size.height as f32) as i32;
                let mut tmp_bb = super::basic::boundingbox::BoundingBox::from_frame(&self.slider);
                // tmp_bb.center_vertical_align(block_center_y);
                self.slider = Frame::from_bb(&tmp_bb);
                self.slider.anchor.x = self.frame.anchor.x;
            }
        }
    }

    pub fn new(frame: Frame, end: usize, layout: ScrollBarLayout, scroll_value: usize) -> ScrollBar {
        let step = end / frame.size.height as usize;
        let mut slider = frame.clone();
        match layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                slider.size.height = frame.size.height / end as i32;
            }
        }
        ScrollBar { frame, slider, max: end, layout, scroll_value }
    }

    // Only use this function when we've validated that pos is inside this objects frame. otherwise, blame yourself
    pub fn scroll_to_ui_pos(&mut self, pos: Vec2i) {
        match self.layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                let percent = (self.frame.anchor.y - pos.y) as f64 / self.frame.size.height as f64;
                self.slider.anchor.y = pos.y.clamp(0 + self.slider.size.height, self.frame.anchor.y);
                self.scroll_value = ((self.max as f64 * percent).floor() as usize).clamp(0, self.max);
                self.ui_update();
            }
        }
    }

    pub fn update_ui_position_by_value(&mut self) {
        match self.layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                let percent = self.scroll_value as f64 / self.max as f64;
                self.slider.anchor.y = self.frame.anchor.y - (percent * self.frame.height() as f64) as i32;
            }
        }
    }
}
