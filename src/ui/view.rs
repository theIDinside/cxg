use glfw::{Action, Key, Modifiers};

use super::boundingbox::BoundingBox;
use super::eventhandling::event::{InputBehavior, InputResponse};
use super::panel::PanelId;
use super::Viewable;
use super::{
    basic::{coordinate::Size, frame::Frame},
    font::Font,
};
use crate::datastructure::generic::Vec2i;
use crate::debugger_catch;
use crate::opengl::polygon_renderer::{PolygonRenderer, PolygonType, Texture};
use crate::opengl::{rectangle_renderer::RectRenderer, text_renderer::TextRenderer, types::RGBAColor};
use crate::ui::basic::coordinate::Margin;
use crate::{app::TEST_DATA, opengl::types::RGBColor};

use crate::textbuffer::{
    cursor::BufferCursor,
    metadata::{Index, Line},
    simple::simplebuffer::SimpleBuffer,
    CharBuffer, Movement, TextKind,
};

use crate::ui::coordinate::Coordinate;
use std::fmt::Formatter;
use std::path::Path;
use std::rc::Rc;

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

pub struct View {
    pub name: String,
    pub id: ViewId,
    pub title_font: Rc<Font>,
    pub edit_font: Rc<Font>,
    pub text_renderer: TextRenderer,
    pub window_renderer: PolygonRenderer,
    pub cursor_renderer: RectRenderer,
    pub title_frame: Frame,
    pub view_frame: Frame,
    pub topmost_line_in_buffer: i32,
    pub panel_id: Option<PanelId>,
    /// The currently edited buffer. We have sole ownership over it. If we want to edit another buffer in this view, (and thus hide the contents of this buffer)
    /// we return it back to the Buffers type, which manages live buffers and we replace this one with another Box<SimpleBuffer>, taking ownership of that
    pub buffer: Box<SimpleBuffer>,
    buffer_in_view: std::ops::Range<usize>,
    _buffer_selection: Option<std::ops::Range<Index>>,
    pub view_changed: bool,
    pub bg_color: RGBAColor,
    pub visible: bool,
    background_image: Texture,
}

pub struct Popup {
    pub visible: bool,
    pub view: View,
}

impl Popup {
    pub fn reset(&mut self) {
        self.view.buffer.clear();
        self.view.set_need_redraw();
    }
}

impl std::ops::Deref for Popup {
    type Target = View;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("title_frame", &self.title_frame)
            .field("view_frame", &self.view_frame)
            .field("size", &self.total_size())
            .field("top buffer line", &self.topmost_line_in_buffer)
            .field("displayable lines", &self.rows_displayable())
            .field("layout by", &self.panel_id)
            .finish()
    }
}

impl InputBehavior for View {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) -> InputResponse {
        match key {
            Key::Home | Key::Kp7 => match modifier {
                Modifiers::Control => self.cursor_goto(crate::textbuffer::metadata::Index(0)),
                _ => self.move_cursor(Movement::Begin(TextKind::Line)),
            },
            Key::End | Key::Kp1 => match modifier {
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
                    if modifier == Modifiers::Shift {
                        self.insert_str(TEST_DATA);
                    }
                }
            }
            Key::S if action == Action::Press && modifier == Modifiers::Control => {
                return InputResponse::SaveFile(self.buffer.file_name().map(Path::to_path_buf))
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

impl View {
    pub fn new(
        name: &str, view_id: ViewId, text_renderer: TextRenderer, mut cursor_renderer: RectRenderer, window_renderer: PolygonRenderer, width: i32, height: i32,
        bg_color: RGBAColor, mut buffer: Box<SimpleBuffer>, edit_font: Rc<Font>, title_font: Rc<Font>, background_image: Texture,
    ) -> View {
        let title_height = title_font.row_height() + 5;

        let tmp_anchor = Vec2i::new(0, height);
        let title_size = Size::new(width, title_height);
        let title_frame = Frame::new(tmp_anchor, title_size);
        let view_anchor = Vec2i::new(0, height - title_height);
        let view_size = Size::new(width, height - title_height);
        let view_frame = Frame::new(view_anchor, view_size);
        buffer.rebuild_metadata();

        cursor_renderer.set_color(RGBAColor { r: 0.5, g: 0.5, b: 0.5, a: 0.5 });
        let mut v = View {
            title_font,
            edit_font,
            name: name.to_string(),
            id: view_id,
            text_renderer,
            window_renderer,
            cursor_renderer,
            title_frame,
            view_frame,
            topmost_line_in_buffer: 0,
            panel_id: None,
            buffer,
            buffer_in_view: 0..0,
            _buffer_selection: None,
            view_changed: true,
            bg_color,
            visible: true,
            background_image,
        };

        v.update(None);
        v
    }

    pub fn set_manager_panel(&mut self, panel_id: PanelId) {
        self.panel_id = Some(panel_id);
    }

    pub fn mouse_to_buffer_position(&self, mouse_pos: Vec2i) -> Option<Index> {
        if BoundingBox::from_frame(&self.title_frame).box_hit_check(mouse_pos) {
            None
        } else {
            let Vec2i { x: ax, y: ay } = self.view_frame.anchor;
            let Vec2i { x: mx, y: my } = mouse_pos;

            let md = self.buffer.meta_data();
            let view_line = ((ay - my) as f64 / self.get_text_font().row_height() as f64).floor() as isize;
            let line_clicked = Line(self.topmost_line_in_buffer as usize).offset(view_line);

            let start_index = md
                .get_line_start_index(line_clicked)
                .unwrap_or(md.get_line_start_index(Line(md.line_count() - 1)).unwrap());

            let end_index = md.get_line_start_index(line_clicked.offset(1)).unwrap_or(Index(self.buffer.len()));

            let line_contents = self.buffer.get_slice(*start_index..*end_index);
            let mut rel_x = mx - ax;
            let text_font = self.get_text_font();
            let final_index_pos = line_contents
                .iter()
                .enumerate()
                .find(|(_, ch)| {
                    rel_x -= text_font.get_glyph(**ch).unwrap().advance;
                    rel_x <= 0
                })
                .map(|(i, _)| start_index.offset(i as isize))
                .unwrap_or(end_index.offset(-1));
            Some(final_index_pos)
        }
    }

    pub fn set_need_redraw(&mut self) {
        self.adjust_view_range();
        self.view_changed = true;
    }

    #[inline(always)]
    pub fn get_title_font(&self) -> Rc<Font> {
        self.title_font.clone()
    }

    #[inline(always)]
    pub fn get_text_font(&self) -> Rc<Font> {
        self.edit_font.clone()
    }

    /// Prepares the renderable data, so that upon next draw() call, it renders the new content
    pub fn update(&mut self, bg_texture: Option<Texture>) {
        self.window_renderer.clear_data();

        /* Make the title bar */
        self.window_renderer.make_bordered_rect(
            BoundingBox::expand(&self.title_frame.to_bb(), Margin::Vertical(10)).translate_mut(Vec2i::new(0, -4)),
            RGBColor::new(0.5, 0.5, 0.5),
            (1, RGBColor::black()),
            PolygonType::RoundedUndecorated { corner_radius: 10.0 },
        );

        let RGBAColor { r, g, b, .. } = self.bg_color;
        let bg_color = RGBColor::new(r, g, b);
        if let Some(texture) = bg_texture {
            self.window_renderer.make_bordered_rect(
                self.view_frame.to_bb(),
                bg_color,
                (2, RGBColor::black()),
                PolygonType::RoundedDecorated { corner_radius: 10.0, texture },
            );
        } else {
            self.window_renderer.make_bordered_rect(
                self.view_frame.to_bb(),
                bg_color,
                (2, RGBColor::black()),
                PolygonType::RoundedUndecorated { corner_radius: 10.0 },
            );
        }

        if self.buffer.empty() {
            let Size { width, height } = self.view_frame.size;
            let image_bb = BoundingBox::shrink(&self.view_frame.to_bb(), Margin::Perpendicular { h: width / 4, v: height / 4 });
            self.window_renderer
                .push_draw_command(image_bb, bg_color, PolygonType::Decorated { texture: self.background_image });
        }

        self.set_need_redraw();
    }

    pub fn draw_title(&mut self, title: &str) {
        let Vec2i { x: tx, y: ty } = self.title_frame.anchor;
        self.text_renderer
            .push_draw_command(title.chars().map(|c| c), RGBColor::white(), tx + 3, ty, self.get_title_font());
    }

    pub fn draw(&mut self) {
        if !self.visible {
            return;
        }
        let total_size = self.total_size();
        if self.view_changed {
            self.text_renderer.clear_data();
            // self.menu_text_renderer.clear_data();
            let BufferCursor { row, col, .. } = self.buffer.cursor();
            let title = format!(
                "{}:{}:{}",
                self.buffer
                    .file_name()
                    .map(|p| p.display().to_string())
                    .unwrap_or("unnamed_file".into()),
                *row,
                *col
            );

            self.draw_title(&title);

            unsafe {
                let Vec2i { x: top_x, y: top_y } = self.title_frame.anchor;
                gl::Enable(gl::SCISSOR_TEST);
                gl::Scissor(top_x, top_y - total_size.height, total_size.width, total_size.height);
            }

            // draw text view
            let Vec2i { x: top_x, y: top_y } = self.view_frame.anchor;
            let top_x = top_x + 2;
            unsafe {
                gl::ClearColor(0.8, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            // either way of these two works
            self.text_renderer.push_draw_command(
                self.buffer
                    .iter()
                    .skip(self.buffer_in_view.start)
                    .take(self.buffer_in_view.len() + 100)
                    .map(|c| *c),
                RGBColor::white(),
                top_x,
                top_y,
                self.get_text_font(),
            );

            if let Some(selection) = self._buffer_selection {}

            let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
            let cols_in = *self.buffer.cursor_col() as i32;

            let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
            let line_contents = self.buffer.get_slice(nl_buf_idx..(nl_buf_idx + cols_in as usize));
            use crate::opengl::text_renderer as gltxt;
            let min_x = top_x + gltxt::calculate_text_dimensions(line_contents, self.edit_font.as_ref()).x();
            let min = Vec2i::new(min_x, top_y - (rows_down * self.get_text_font().row_height()) - self.get_text_font().row_height() - 6);
            let max = Vec2i::new(min_x + self.get_text_font().get_max_glyph_width() - 2, top_y - (rows_down * self.get_text_font().row_height()));

            let mut cursor_bound_box = BoundingBox::new(min, max);
            let mut line_bounding_box = cursor_bound_box.clone();
            line_bounding_box.min.x = top_x;
            line_bounding_box.max.x = top_x + total_size.width;

            line_bounding_box.min.y += 2;
            line_bounding_box.max.y -= 2;

            cursor_bound_box.min.y += 3;
            cursor_bound_box.max.y -= 3;

            self.cursor_renderer.clear_data();

            self.cursor_renderer
                .add_rect(line_bounding_box, RGBAColor { r: 0.75, g: 0.75, b: 0.75, a: 0.2 });
            self.cursor_renderer
                .add_rect(cursor_bound_box, RGBAColor { r: 0.95, g: 0.75, b: 0.75, a: 0.5 });
            self.view_changed = false;
        }

        // Remember to draw in correct Z-order! We manage our own "layers". Therefore, draw cursor last
        self.window_renderer.draw_list();
        unsafe {
            let Vec2i { x: top_x, y: top_y } = self.title_frame.anchor;
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(top_x + 2, top_y - total_size.height, total_size.width - 4, total_size.height);
        }
        self.text_renderer.draw_list();
        // self.text_renderer.draw();
        self.cursor_renderer.draw();
        //self.menu_text_renderer.draw();

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
        let d: Vec<_> = s.chars().collect();
        self.buffer_in_view = 0..s.len();
        self.buffer.insert_slice(&d[..]);
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
        let Vec2i { x: top_x, y: top_y } = self.view_frame.anchor;
        let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
        let cols_in = *self.buffer.cursor_col() as i32;

        let text_font = self.get_text_font();

        let g = text_font.get_glyph(*self.buffer.get(self.buffer.cursor_abs()).unwrap_or(&'\0'));
        let cursor_width = g
            .map(|glyph| if glyph.width() == 0 as _ { glyph.advance } else { glyph.width() as _ })
            .unwrap_or(text_font.get_max_glyph_width());

        let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
        let line_contents = self.buffer.get_slice(nl_buf_idx..(nl_buf_idx + cols_in as usize));

        let min_x = top_x
            + line_contents
                .iter()
                .map(|&c| text_font.get_glyph(c).map(|g| g.advance as _).unwrap_or(cursor_width))
                .sum::<i32>();

        let min = Vec2i::new(min_x, top_y - (rows_down * text_font.row_height()) - text_font.row_height());
        let max = Vec2i::new(min_x + cursor_width, top_y - (rows_down * text_font.row_height()));

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
        self.view_frame.size.height / self.get_text_font().row_height()
    }

    pub fn total_boundingbox(&self) -> BoundingBox {
        let title_bb = BoundingBox::from_frame(&self.title_frame);
        let view_bb = BoundingBox::from_frame(&self.view_frame);
        BoundingBox::new(Vec2i::new(view_bb.min.x, view_bb.min.y), Vec2i::new(title_bb.max.x, title_bb.max.y))
    }

    pub fn total_size(&self) -> Size {
        Size {
            width: self.view_frame.size.width,
            height: self.view_frame.size.height + self.title_frame.size.height,
        }
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

impl Viewable for View {
    fn resize(&mut self, mut size: Size) {
        debug_assert!(size.height > 20, "resize size invalid. Must be larger than 20");
        size.height -= self.get_title_font().row_height() + 5;
        self.title_frame.size.width = size.width;
        self.view_frame.anchor.y = self.title_frame.anchor.y - self.title_frame.size.height;
        // self.view_frame.anchor = self.title_frame.anchor + Vec2i::new(0, -self.row_height - 5);
        self.view_frame.size = size;
        assert_eq!(self.view_frame.anchor, self.title_frame.anchor + Vec2i::new(0, -self.get_title_font().row_height() - 5));
        assert_eq!(self.view_frame.size.width, self.title_frame.size.width);
    }

    fn set_anchor(&mut self, anchor: Vec2i) {
        self.title_frame.anchor = anchor;
        self.view_frame.anchor = self.title_frame.anchor + Vec2i::new(0, -self.title_frame.size.height);
    }

    fn bounding_box(&self) -> BoundingBox {
        self.total_boundingbox()
    }

    fn mouse_clicked(&mut self, validated_inside_pos: Vec2i) {
        debugger_catch!(
            self.bounding_box().box_hit_check(validated_inside_pos),
            crate::DebuggerCatch::Handle(format!("This coordinate is not enclosed by this view"))
        );
        // means we clicked the title frame, we do not need to scan where the buffer cursor should land, we only need to activate the view
        if BoundingBox::from_frame(&self.title_frame).box_hit_check(validated_inside_pos) {
        } else {
            if let Some(final_index_pos) = self.mouse_to_buffer_position(validated_inside_pos) {
                self.cursor_goto(final_index_pos);
            }
        }
    }

    fn mouse_dragged(&mut self, begin_coordinate: Vec2i, current_coordinate: Vec2i) {
        // description
        // check if begin_coordinate lands on cursor
        // if not, set cursor to begin_coordinate, selecting that position in the text buffer, set at begin_range
        // check where in the text buffer current_coordinate lands, set this as current_selection, thus we have selected begin_range ..= current_selection
        // draw selection cursor accordingly
        use std::cmp::{max, min};

        self._buffer_selection = self
            .mouse_to_buffer_position(begin_coordinate)
            .zip(self.mouse_to_buffer_position(current_coordinate))
            .and_then(|(b, e)| Some(min(b, e)..max(b, e).offset(1)));
    }
}
