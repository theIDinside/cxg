use crate::datastructure::generic::Vec2i;

use super::basic::frame::Frame;
// FIXME: fix so that when clicking a scroll bar, it doesn't snap it's top to the mouse cursor
#[derive(Debug)]
pub enum ScrollBarLayout {
    Horizontal,
    Vertical,
}

/// Scroll bar UI Element
#[derive(Debug)]
pub struct ScrollBar {
    /// The visual frame of this UI Element
    pub frame: Frame,
    /// The actual sliding block
    pub slider: Frame,
    /// The layout of the slider/scroll bar
    pub layout: ScrollBarLayout,
    /// The range of values this slider slides beween
    pub max: usize,
    pub scroll_value: usize,
}

impl ScrollBar {
    pub fn ui_update(&mut self) {
        match self.layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                self.slider.size.height = std::cmp::max(35, self.frame.size.height / self.max as i32);
                self.slider.anchor.x = self.frame.anchor.x;
            }
        }
    }

    pub fn new(frame: Frame, end: usize, layout: ScrollBarLayout, scroll_value: usize) -> ScrollBar {
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
                let len = self.scrollable_length_pixels();
                if len > 1 {
                    self.slider.anchor.y = pos.y.clamp(0 + self.slider.size.height, self.frame.anchor.y);
                    let percent = (len - (self.slider.anchor.y - self.slider.height())) as f32 / len as f32;
                    self.scroll_value = ((self.max as f32 * percent).floor() as usize).clamp(0, self.max);
                    self.ui_update();
                }
            }
        }
    }

    pub fn scroll_by(&mut self, pixels: i32) {
        match self.layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                let len = self.scrollable_length_pixels();
                if len > 1 {
                    self.slider.anchor.y = (self.slider.anchor.y + pixels).clamp(0 + self.slider.size.height, self.frame.anchor.y);
                    let percent = (len - (self.slider.anchor.y - self.slider.height())) as f32 / len as f32;
                    println!("Percentage: {}", percent);
                    println!("(scroll_by) Percentage scrolled: {}", percent);
                    self.scroll_value = ((self.max as f32 * percent).floor() as usize).clamp(0, self.max);
                    self.ui_update();
                }
            }
        }
    }

    pub fn set_max(&mut self, max_value: usize) {
        self.max = max_value;
        self.slider.size.height = std::cmp::max(35, self.frame.size.height / self.max as i32);
    }

    pub fn update_ui_position_by_value(&mut self) {
        match self.layout {
            ScrollBarLayout::Horizontal => todo!(),
            ScrollBarLayout::Vertical => {
                self.slider.size.height = std::cmp::max(35, self.frame.size.height / self.max as i32);
                self.slider.anchor.x = self.frame.anchor.x;
                let percent = (self.scroll_value as f64 / self.max as f64).clamp(0.0, 1.0);
                println!("(update_ui_position_by_value) Percentage scrolled: {}", percent);
                let len = self.scrollable_length_pixels() as f64;
                let tmp = self.frame.anchor.y - (percent * len) as i32;
                self.slider.anchor.y = tmp.clamp(0 + self.slider.height(), self.frame.anchor.y);
            }
        }
    }

    pub fn scrollable_length_pixels(&self) -> i32 {
        self.frame.height() - self.slider.height()
    }

    pub fn debug(&self) {
        let len = self.scrollable_length_pixels();
        let percent = (len - (self.slider.anchor.y - self.slider.height())) as f32 / len as f32;
        println!("Scroll {}%", percent * 100.0);
    }
}
