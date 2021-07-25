use crate::{
    datastructure::generic::Vec2i,
    opengl::{
        rect::RectRenderer,
        text::TextRenderer,
        types::{RGBAColor, RGBColor},
    },
};

use super::{boundingbox::BoundingBox, coordinate::Size};
use crate::textbuffer::metadata::{Column, Line};

#[derive(Debug)]
pub enum StatusBarContent<'a> {
    FileEdit(Option<&'a std::path::PathBuf>, (Line, Column)),
    Message(Vec<char>),
}

impl<'a> StatusBarContent<'a> {
    pub fn to_str(&self) -> String {
        match self {
            StatusBarContent::FileEdit(path, (line, column)) => {
                format!("{}:{}:{}", path.map(|p| p.display().to_string()).unwrap_or("unnamed_file".into()), **line, **column)
            }
            StatusBarContent::Message(msg) => msg.iter().collect(),
        }
    }
}

pub struct StatusBar<'app> {
    pub text_renderer: TextRenderer<'app>,
    pub window_renderer: RectRenderer,
    pub size: Size,
    pub anchor: Vec2i,
    pub display_data: StatusBarContent<'app>,
    pub bg_color: RGBAColor,
    pub needs_update: bool,
}

impl<'app> StatusBar<'app> {
    pub fn new(text_renderer: TextRenderer<'app>, window_renderer: RectRenderer, anchor: Vec2i, size: Size, bg_color: RGBAColor) -> StatusBar<'app> {
        StatusBar {
            text_renderer,
            window_renderer,
            size,
            anchor,
            display_data: StatusBarContent::Message("<cxgledit>".chars().into_iter().collect()),
            bg_color,
            needs_update: true,
        }
    }

    pub fn update_text_content(&mut self, bar_content: StatusBarContent<'app>) {
        self.display_data = bar_content;
        self.needs_update = true;
    }

    pub fn update_string_contents(&mut self, data: &str) {
        self.display_data = StatusBarContent::Message(data.chars().map(|c| c).collect());
    }

    pub fn update(&mut self) {
        let Vec2i { x, y } = self.anchor;
        self.window_renderer.clear_data();
        self.text_renderer.clear_data();

        self.window_renderer
            .add_rect(BoundingBox::from((self.anchor, self.size)), self.bg_color);
        let t: Vec<_> = self.display_data.to_str().chars().map(|c| c).collect();
        let color = RGBColor { r: 1.0f32, g: 1.0, b: 1.3 };
        let font = self.text_renderer.font;
        self.text_renderer.push_draw_command(t.iter().map(|c| *c), color, x, y, font);
    }

    pub fn draw(&mut self) {
        if self.needs_update {
            self.update();
        }
        self.window_renderer.draw();
        self.text_renderer.draw_list();
    }
}
