use crate::opengl::{rect::RectRenderer, text::TextRenderer, types::RGBAColor};

use super::{
    boundingbox::BoundingBox,
    coordinate::{Anchor, Size},
};
use crate::textbuffer::metadata::{Column, Line};

#[derive(Debug)]
pub enum StatusBarContent<'a> {
    FileEdit(&'a std::path::Path, (Line, Column)),
    Message(Vec<char>),
}

impl<'a> StatusBarContent<'a> {
    pub fn to_str(&self) -> String {
        match self {
            StatusBarContent::FileEdit(path, (line, column)) => {
                format!("{}:{}:{}", path.display(), **line, **column)
            }
            StatusBarContent::Message(msg) => msg.iter().collect(),
        }
    }
}

pub struct StatusBar<'app> {
    pub text_renderer: TextRenderer<'app>,
    pub window_renderer: RectRenderer,
    pub size: Size,
    pub anchor: Anchor,
    pub display_data: StatusBarContent<'app>,
    pub bg_color: RGBAColor,
}

impl<'app> StatusBar<'app> {
    pub fn new(text_renderer: TextRenderer<'app>, window_renderer: RectRenderer, anchor: Anchor, size: Size, bg_color: RGBAColor) -> StatusBar<'app> {
        StatusBar {
            text_renderer,
            window_renderer,
            size,
            anchor,
            display_data: StatusBarContent::Message("<cxgledit>".chars().into_iter().collect()),
            bg_color,
        }
    }

    pub fn update_text_content(&mut self, bar_content: StatusBarContent<'app>) {
        self.display_data = bar_content;
    }

    pub fn update_string_contents(&mut self, data: &str) {
        self.display_data = StatusBarContent::Message(data.chars().map(|c| c).collect());
    }

    pub fn update(&mut self) {
        let Anchor(x, y) = self.anchor;
        self.window_renderer.clear_data();
        self.window_renderer
            .add_rect(BoundingBox::from((self.anchor, self.size)), self.bg_color);
        let t: Vec<_> = self.display_data.to_str().chars().map(|c| c).collect();
        self.text_renderer.prepare_data_iter(t.iter(), x, y);
    }

    pub fn draw(&mut self) {
        self.window_renderer.draw();
        self.text_renderer.draw();
    }
}
