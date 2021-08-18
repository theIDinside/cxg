use glfw::{Action, Key, Modifiers};

use super::boundingbox::BoundingBox;
use super::eventhandling::event::{key_press, key_press_repeat, CommandOutput, InputBehavior};
use super::eventhandling::input::KeyboardInputContext;
use super::panel::PanelId;
use super::scrollbar::{ScrollBar, ScrollBarLayout};
use super::Viewable;
use super::{
    basic::{coordinate::Size, frame::Frame},
    font::Font,
};
use crate::datastructure::generic::Vec2i;
use crate::opengl::polygon_renderer::{PolygonRenderer, PolygonType, Texture};
use crate::opengl::{rectangle_renderer::RectRenderer, text_renderer::TextRenderer, types::RGBAColor};
use crate::textbuffer::cursor::MetaCursor;
use crate::textbuffer::operations::LineOperation;
use crate::ui::basic::coordinate::Margin;
use crate::Assert;
use crate::{app::TEST_DATA, opengl::types::RGBColor};

use crate::textbuffer::{
    contiguous::contiguous::ContiguousBuffer,
    cursor::BufferCursor,
    metadata::{Index, Line},
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

use crate::opengl::text_renderer as gltxt;
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
    pub buffer: Box<ContiguousBuffer>,
    buffer_in_view: std::ops::Range<usize>,
    pub view_changed: bool,
    pub bg_color: RGBAColor,
    pub visible: bool,
    background_image: Texture,
    text_margin_left: i32,
    pub scroll_bar: ScrollBar,
    pub scroll_bar_interacting: bool,
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
            .field("size", &self.total_size())
            .field("top buffer line", &self.topmost_line_in_buffer)
            .field("displayable lines", &self.rows_displayable())
            .field("layout by", &self.panel_id)
            .finish()
    }
}

impl InputBehavior for View {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) -> CommandOutput {
        match key {
            Key::Tab if key_press(action) => {
                if let Some((begin, end)) = self.buffer.get_selection() {
                    let md = self.buffer.meta_data();
                    let a = unsafe { md.get_line_number_of_buffer_index(begin).unwrap_unchecked() };
                    let b_inclusive = unsafe { md.get_line_number_of_buffer_index(end).unwrap_unchecked() };
                    if modifier == Modifiers::Shift {
                        self.buffer
                            .line_operation(a..b_inclusive + 1, &LineOperation::ShiftLeft { shift_by: 4 });
                    } else {
                        self.buffer
                            .line_operation(a..b_inclusive + 1, &LineOperation::ShiftRight { shift_by: 4 });
                    }
                } else {
                    self.insert_slice(&[' ', ' ', ' ', ' ']);
                }
            }
            Key::Home | Key::Kp7 if key_press(action) => match modifier {
                Modifiers::Control => self.cursor_goto(crate::textbuffer::metadata::Index(0)),
                _ => self.move_cursor(Movement::Begin(TextKind::Line)),
            },
            Key::End | Key::Kp1 if key_press(action) => match modifier {
                Modifiers::Control => self.cursor_goto(crate::textbuffer::metadata::Index(self.buffer.len())),
                Modifiers::Shift => {
                    self.buffer.select_move_cursor_absolute(Movement::End(TextKind::Line));
                }
                _ => self.move_cursor(Movement::End(TextKind::Line)),
            },
            Key::Right if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control | Modifiers::Shift {
                    self.buffer.select_move_cursor_absolute(Movement::End(TextKind::Word));
                } else if modifier == (Modifiers::Shift | Modifiers::Alt) {
                    self.move_cursor(Movement::End(TextKind::Block));
                } else if modifier == Modifiers::Control {
                    self.move_cursor(Movement::End(TextKind::Word));
                } else if modifier == Modifiers::Shift {
                    self.buffer.select_move_cursor_absolute(Movement::Forward(TextKind::Char, 1));
                } else {
                    self.move_cursor(Movement::Forward(TextKind::Char, 1));
                }
            }
            Key::Left if key_press_repeat(action) => {
                if modifier == Modifiers::Control {
                    self.move_cursor(Movement::Begin(TextKind::Word));
                } else if modifier == Modifiers::Shift | Modifiers::Alt {
                    self.move_cursor(Movement::Begin(TextKind::Block));
                } else if modifier == Modifiers::Shift {
                    self.buffer.select_move_cursor_absolute(Movement::Backward(TextKind::Char, 1));
                } else if modifier == Modifiers::Shift | Modifiers::Control {
                    self.buffer.select_move_cursor_absolute(Movement::Begin(TextKind::Word));
                } else {
                    self.move_cursor(Movement::Backward(TextKind::Char, 1));
                }
            }
            Key::Up if key_press_repeat(action) => {
                if modifier == Modifiers::Shift {
                    self.buffer.select_move_cursor_absolute(Movement::Backward(TextKind::Line, 1));
                } else {
                    self.move_cursor(Movement::Backward(TextKind::Line, 1));
                }
            }
            Key::Down if key_press_repeat(action) => {
                if modifier == Modifiers::Shift {
                    self.buffer.select_move_cursor_absolute(Movement::Forward(TextKind::Line, 1));
                } else {
                    self.move_cursor(Movement::Forward(TextKind::Line, 1));
                }
            }
            Key::PageDown | Key::Kp3 if key_press_repeat(action) => {
                if modifier == Modifiers::Shift {
                    self.buffer
                        .select_move_cursor_absolute(Movement::Forward(TextKind::Line, self.rows_displayable() as _));
                } else {
                    self.move_cursor(Movement::Forward(TextKind::Line, self.rows_displayable() as _));
                }
            }
            Key::PageUp | Key::Kp9 if key_press_repeat(action) => {
                if modifier == Modifiers::Shift {
                    self.buffer
                        .select_move_cursor_absolute(Movement::Backward(TextKind::Line, self.rows_displayable() as _));
                } else {
                    self.move_cursor(Movement::Backward(TextKind::Line, self.rows_displayable() as _));
                }
            }
            Key::Backspace if key_press_repeat(action) => {
                if modifier == Modifiers::Control {
                    self.delete(Movement::Backward(TextKind::Word, 1));
                } else {
                    self.delete(Movement::Backward(TextKind::Char, 1));
                }
            }
            Key::Delete if key_press_repeat(action) => {
                if modifier == Modifiers::Control {
                    self.delete(Movement::Forward(TextKind::Word, 1));
                } else if modifier.is_empty() {
                    self.delete(Movement::Forward(TextKind::Char, 1));
                }
            }
            Key::F1 if key_press(action) => {
                if modifier == Modifiers::Shift {
                    self.insert_str(TEST_DATA);
                }
            }
            Key::S if key_press(action) && modifier == Modifiers::Control => return CommandOutput::SaveFile(self.buffer.file_name().map(Path::to_path_buf)),
            Key::Enter if key_press_repeat(action) => {
                self.insert_ch('\n');
            }
            // Copy
            Key::C if key_press(action) && modifier == Modifiers::Control => return CommandOutput::ClipboardCopy(self.buffer.copy_range_or_line()),
            // Cut. todo: for now it just copies it. change it so it actually cuts
            Key::X if key_press(action) && modifier == Modifiers::Control => return CommandOutput::ClipboardCopy(self.buffer.copy_range_or_line()),
            Key::Escape if key_press(action) => {
                if self.buffer.meta_cursor.is_some() {
                    self.buffer.meta_cursor = None;
                    self.set_need_redraw();
                }
            }
            _ => {}
        }
        self.set_view_on_buffer_cursor();
        CommandOutput::None
    }

    fn handle_char(&mut self, ch: char) {
        self.insert_ch(ch);
    }

    fn get_uid(&self) -> Option<super::UID> {
        Some(super::UID::View(*self.id))
    }

    fn move_cursor(&mut self, movement: Movement) {
        self.move_cursor(movement);
    }

    fn context(&self) -> KeyboardInputContext {
        KeyboardInputContext::TextView
    }

    fn select_move_cursor(&mut self, movement: Movement) {
        self.buffer.select_move_cursor_absolute(movement);
        self.set_view_on_buffer_cursor();
    }

    fn delete(&mut self, movement: Movement) {
        self.delete(movement);
    }

    fn copy(&self) -> Option<String> {
        self.buffer.copy_range_or_line()
    }

    fn cut(&self) -> Option<String> {
        self.buffer.copy_range_or_line()
    }
}

impl View {
    const SCROLL_BAR_WIDTH: i32 = 15;

    pub fn set_font(&mut self, font: Rc<Font>) {
        self.edit_font = font;
    }

    pub fn new(
        name: &str, view_id: ViewId, text_renderer: TextRenderer, mut cursor_renderer: RectRenderer, window_renderer: PolygonRenderer, width: i32, height: i32,
        bg_color: RGBAColor, mut buffer: Box<ContiguousBuffer>, edit_font: Rc<Font>, title_font: Rc<Font>, background_image: Texture,
    ) -> View {
        let title_height = title_font.row_height() + 5;

        let tmp_anchor = Vec2i::new(0, height);
        let title_size = Size::new(width, title_height);
        let title_frame = Frame::new(tmp_anchor, title_size);
        let view_anchor = Vec2i::new(0, height - title_height);
        let view_size = Size::new(width - View::SCROLL_BAR_WIDTH, height - title_height);
        let view_frame = Frame::new(view_anchor, view_size);
        buffer.rebuild_metadata();

        let scroll_bar_frame =
            Frame::new(view_frame.anchor + Vec2i::new(width - View::SCROLL_BAR_WIDTH, 0), Size::new(View::SCROLL_BAR_WIDTH, height - title_height));

        let sb = ScrollBar::new(scroll_bar_frame, buffer.meta_data().line_count(), ScrollBarLayout::Vertical, 0);

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
            view_changed: true,
            bg_color,
            visible: true,
            background_image,
            text_margin_left: 4,
            scroll_bar: sb,
            scroll_bar_interacting: false,
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
        } else if self.scroll_bar.frame.to_bb().box_hit_check(mouse_pos) {
            None
        } else {
            let Vec2i { x: ax, y: ay } = self.view_frame.anchor;
            let Vec2i { x: mx, y: my } = mouse_pos;

            let md = self.buffer.meta_data();
            let view_line = ((ay - my) as f64 / self.get_text_font().row_height() as f64).floor() as isize;
            let line_clicked = Line(self.topmost_line_in_buffer as usize).offset(view_line);

            let start_index = md.get_line_start_index(line_clicked).unwrap_or(md.get_last_line());
            Assert!(
                *start_index <= self.buffer.len(),
                format!("Illegal access of buffer; getting start {} from buffer of only {} len", *start_index, self.buffer.len(),)
            );

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
        self.view_changed = true;
        self.scroll_bar.ui_update();
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
            RGBAColor::new(0.5, 0.5, 0.5, 1.0),
            (1, RGBAColor::black()),
            PolygonType::RoundedUndecorated { corner_radius: 3.5 },
        );

        let bg_color = self.bg_color;
        if let Some(texture) = bg_texture {
            self.window_renderer.make_bordered_rect(
                self.view_frame.to_bb(),
                bg_color,
                (2, RGBAColor::black()),
                PolygonType::RoundedDecorated { corner_radius: 3.5, texture },
            );
        } else {
            self.window_renderer.make_bordered_rect(
                self.view_frame.to_bb(),
                bg_color,
                (2, RGBAColor::black()),
                PolygonType::RoundedUndecorated { corner_radius: 3.5 },
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

    pub fn draw(&mut self) {
        if !self.visible {
            return;
        }
        let total_size = self.total_size();
        if self.view_changed {
            self.scroll_bar.set_max(self.buffer.meta_data().line_count());
            self.text_renderer.clear_data();
            self.cursor_renderer.clear_data();
            self.update(None);
            // create the scroll bar
            self.window_renderer
                .push_draw_command(self.scroll_bar.frame.to_bb(), self.bg_color.uniform_scale(-0.05), PolygonType::Undecorated);
            assert_eq!(self.scroll_bar.slider.width(), self.scroll_bar.frame.width());
            self.window_renderer.make_bordered_rect(
                self.scroll_bar.slider.to_bb(),
                self.bg_color.uniform_scale(0.2),
                (1, RGBAColor::white()),
                PolygonType::RoundedUndecorated { corner_radius: 7.5 },
            );

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
            let top_x = top_x + self.text_margin_left;

            // render text contents
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
            self.cursor_renderer.clear_data();
            if let Some(marker) = self.buffer.meta_cursor {
                match marker {
                    crate::textbuffer::cursor::MetaCursor::Absolute(ref abs_pos) => {
                        self.render_absolute_selection(*abs_pos);
                    }
                    #[allow(unused)]
                    crate::textbuffer::cursor::MetaCursor::LineRange { column, begin, end } => {
                        todo!();
                    }
                }
            } else {
                self.view_changed = false;
            }
            self.render_normal_cursor();
            self.view_changed = false;
        }

        // Remember to draw in correct Z-order! We manage our own "layers". Therefore, draw cursor last
        self.window_renderer.execute_draw_list();
        let Vec2i { x: top_x, y: top_y } = self.title_frame.anchor;
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(top_x + 2, top_y - total_size.height, self.view_frame.width() - self.text_margin_left, total_size.height);
        }
        self.text_renderer.execute_draw_list();

        // we clip here as well, because otherwise the cursor might show up "on top" of the title bar, which is undesirable
        unsafe {
            gl::Scissor(top_x + 2, top_y - total_size.height, self.view_frame.width() - self.text_margin_left, self.view_frame.height());
        }
        self.cursor_renderer.draw();
        //self.menu_text_renderer.draw();

        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }

    fn render_absolute_selection(&mut self, absolute_metacursor_position: Index) {
        let selection_color = RGBAColor { r: 0.75, g: 0.75, b: 0.95, a: 0.3 };
        // draw text view
        let Vec2i { x: top_x, y: top_y } = self.view_frame.anchor;
        let top_x = top_x + self.text_margin_left;
        if absolute_metacursor_position < self.buffer.cursor_abs() {
            // means we have drag-selected downwards/forwards
            let first_line = self
                .buffer
                .meta_data()
                .get_line_number_of_buffer_index(absolute_metacursor_position)
                .map_or(Line(0), |l| Line(l));
            let last_line = self.buffer.cursor_row();
            if first_line == last_line {
                let rows_down_in_view: i32 = *first_line as i32 - self.topmost_line_in_buffer;
                let line_begin = self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
                let begin_selection = absolute_metacursor_position - line_begin;
                let end_selection = self.buffer.cursor_col();
                let slice = self.buffer.get_slice(*line_begin..*self.buffer.cursor_abs());
                let begin_x = gltxt::calculate_text_dimensions(&slice[0..*begin_selection], self.edit_font.as_ref()).x();
                let end_x = gltxt::calculate_text_dimensions(&slice[0..*end_selection], self.edit_font.as_ref()).x();

                let min = Vec2i::new(top_x + begin_x, top_y - (rows_down_in_view + 1) * self.get_text_font().row_height());
                let max =
                    Vec2i::new(top_x + end_x + self.get_text_font().get_max_glyph_width() - 2, top_y - rows_down_in_view * self.get_text_font().row_height());
                let rect = BoundingBox::new(min, max).translate(Vec2i::new(self.text_margin_left / 2, -3));
                self.cursor_renderer.add_rect(rect, selection_color);
            } else {
                let rows_down_in_view: i32 = *first_line as i32 - self.topmost_line_in_buffer;
                let translate_vector = self.view_frame.anchor + Vec2i::new(self.text_margin_left, -(rows_down_in_view * self.edit_font.row_height()));
                let rendered = self.render_selection_requires_translation(absolute_metacursor_position, self.buffer.cursor_abs());
                for bb in rendered {
                    let translated = bb.translate(translate_vector);
                    self.cursor_renderer.add_rect(translated, selection_color);
                }
                self.view_changed = false;
            }
        } else {
            // means we drag-selected upwards/backwards
            let md = self.buffer.meta_data();
            let first_line_number = self.buffer.cursor_row();
            let last_line_number = md
                .get_line_number_of_buffer_index(absolute_metacursor_position)
                .map_or(Line(md.line_count()).offset(-1), |l| Line(l));

            if first_line_number == last_line_number {
                let rows_down_in_view: i32 = *first_line_number as i32 - self.topmost_line_in_buffer;
                let line_begin = self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
                // let begin_selection = marker - line_begin;
                let begin_selection = Index(*self.buffer.cursor_col());
                let end_selection = *absolute_metacursor_position - *line_begin;
                let slice = self.buffer.get_slice(*line_begin..*absolute_metacursor_position);
                let begin_x = gltxt::calculate_text_dimensions(&slice[0..*begin_selection], self.edit_font.as_ref()).x();
                let end_x = gltxt::calculate_text_dimensions(&slice[0..end_selection], self.edit_font.as_ref()).x();

                let min = Vec2i::new(top_x + begin_x, top_y - (rows_down_in_view + 1) * self.get_text_font().row_height());
                let max =
                    Vec2i::new(top_x + end_x + self.get_text_font().get_max_glyph_width() - 2, top_y - rows_down_in_view * self.get_text_font().row_height());
                let rect = BoundingBox::new(min, max).translate(Vec2i::new(0, -3));
                self.cursor_renderer.add_rect(rect, selection_color);
            } else {
                let rows_down_in_view: i32 = *first_line_number as i32 - self.topmost_line_in_buffer;
                // let rows_down_in_view: i32 = *first_line as i32 - self.topmost_line_in_buffer;
                let translate_vector = self.view_frame.anchor + Vec2i::new(self.text_margin_left, -(rows_down_in_view * self.edit_font.row_height()));
                let rendered = self.render_selection_requires_translation(self.buffer.cursor_abs(), absolute_metacursor_position);
                for bb in rendered {
                    let translated = bb.translate(translate_vector);
                    self.cursor_renderer.add_rect(translated, selection_color);
                }

                self.view_changed = false;
            }
        }
    }

    fn render_normal_cursor(&mut self) {
        // Rendering the "normal" cursor stuff, i.e. the block cursor, and the line highlighter
        let rows_down: i32 = *self.buffer.cursor_row() as i32 - self.topmost_line_in_buffer;
        let cols_in = *self.buffer.cursor_col() as i32;

        let nl_buf_idx = *self.buffer.meta_data().get_line_start_index(self.buffer.cursor_row()).unwrap();
        let line_contents = self.buffer.get_slice(nl_buf_idx..(nl_buf_idx + cols_in as usize));

        let min_x = gltxt::calculate_text_dimensions(line_contents, self.edit_font.as_ref()).x();
        let min = Vec2i::new(min_x, 0 - (rows_down + 1) * self.get_text_font().row_height());
        let max = Vec2i::new(min_x + self.get_text_font().get_max_glyph_width() - 2, 0 - (rows_down * self.get_text_font().row_height()));

        let cursor_bound_box = BoundingBox::new(min, max)
            .translate(Vec2i::new(self.text_margin_left, -3))
            .translate(self.view_frame.anchor);
        let mut line_bounding_box = cursor_bound_box.clone();
        line_bounding_box.min.x = self.view_frame.anchor.x + 2;
        line_bounding_box.max.x = self.view_frame.anchor.x + 2 + self.view_frame.width();

        self.cursor_renderer
            .add_rect(line_bounding_box, RGBAColor { r: 0.75, g: 0.75, b: 0.75, a: 0.2 });
        self.cursor_renderer
            .add_rect(cursor_bound_box, RGBAColor { r: 0.95, g: 0.75, b: 0.75, a: 0.5 });
    }

    // Renders bounding box(es) for the text range between begin and end. If this encompasses only one line, a vec![bb] will be returned, if more, then vec![bb_a, ..] and so on
    // The bounding boxes will be in it's own coordinate space, and thus has to be mapped onto whatever coordinate space that the caller requires, which isn't that hard
    // of a job. Therefore, the first bounding box, will have it's origin (the min member and its x,y values, that is): Vec2i(0, 0)
    // and if spanning multiple lines, each subsequent line will have Vec2i(0, (line * row_height) * -1). This should make remapping fairly easy
    fn render_selection_requires_translation(&self, begin: Index, end: Index) -> Vec<BoundingBox> {
        debug_assert!(begin < end);

        let md = self.buffer.meta_data();
        let first_line = md.get_line_number_of_buffer_index(begin).map_or(Line(0), |l| Line(l));
        let last_line = md
            .get_line_number_of_buffer_index(end)
            .map_or(Line(md.line_count()).offset(-1), |l| Line(l));
        let mut render_infos = Vec::with_capacity(*last_line - *first_line);
        let mut lines_contents = self.buffer.get_lines_as_slices(first_line, last_line);
        let mut rows_down_in_view: i32 = 0;
        let first_selected_col_position = *begin - *md.get_line_start_index(first_line).unwrap();
        let last_selected_col_position = *end - *md.get_line_start_index(last_line).unwrap();

        let cursor_start_x = gltxt::calculate_text_dimensions(&lines_contents[0][0..first_selected_col_position], self.edit_font.as_ref()).x();
        let remaining_line_width = gltxt::calculate_text_dimensions(&lines_contents[0][first_selected_col_position..], self.edit_font.as_ref()).x();
        let min = Vec2i::new(cursor_start_x, 0 - (rows_down_in_view + 1) * self.get_text_font().row_height());
        let max = Vec2i::new(cursor_start_x + remaining_line_width, 0 - rows_down_in_view * self.get_text_font().row_height());
        let rect = BoundingBox::new(min, max).translate(Vec2i::new(0, -3));

        render_infos.push(rect);
        rows_down_in_view += 1;
        if lines_contents.len() > 2 {
            let last_line_content = lines_contents.pop().unwrap();
            for &l in lines_contents.iter().skip(1) {
                let line_width = gltxt::calculate_text_dimensions(&l, self.edit_font.as_ref()).width;
                let min = Vec2i::new(0, 0 - (rows_down_in_view + 1) * self.get_text_font().row_height());
                let max = Vec2i::new(line_width + self.get_text_font().get_max_glyph_width() - 2, 0 - rows_down_in_view * self.get_text_font().row_height());
                let line_bb = BoundingBox::new(min, max).translate(Vec2i::new(0, -3));
                render_infos.push(line_bb);
                rows_down_in_view += 1;
            }
            let line_width = gltxt::calculate_text_dimensions(&last_line_content[0..last_selected_col_position], self.edit_font.as_ref()).width;
            let min = Vec2i::new(0, 0 - (rows_down_in_view + 1) * self.get_text_font().row_height());
            let max = Vec2i::new(line_width + self.get_text_font().get_max_glyph_width() - 2, 0 - rows_down_in_view * self.get_text_font().row_height());
            let line_bb = BoundingBox::new(min, max).translate(Vec2i::new(0, -3));
            render_infos.push(line_bb);
        } else {
            let last_line_content = lines_contents.pop().unwrap();
            let line_width = gltxt::calculate_text_dimensions(&last_line_content[0..last_selected_col_position], self.edit_font.as_ref()).width;
            let min = Vec2i::new(0, 0 - (rows_down_in_view + 1) * self.get_text_font().row_height());
            let max = Vec2i::new(line_width + self.get_text_font().get_max_glyph_width() - 2, 0 - rows_down_in_view * self.get_text_font().row_height());
            let line_bb = BoundingBox::new(min, max).translate(Vec2i::new(0, -3));
            render_infos.push(line_bb);
        }
        render_infos
    }

    pub fn draw_title(&mut self, title: &str) {
        let Vec2i { x: tx, y: ty } = self.title_frame.anchor;
        self.text_renderer
            .push_draw_command(title.chars().map(|c| c), RGBColor::white(), tx + 3, ty, self.get_title_font());
    }

    pub fn load_file(&mut self, path: &Path) {
        Assert!(self.buffer.empty(), "View must be empty in order to load data from file");
        if self.buffer.empty() {
            self.buffer.load_file(path);
            self.set_view_on_buffer_cursor();
        }
        self.scroll_bar.set_max(self.buffer.meta_data().line_count())
    }

    pub fn insert_ch(&mut self, ch: char) {
        if input_not_valid(ch) {
            return;
        }

        self.buffer.insert(ch, true);
        if self.buffer.cursor_row() >= Line((self.topmost_line_in_buffer + self.rows_displayable()) as _) {
            self.set_view_on_buffer_cursor();
        } else {
            self.buffer_in_view.end += 1;
            self.view_changed = true;
        }
        self.scroll_bar.set_max(self.buffer.meta_data().line_count());
    }

    /// Sets the view of the buffer, so that it "sees" the buffer cursor.
    /// This will be called quite often, since what we edit, is what we should see in the view.
    /// So this should get called whenever the buffer cursor moves.
    pub fn set_view_on_buffer_cursor(&mut self) {
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
        self.scroll_bar.set_max(self.buffer.meta_data().line_count());
        self.scroll_bar.scroll_value = self.topmost_line_in_buffer as _;
        self.scroll_bar.update_ui_position_by_value();
        self.view_changed = true;
    }

    pub fn insert_slice(&mut self, s: &[char]) {
        self.buffer.insert_slice(s);
        self.text_renderer.pristine = false;
        self.validate_range();
        self.set_view_on_buffer_cursor();
    }

    pub fn insert_str(&mut self, s: &str) {
        let d: Vec<_> = s.chars().collect();
        self.buffer_in_view = 0..s.len();
        self.buffer.insert_slice(&d[..]);
        self.text_renderer.pristine = false;
        self.set_view_on_buffer_cursor();
    }

    pub fn cursor_goto(&mut self, pos: Index) {
        self.buffer.cursor_goto(pos);
        self.set_view_on_buffer_cursor();
    }
    pub fn move_cursor(&mut self, dir: Movement) {
        let translated = dir.transform_page_param(self.rows_displayable() as _);
        self.buffer.move_cursor(translated);
        self.set_view_on_buffer_cursor();
    }

    pub fn delete(&mut self, dir: Movement) {
        self.buffer.delete(dir);
        self.view_changed = true;
        self.validate_range();
        self.set_view_on_buffer_cursor();
    }

    pub fn backspace_handle(&mut self, kind: TextKind) {
        match kind {
            TextKind::Char => self.buffer.delete(Movement::Backward(TextKind::Char, 1)),
            TextKind::Word => self.buffer.delete(Movement::Backward(TextKind::Word, 1)),
            TextKind::Line => self.buffer.delete(Movement::Backward(TextKind::Line, 1)),
            TextKind::Block => self.buffer.delete(Movement::Backward(TextKind::Block, 1)),
            _ => {
                todo!("TextKind::{:?} not yet implemented", kind)
            }
        }
        self.view_changed = true;
        self.validate_range();
        self.set_view_on_buffer_cursor();
    }

    pub fn validate_range(&mut self) {
        if self.buffer_in_view.end > self.buffer.len() {
            self.buffer_in_view.end = self.buffer.len();
        }
    }

    pub fn id(&self) -> ViewId {
        self.id
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
            width: self.title_frame.size.width,
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
        size.width -= View::SCROLL_BAR_WIDTH;
        self.view_frame.anchor.y = self.title_frame.anchor.y - self.title_frame.size.height;
        // self.view_frame.anchor = self.title_frame.anchor + Vec2i::new(0, -self.row_height - 5);
        self.view_frame.size = size;
        assert_eq!(self.view_frame.anchor, self.title_frame.anchor + Vec2i::new(0, -self.get_title_font().row_height() - 5));
        let sb_frame =
            Frame::new(self.view_frame.anchor + Vec2i::new(self.view_frame.size.width, 0), Size::new(View::SCROLL_BAR_WIDTH, self.view_frame.size.height));
        self.scroll_bar.frame = sb_frame;
        self.scroll_bar.ui_update();
        self.scroll_bar.set_max(self.buffer.meta_data().line_count());
    }

    fn set_anchor(&mut self, anchor: Vec2i) {
        self.title_frame.anchor = anchor;
        self.view_frame.anchor = self.title_frame.anchor + Vec2i::new(0, -self.title_frame.size.height);
        self.scroll_bar.frame.anchor = self.view_frame.anchor + Vec2i::new(self.view_frame.width(), 0);
        self.scroll_bar.ui_update();
    }

    fn bounding_box(&self) -> BoundingBox {
        self.total_boundingbox()
    }

    fn mouse_clicked(&mut self, validated_inside_pos: Vec2i) {
        Assert!(self.bounding_box().box_hit_check(validated_inside_pos), "This coordinate is not enclosed by this view");
        // means we clicked the title frame, we do not need to scan where the buffer cursor should land, we only need to activate the view
        if BoundingBox::from_frame(&self.title_frame).box_hit_check(validated_inside_pos) {
        } else if self.scroll_bar.frame.to_bb().box_hit_check(validated_inside_pos) {
            self.scroll_bar_interacting = true;
            // if we clicked on scroll bar, but not on slider, we want the slider to jump to this location
            if !self.scroll_bar.slider.to_bb().box_hit_check(validated_inside_pos) {
                self.scroll_bar.scroll_to_ui_pos(validated_inside_pos);
                let md = self.buffer.meta_data();
                let buf_view_begin = *self
                    .buffer
                    .meta_data()
                    .get_line_start_index(Line(self.scroll_bar.scroll_value.clamp(0, md.line_count() - 1)))
                    .unwrap();
                let buf_view_end = self
                    .buffer
                    .meta_data()
                    .get_line_start_index(Line(self.scroll_bar.scroll_value).offset(self.rows_displayable() as _))
                    .map_or(self.buffer.len(), |v| *v);

                self.buffer_in_view = buf_view_begin..buf_view_end;
                self.topmost_line_in_buffer = self.scroll_bar.scroll_value as i32;
                self.view_changed = true;
                self.set_need_redraw();
            }
        } else {
            self.buffer.meta_cursor = None;
            if let Some(final_index_pos) = self.mouse_to_buffer_position(validated_inside_pos) {
                self.cursor_goto(final_index_pos);
            }
        }
    }

    fn mouse_dragged(&mut self, begin_coordinate: Vec2i, current_coordinate: Vec2i) -> Option<Vec2i> {
        if !self.scroll_bar_interacting {
            if let Some((begin_coord_idx, target_coord_idx)) = self
                .mouse_to_buffer_position(begin_coordinate)
                .zip(self.mouse_to_buffer_position(current_coordinate))
            {
                match self.buffer.meta_cursor {
                    Some(MetaCursor::Absolute(..)) => {
                        self.buffer.cursor_goto(target_coord_idx);
                    }
                    _ => {
                        self.buffer.cursor_goto(target_coord_idx);
                        self.buffer.meta_cursor = Some(MetaCursor::Absolute(begin_coord_idx));
                    }
                }
                self.set_view_on_buffer_cursor();
            }
            None
        } else {
            match self.scroll_bar.layout {
                ScrollBarLayout::Horizontal => todo!(),
                ScrollBarLayout::Vertical => {
                    let diff = current_coordinate.y - begin_coordinate.y;
                    self.scroll_bar.scroll_by(diff);
                    let md = self.buffer.meta_data();
                    let buf_view_begin = *self
                        .buffer
                        .meta_data()
                        .get_line_start_index(Line(self.scroll_bar.scroll_value.clamp(0, md.line_count() - 1)))
                        .unwrap();
                    let buf_view_end = self
                        .buffer
                        .meta_data()
                        .get_line_start_index(Line(self.scroll_bar.scroll_value).offset(self.rows_displayable() as _))
                        .map_or(self.buffer.len(), |v| *v);

                    self.buffer_in_view = buf_view_begin..buf_view_end;
                    // fuck me this is a cryptic name. But it means "the most scrollable down to line"
                    // so if we have 30 lines, indexed by 0-29, the *absolute* most we can scroll down to, is 29, thus showing only the 30th line
                    // In the view. We can not scroll down further, since this would mean, the cursor disappear and we'd be typing off view and all hell would break loose
                    // as far as layout
                    let top_mostable_line = (md.line_count() - 1) as i32;
                    self.topmost_line_in_buffer = std::cmp::min(top_mostable_line, self.scroll_bar.scroll_value as i32);
                    self.view_changed = true;
                    Some(current_coordinate)
                }
            }
        }
        /*
        if let Some((begin_coord_idx, target_coord_idx)) = self
            .mouse_to_buffer_position(begin_coordinate)
            .zip(self.mouse_to_buffer_position(current_coordinate))
        {
            match self.buffer.meta_cursor {
                Some(MetaCursor::Absolute(..)) => {
                    self.buffer.cursor_goto(target_coord_idx);
                }
                _ => {
                    self.buffer.cursor_goto(target_coord_idx);
                    self.buffer.meta_cursor = Some(MetaCursor::Absolute(begin_coord_idx));
                }
            }
            self.set_view_on_buffer_cursor();
            None
        } else if self.scroll_bar.frame.to_bb().box_hit_check(begin_coordinate) {
            match self.scroll_bar.layout {
                ScrollBarLayout::Horizontal => todo!(),
                ScrollBarLayout::Vertical => {
                    let translated = Vec2i::new(self.scroll_bar.frame.anchor.x, current_coordinate.y);
                    println!("Scrollbar {:?} - Current coord: {:?} (Begin coord: {:?}", self.scroll_bar.slider.anchor, current_coordinate, begin_coordinate);
                    let diff = current_coordinate.y - begin_coordinate.y;
                    // self.scroll_bar.scroll_to_ui_pos(translated);
                    self.scroll_bar.scroll_by(diff);
                    let md = self.buffer.meta_data();
                    let buf_view_begin = *self
                        .buffer
                        .meta_data()
                        .get_line_start_index(Line(self.scroll_bar.scroll_value.clamp(0, md.line_count() - 1)))
                        .unwrap();
                    let buf_view_end = self
                        .buffer
                        .meta_data()
                        .get_line_start_index(Line(self.scroll_bar.scroll_value).offset(self.rows_displayable() as _))
                        .map_or(self.buffer.len(), |v| *v);

                    self.buffer_in_view = buf_view_begin..buf_view_end;
                    self.topmost_line_in_buffer = self.scroll_bar.scroll_value as i32;
                    self.view_changed = true;
                    Some(current_coordinate)
                }
            }
        } else {
            self.buffer.meta_cursor = None;
            None
        }
        */
    }
}
