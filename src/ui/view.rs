use glfw::{Action, Key, Modifiers};

use super::boundingbox::BoundingBox;
use super::coordinate::{Anchor, Size};
use super::eventhandling::event::{InputBehavior, InputResponse};
use super::panel::PanelId;
use super::Viewable;
use crate::app::TEST_DATA;
use crate::datastructure::generic::Vec2i;
use crate::debugger_catch;
use crate::opengl::rect::RectRenderer;
use crate::opengl::text::TextRenderer;
use crate::opengl::types::RGBAColor;
use crate::textbuffer::cursor::BufferCursor;
use crate::textbuffer::metadata::{Index, Line};
use crate::textbuffer::simple::simplebuffer::SimpleBuffer;
use crate::textbuffer::{CharBuffer, Movement, TextKind};

use crate::ui::coordinate::Coordinate;
use std::fmt::Formatter;
use std::path::Path;

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
    pub size: Size,
    pub anchor: Anchor,
    pub topmost_line_in_buffer: i32,
    row_height: i32,
    pub panel_id: Option<PanelId>,
    /// The currently edited buffer. We have sole ownership over it. If we want to edit another buffer in this view, (and thus hide the contents of this buffer)
    /// we return it back to the Buffers type, which manages live buffers and we replace this one with another Box<SimpleBuffer>, taking ownership of that
    pub buffer: Box<SimpleBuffer>,
    buffer_in_view: std::ops::Range<usize>,
    pub view_changed: bool,
    pub bg_color: RGBAColor,
    pub visible: bool,
}

pub struct Popup<'app> {
    pub visible: bool,
    pub view: View<'app>,
}

impl<'app> Popup<'app> {
    pub fn reset(&mut self) {
        self.view.buffer.clear();
        self.view.set_need_redraw();
    }
}

impl<'a> std::ops::Deref for Popup<'a> {
    type Target = View<'a>;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<'a> std::fmt::Debug for View<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("size", &self.size)
            .field("anchor", &self.anchor)
            .field("top buffer line", &self.topmost_line_in_buffer)
            .field("displayable lines", &self.rows_displayable())
            .field("layout by", &self.panel_id)
            .finish()
    }
}

impl<'app> InputBehavior for View<'app> {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) -> InputResponse {
        match key {
            Key::Home => match modifier {
                Modifiers::Control => self.cursor_goto(crate::textbuffer::metadata::Index(0)),
                _ => self.move_cursor(Movement::Begin(TextKind::Line)),
            },
            Key::End => match modifier {
                Modifiers::Control => self.cursor_goto(crate::textbuffer::metadata::Index(self.buffer.len())),
                _ => self.move_cursor(Movement::End(TextKind::Line)),
            },
            Key::Right if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    self.move_cursor(Movement::End(TextKind::Word));
                } else if modifier == (Modifiers::Shift | Modifiers::Alt) {
                    self.move_cursor(Movement::End(TextKind::Block));
                } else {
                    self.move_cursor(Movement::Forward(TextKind::Char, 1));
                }
            }
            Key::Left if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    self.move_cursor(Movement::Begin(TextKind::Word));
                } else if modifier == (Modifiers::Shift | Modifiers::Alt) {
                    self.move_cursor(Movement::Begin(TextKind::Block));
                } else {
                    self.move_cursor(Movement::Backward(TextKind::Char, 1));
                }
            }
            Key::Up if action == Action::Repeat || action == Action::Press => {
                self.move_cursor(Movement::Backward(TextKind::Line, 1));
            }
            Key::Down if action == Action::Repeat || action == Action::Press => {
                self.move_cursor(Movement::Forward(TextKind::Line, 1));
            }
            Key::Backspace if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    self.delete(Movement::Backward(TextKind::Word, 1));
                } else {
                    self.delete(Movement::Backward(TextKind::Char, 1));
                }
            }
            Key::Delete if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    self.delete(Movement::Forward(TextKind::Word, 1));
                } else if modifier.is_empty() {
                    self.delete(Movement::Forward(TextKind::Char, 1));
                }
            }
            Key::F1 => {
                if action == Action::Press {
                    if modifier == Modifiers::Control {
                        self.insert_str(TEST_DATA);
                    }
                }
            }
            Key::Enter if action == Action::Press || action == Action::Repeat => {
                self.insert_ch('\n');
            }
            _ => {}
        }
        InputResponse::None
    }

    fn handle_char(&mut self, ch: char) {
        self.insert_ch(ch);
    }

    fn get_uid(&self) -> Option<super::UID> {
        Some(super::UID::View(*self.id))
    }
}

impl<'a> View<'a> {
    pub fn new(
        name: &str, view_id: ViewId, text_renderer: TextRenderer<'a>, window_renderer: RectRenderer, width: i32, height: i32, bg_color: RGBAColor,
        buffer: Box<SimpleBuffer>,
    ) -> View<'a> {
        let row_height = text_renderer.font.row_height();
        let cursor_shader = window_renderer.shader.clone();
        let mut cursor_renderer = RectRenderer::create(cursor_shader, 100);
        cursor_renderer.set_color(RGBAColor { r: 0.5, g: 0.5, b: 0.5, a: 0.5 });
        let mut v = View {
            name: name.to_string(),
            id: view_id,
            text_renderer,
            window_renderer,
            cursor_renderer,
            size: Size::new(width, height),
            anchor: Anchor(0, 0),
            topmost_line_in_buffer: 0,
            row_height,
            panel_id: None,
            buffer,
            buffer_in_view: 0..0,
            view_changed: true,
            bg_color,
            visible: true,
        };
        v.window_renderer.add_rect(BoundingBox::from_info(v.anchor, v.size), bg_color);
        v
    }

    pub fn set_manager_panel(&mut self, panel_id: PanelId) {
        self.panel_id = Some(panel_id);
    }

    pub fn set_need_redraw(&mut self) {
        self.adjust_view_range();
        self.view_changed = true;
    }

    /// Prepares the renderable data, so that upon next draw() call, it renders the new content
    pub fn update(&mut self) {
        self.window_renderer
            .set_rect(BoundingBox::from_info(self.anchor, self.size), self.bg_color);
        self.set_need_redraw();
    }

    pub fn draw(&mut self) {
        if !self.visible {
            return;
        }
        let Anchor(top_x, top_y) = self.anchor;
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(top_x, top_y - self.size.height, self.size.width, self.size.height);
        }
        if self.view_changed {
            unsafe {
                gl::ClearColor(0.8, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            // either way of these two works
            self.text_renderer
                .prepare_data_from_iter(self.buffer.str_view(self.buffer_in_view.clone()), top_x, top_y);
            // self.text_renderer.prepare_data_iter(self.buffer.iter().skip(self.buffer_in_view.start).take(self.buffer_in_view.len()), top_x, top_y);

            let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
            let cols_in = *self.buffer.cursor_col() as i32;

            let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
            crate::only_in_debug!(crate::debugger_catch!(nl_buf_idx + (cols_in as usize) <= self.buffer.len(), "range is outside of buffer"));
            let line_contents = self.buffer.get_slice(nl_buf_idx..(nl_buf_idx + cols_in as usize));

            let min_x = top_x + self.text_renderer.calculate_text_line_dimensions(line_contents).x();
            let min = Vec2i::new(min_x, top_y - (rows_down * self.row_height) - self.row_height - 6);
            let max = Vec2i::new(min_x + self.text_renderer.get_cursor_width_size(), top_y - (rows_down * self.row_height));

            let mut cursor_bound_box = BoundingBox::new(min, max);
            let mut line_bounding_box = cursor_bound_box.clone();
            line_bounding_box.min.x = top_x;
            line_bounding_box.max.x = top_x + self.size.width;

            cursor_bound_box.min.y += 2;
            cursor_bound_box.max.y -= 2;

            self.cursor_renderer.clear_data();

            self.cursor_renderer
                .add_rect(line_bounding_box, RGBAColor { r: 0.75, g: 0.75, b: 0.75, a: 0.2 });
            self.cursor_renderer
                .add_rect(cursor_bound_box, RGBAColor { r: 0.95, g: 0.75, b: 0.75, a: 0.5 });
            self.view_changed = false;
        }

        // Remember to draw in correct Z-order! We manage our own "layers". Therefore, draw cursor last
        self.window_renderer.draw();
        self.text_renderer.draw();
        self.cursor_renderer.draw();

        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        debugger_catch!(self.buffer.empty(), crate::DebuggerCatch::Handle(format!("View must be empty in order to load data from file")));
        if self.buffer.empty() {
            self.buffer.load_file(path);
            self.adjust_view_range();
        }
    }

    pub fn insert_ch(&mut self, ch: char) {
        if input_not_valid(ch) {
            return;
        }

        self.buffer.insert(ch);
        if self.buffer.cursor_row() >= Line((self.topmost_line_in_buffer + self.rows_displayable()) as _) {
            self.adjust_view_range();
        } else {
            self.buffer_in_view.end += 1;
            self.view_changed = true;
        }
    }

    pub fn adjust_view_range(&mut self) {
        let md = self.buffer.meta_data();
        if self.buffer.cursor_row() >= Line((self.topmost_line_in_buffer + self.rows_displayable()) as _) {
            let diff = std::cmp::max((*self.buffer.cursor_row() as i32) - (self.topmost_line_in_buffer + self.rows_displayable()) as i32, 1);
            self.topmost_line_in_buffer += diff;
            if let (Some(a), end) =
                md.get_byte_indices_of_lines(Line(self.topmost_line_in_buffer as _), Line((self.topmost_line_in_buffer + self.rows_displayable()) as _))
            {
                self.buffer_in_view = *a..*end.unwrap_or(Index(self.buffer.len()));
            }

            self.view_changed = true;
        } else if self.buffer.cursor_row() < Line(self.topmost_line_in_buffer as _) {
            self.topmost_line_in_buffer = *self.buffer.cursor_row() as _;
            if let (Some(a), end) =
                md.get_byte_indices_of_lines(Line(self.topmost_line_in_buffer as _), Line((self.topmost_line_in_buffer + self.rows_displayable()) as _))
            {
                self.buffer_in_view = *a..*end.unwrap_or(Index(self.buffer.len()));
            }
        } else {
            if let (Some(a), end) =
                md.get_byte_indices_of_lines(Line(self.topmost_line_in_buffer as _), Line((self.topmost_line_in_buffer + self.rows_displayable()) as _))
            {
                self.buffer_in_view = *a..*end.unwrap_or(Index(self.buffer.len()));
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
        self.buffer_in_view = 0..s.len();
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
        self.buffer.move_cursor(dir);
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
        let Anchor(top_x, top_y) = self.anchor;
        let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
        let cols_in = *self.buffer.cursor_col() as i32;

        let g = self
            .text_renderer
            .get_glyph(*self.buffer.get(self.buffer.cursor_abs()).unwrap_or(&'\0'));
        let cursor_width = g
            .map(|glyph| if glyph.width() == 0 as _ { glyph.advance } else { glyph.width() as _ })
            .unwrap_or(self.text_renderer.get_cursor_width_size());

        let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
        let line_contents = self.buffer.get_slice(nl_buf_idx..(nl_buf_idx + cols_in as usize));

        let min_x = top_x
            + line_contents
                .iter()
                .map(|&c| self.text_renderer.get_glyph(c).map(|g| g.advance as _).unwrap_or(cursor_width))
                .sum::<i32>();

        let min = Vec2i::new(min_x, top_y - (rows_down * self.row_height) - self.row_height);
        let max = Vec2i::new(min_x + cursor_width, top_y - (rows_down * self.row_height));

        let bb = BoundingBox::new(min, max);

        println!("View cursor: {:?}", bb);
    }

    pub fn debug_viewed_range(&self) {
        println!(
            "Viewed data in buffer range {:?}: \n'{}'",
            self.buffer_in_view,
            &self.buffer.data[self.buffer_in_view.clone()].iter().map(|c| c).collect::<String>()
        );
    }

    pub fn get_file_info(&self) -> (Option<&Path>, BufferCursor) {
        self.buffer.buffer_info()
    }

    pub fn rows_displayable(&self) -> i32 {
        self.size.height / self.row_height
    }
}

fn input_not_valid(ch: char) -> bool {
    let mut buf = [0; 4];
    ch.encode_utf16(&mut buf);
    for cp in buf {
        if cp > 1000 {
            return true;
        }
    }
    false
}

impl<'app> Viewable for View<'app> {
    fn resize(&mut self, size: Size) {
        self.size = size;
    }

    fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_info(self.anchor, self.size)
    }

    fn mouse_clicked(&mut self, pos: Vec2i) {
        debugger_catch!(self.bounding_box().box_hit_check(pos), crate::DebuggerCatch::Handle(format!("This coordinate is not enclosed by this view")));

        let Anchor(ax, ay) = self.anchor;
        let Vec2i { x: mx, y: my } = pos;

        let md = self.buffer.meta_data();
        let view_line = ((ay - my) as f64 / self.row_height as f64).floor() as isize;
        let line_clicked = Line(self.topmost_line_in_buffer as usize).offset(view_line);

        let start_index = md
            .get_line_start_index(line_clicked)
            .unwrap_or(md.get_line_start_index(Line(md.line_count() - 1)).unwrap());

        let end_index = md.get_line_start_index(line_clicked.offset(1)).unwrap_or(Index(self.buffer.len()));

        let line_contents = self.buffer.get_slice(*start_index..*end_index);
        let mut rel_x = mx - ax;

        let final_index_pos = line_contents
            .iter()
            .enumerate()
            .find(|(_, ch)| {
                rel_x -= self.text_renderer.get_glyph(**ch).unwrap().advance;
                rel_x <= 0
            })
            .map(|(i, _)| start_index.offset(i as isize))
            .unwrap_or(end_index.offset(-1));

        self.cursor_goto(final_index_pos);
    }
}
