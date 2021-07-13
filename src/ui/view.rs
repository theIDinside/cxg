use crate::opengl::rect::RectRenderer;
use crate::opengl::text::TextRenderer;
use crate::opengl::types::RGBAColor;
use crate::textbuffer::simple::simplebuffer::SimpleBuffer;
use crate::textbuffer::{CharBuffer, Movement, TextKind};
use crate::textbuffer::metadata::{Index, Line};
use crate::ui::font::Font;
use super::boundingbox::BoundingBox;
use super::coordinate::{Anchor, Size};

use crate::opengl::{Renderable};
use crate::ui::coordinate::Coordinate;
use std::fmt::Formatter;

pub trait Viewable {
    fn set_anchor(&mut self, anchor: Anchor);
    fn resize(&mut self, width: i32, height: i32);
    fn update(&mut self, renderable: Box<dyn Renderable>);
}

#[derive(PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct ViewId(pub u32);

impl std::ops::Deref for ViewId {
    type Target = u32;

    fn deref<'a>(&'a self) -> &'a u32 {
        &self.0
    }
}

impl Into<ViewId> for u32 {
    fn into(self) -> ViewId {
        ViewId(self)
    }
}

pub struct View<'a> {
    pub name: String,
    pub id: ViewId,
    pub text_renderer: TextRenderer<'a>,
    pub window_renderer: RectRenderer,
    pub cursor_renderer: RectRenderer,
    pub buffer_id: u32,
    pub size: Size,
    pub anchor: Anchor,
    pub topmost_line_in_buffer: i32,
    displayable_lines: i32,
    row_height: i32,
    panel_id: Option<u32>,
    pub buffer: SimpleBuffer,
    buffer_in_view: std::ops::Range<usize>,
    view_changed: bool,
    cursor_width: i32
}

pub struct Popup<'a> {
    pub visible: bool,
    pub view: View<'a>
}


 impl<'a> std::ops::Deref for Popup<'a> {
    type Target = View<'a>;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<'a> std::fmt::Debug for View<'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("anchor", &self.anchor)
            .field("top buffer line", &self.topmost_line_in_buffer)
            .field("displayable lines", &self.displayable_lines)
            .field("layout by", &self.panel_id)
            .finish()
    }
}

impl<'a> View<'a>{
    pub fn new(name: &str, view_id: ViewId, text_renderer: TextRenderer<'a>, window_renderer: RectRenderer, buffer_id: u32, width: i32, height: i32, row_height: i32) -> View<'a> {
        let cursor_width = text_renderer.get_cursor_width_size();
        let cursor_shader = window_renderer.shader.clone();
        let mut cursor_renderer = RectRenderer::create(cursor_shader, 100).expect("failed to create rectangle renderer");
        cursor_renderer.set_color(RGBAColor{ r: 0.5, g: 0.5, b: 0.5, a: 0.5});
        let mut v = View {
            name: name.to_string(),
            id: view_id,
            text_renderer,
            window_renderer,
            cursor_renderer,
            buffer_id,
            size: Size::new(width, height),
            anchor: Anchor(0, 0),
            topmost_line_in_buffer: 0,
            displayable_lines: height / row_height,
            row_height,
            cursor_width,
            panel_id: None,
            buffer: SimpleBuffer::new(*view_id, 1000),
            buffer_in_view: 0 .. 0,
            view_changed: true
        };
        v.window_renderer.update_rectangle(v.anchor, v.size);
        v
    }

    pub fn watch_buffer(&mut self, id: u32) {
        self.buffer_id = id;   
    }

    pub fn update(&mut self) {
        self.window_renderer.update_rectangle(self.anchor, self.size);
        self.view_changed = true;
    }

    pub fn draw(&mut self) {
        
        let Anchor(top_x, top_y) = self.anchor;
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(top_x, top_y-self.size.height, self.size.width, self.size.height);
        }
        if self.view_changed {
            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            self.text_renderer.push_data(self.buffer.str_view(&self.buffer_in_view), top_x, top_y);    
            let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
            let cols_in = *self.buffer.cursor_col() as i32;
            
            let g = self.text_renderer.get_glyph(*self.buffer.get(self.buffer.cursor_abs()).unwrap_or(&'\0'));
            let cursor_width = g.map(|glyph| {
                if glyph.width() == 0 as _ {
                    glyph.advance
                } else {
                    glyph.width() as _
                }
            }).unwrap_or(self.cursor_width);

            let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
            crate::only_in_debug!(crate::debugger_catch!(nl_buf_idx + (cols_in as usize) <= self.buffer.len(), "range is outside of buffer"));
            let line_contents = self.buffer.get_slice(nl_buf_idx .. (nl_buf_idx + cols_in as usize));

            let min_x = top_x + line_contents.iter().map(|& c| self.text_renderer.get_glyph(c).map(|g| g.advance as _).unwrap_or(cursor_width)).sum::<i32>();

            use crate::datastructure::generic::Vec2i;

            let min = Vec2i::new(min_x, top_y - (rows_down * self.row_height) - self.row_height);
            let max = Vec2i::new(min_x + cursor_width, top_y - (rows_down * self.row_height));

            let bb = BoundingBox::new(min, max);
            self.cursor_renderer.set_rect(bb);
            self.view_changed = false;

        }
        
        self.window_renderer.draw();
        self.text_renderer.bind();
        self.text_renderer.draw();
        // Remember to draw in correct Z-order! We manage our own "layers". Therefore, draw cursor last
        self.cursor_renderer.draw();
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.size.width = width;
        self.size.height = height;
        self.displayable_lines = self.size.height / self.row_height;
    }

    pub fn calc_displayable_lines(&mut self, font: &Font) {
        self.row_height = font.row_height();
        self.displayable_lines = (self.size.height as f32 / font.row_height() as f32).floor() as _;
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        BoundingBox::from((self.anchor, self.size))
    }

    pub fn insert_ch(&mut self, ch: char) {
        self.buffer.insert(ch);
        if self.buffer.cursor_row() >= Line((self.topmost_line_in_buffer + self.displayable_lines) as _) {
            self.adjust_view_range();
        } else {
            self.buffer_in_view.end += 1;
            self.view_changed = true;
        }
    }

    pub fn adjust_view_range(&mut self) {
        let md = self.buffer.meta_data();
        if self.buffer.cursor_row() >= Line((self.topmost_line_in_buffer + self.displayable_lines) as _)  {
            let diff = std::cmp::max((*self.buffer.cursor_row() as i32) - (self.topmost_line_in_buffer + self.displayable_lines) as i32, 1);
            self.topmost_line_in_buffer += diff;
            if let (Some(a), end) = md.get_byte_indices_of_lines(Line(self.topmost_line_in_buffer as _), Line((self.topmost_line_in_buffer + self.displayable_lines) as _ )) {
                self.buffer_in_view = *a .. *end.unwrap_or(Index(self.buffer.len()));
            }

            self.view_changed = true;
        } else if self.buffer.cursor_row() < Line(self.topmost_line_in_buffer as _) {
            self.topmost_line_in_buffer = *self.buffer.cursor_row() as _;
            if let (Some(a), end) = md.get_byte_indices_of_lines(Line(self.topmost_line_in_buffer as _), Line((self.topmost_line_in_buffer + self.displayable_lines) as _ )) {
                self.buffer_in_view = *a .. *end.unwrap_or(Index(self.buffer.len()));
            }
        } else {
            if let (Some(a), end) = md.get_byte_indices_of_lines(Line(self.topmost_line_in_buffer as _), Line((self.topmost_line_in_buffer + self.displayable_lines) as _ )) {
                self.buffer_in_view = *a .. *end.unwrap_or(Index(self.buffer.len()));
            }
        }
        self.view_changed = true;
    }

    pub fn insert_slice(&mut self, s: &[char]) {
        self.buffer.insert_slice(s);
        self.text_renderer.pristine = false;
        self.validate_range();
        self.adjust_view_range();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.buffer_in_view = 0 .. s.len();
        for c in s.chars() {
            self.buffer.insert(c);
        }
        self.text_renderer.pristine = false;
        self.adjust_view_range();
    }

    pub fn cursor_goto(&mut self, pos: Index) {
        self.buffer.cursor_goto(pos);
        self.adjust_view_range();
    }
    pub fn move_cursor(&mut self, dir: Movement) {
        match dir {
            Movement::Forward(kind, count) => self.buffer.cursor_move_forward(kind, count),
            Movement::Backward(kind, count) => self.buffer.cursor_move_backward(kind, count),
            Movement::Begin(kind) => {
                match kind {
                    TextKind::Char => todo!(),
                    TextKind::Word => todo!(),
                    TextKind::Line => {
                        if let Some(Index(start)) = self.buffer.meta_data().get(self.buffer.cursor_row()) {
                            self.buffer.cursor_goto(Index(start));
                        }
                    },
                    TextKind::Block => todo!(),
                }
            },
            Movement::End(kind) => {
                match kind {
                    TextKind::Char => todo!(),
                    TextKind::Word => todo!(),
                    TextKind::Line => {
                        if let Some(Index(end)) = self.buffer.meta_data().get(self.buffer.cursor_row() + Line(1)).map(|Index(start)| Index(start-1)) {
                            self.buffer.cursor_goto(Index(end));
                        }
                    },
                    TextKind::Block => todo!(),
                }
            },
        }
        self.adjust_view_range();
    }

    pub fn delete(&mut self, dir: Movement) {
        self.buffer.delete(dir);
        self.view_changed = true;
        self.validate_range();
        self.adjust_view_range();
    }

    pub fn backspace_handle(&mut self, kind: TextKind) {
        match kind {
            TextKind::Char => self.buffer.delete(Movement::Backward(TextKind::Char, 1)),
            TextKind::Word => self.buffer.delete(Movement::Backward(TextKind::Word, 1)),
            TextKind::Line => self.buffer.delete(Movement::Backward(TextKind::Line, 1)),
            TextKind::Block => self.buffer.delete(Movement::Backward(TextKind::Block, 1)),
        }
        self.view_changed = true;
        self.validate_range();
        self.adjust_view_range();
    }

    pub fn validate_range(&mut self) {
        if self.buffer_in_view.end > self.buffer.len() {
            self.buffer_in_view.end = self.buffer.len();
        }
    }

    pub fn id(&self) -> ViewId {
        self.id
    }

    pub fn debug_viewcursor(&self) {
        use crate::datastructure::generic::Vec2i;
        let Anchor(top_x, top_y) = self.anchor;
        let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
        let cols_in = *self.buffer.cursor_col() as i32;

        let g = self.text_renderer.get_glyph(*self.buffer.get(self.buffer.cursor_abs()).unwrap_or(&'\0'));
        let cursor_width = g.map(|glyph| {
            if glyph.width() == 0 as _ {
                glyph.advance
            } else {
                glyph.width() as _
            }
        }).unwrap_or(self.cursor_width);
        
        let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
        let line_contents = self.buffer.get_slice(nl_buf_idx .. (nl_buf_idx + cols_in as usize));

        let min_x = top_x + line_contents.iter().map(|& c| self.text_renderer.get_glyph(c).map(|g| g.advance as _).unwrap_or(cursor_width)).sum::<i32>();


        let min = Vec2i::new(min_x, top_y - (rows_down * self.row_height) - self.row_height);
        let max = Vec2i::new(min_x + cursor_width, top_y - (rows_down * self.row_height));

        let bb = BoundingBox::new(min, max);

        println!("View cursor: {:?}", bb);
    }

    pub fn debug_viewed_range(&self) {
        println!("Viewed data in buffer range {:?}: \n'{}'", self.buffer_in_view, &self.buffer.data[self.buffer_in_view.clone()].iter().map(|c| c).collect::<String>());
    }
} 