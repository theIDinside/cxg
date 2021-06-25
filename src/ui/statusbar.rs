use crate::opengl::{rect::RectRenderer, text::TextRenderer};

use super::coordinate::{Anchor, Size};


pub struct StatusBar<'app> {
    pub text_renderer: TextRenderer<'app>,
    pub window_renderer: RectRenderer,
    pub size: Size,
    pub anchor: Anchor,
    pub display_data: String,
    needs_rerender: bool,
}


impl<'app> StatusBar<'app> {
    pub fn new(text_renderer: TextRenderer<'app>, window_renderer: RectRenderer, anchor: Anchor, size: Size) -> StatusBar<'app> {
        StatusBar {
            text_renderer,
            window_renderer,
            size,
            anchor,
            display_data: String::new(),
            needs_rerender: true,
        }
    }

    pub fn update_string_contents(&mut self, data: &str) {
        self.display_data.clear();
        self.display_data.push_str(data);
        self.needs_rerender = true;
    }

    pub fn update(&mut self) {
        let Anchor(x, y) = self.anchor;
        self.window_renderer.update_rectangle(self.anchor, self.size);
        self.text_renderer.draw_text(&self.display_data, x, y);
    }

    pub fn draw(&self) {
        self.window_renderer.draw();
        self.text_renderer.draw();
    }
}