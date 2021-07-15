use crate::opengl::{rect::RectRenderer, text::TextRenderer, types::RGBAColor};

use super::coordinate::{Anchor, Size};

pub struct StatusBar<'app> {
    pub text_renderer: TextRenderer<'app>,
    pub window_renderer: RectRenderer,
    pub size: Size,
    pub anchor: Anchor,
    pub display_data: Vec<char>,
    pub bg_color: RGBAColor
}

impl<'app> StatusBar<'app> {
    pub fn new(text_renderer: TextRenderer<'app>, window_renderer: RectRenderer, anchor: Anchor, size: Size, bg_color: RGBAColor) -> StatusBar<'app> {
        StatusBar {
            text_renderer,
            window_renderer,
            size,
            anchor,
            display_data: vec![],
            bg_color
        }
    }

    pub fn update_string_contents(&mut self, data: &str) {
        self.display_data.clear();
        self.display_data = data.chars().map(|c| c).collect();
    }

    pub fn update(&mut self) {
        let Anchor(x, y) = self.anchor;
        self.window_renderer.update_rectangle(self.anchor, self.size, self.bg_color);
        self.text_renderer.push_data(&self.display_data, x, y);
        self.text_renderer.draw();
    }

    pub fn draw(&self) {
        self.window_renderer.draw();
        self.text_renderer.draw();
    }
}
