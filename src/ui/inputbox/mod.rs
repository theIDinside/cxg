pub mod line_text_box;
pub mod listbox;

use line_text_box::LineTextBox;
use listbox::ListBox;

use std::rc::Rc;

use walkdir::WalkDir;

use super::{
    boundingbox::BoundingBox,
    coordinate::*,
    eventhandling::event::InputBehavior,
    font::Font,
    frame::{make_inner_frame, Frame},
    Viewable, ACTIVE_VIEW_BACKGROUND,
};
use crate::{
    cmd::{commands_matching, get_command, CommandTag, COMMAND_NAMES},
    datastructure::generic::Vec2i,
    opengl::{
        rectangle_renderer::RectRenderer,
        shaders::{RectShader, TextShader},
        text_renderer::{self, TextRenderer},
        types::{RGBAColor, RGBColor},
    },
    ui::eventhandling::event::CommandOutput,
};

pub struct TextRenderSetting {
    _scale: f32,
    text_color: RGBColor,
}

impl TextRenderSetting {
    pub fn new(scale: f32, text_color: RGBColor) -> TextRenderSetting {
        TextRenderSetting { _scale: scale, text_color }
    }
}

impl Default for TextRenderSetting {
    fn default() -> Self {
        TextRenderSetting { _scale: 1.0, text_color: RGBColor { r: 0.0, g: 1.0, b: 1.0 } }
    }
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    /// Mode when we are typing in a command and showing a list of commands matching that input
    CommandList,
    /// Mode where we are inputing parameters for actual commands
    CommandInput(CommandTag),
}

const INPUT_BOX_MSG: &str = "Search by file name in project folder...";

pub struct InputBox {
    /// Contains the user input. Might as well use String, input won't be long and this is just easier
    pub visible: bool,
    pub input_box: LineTextBox,
    pub selection_list: ListBox,
    pub frame: Frame,
    text_renderer: TextRenderer,
    rect_renderer: RectRenderer,
    pub mode: Mode,
    pub needs_update: bool,
    font: Rc<Font>,
}

impl InputBox {
    pub fn new(frame: Frame, font: Rc<Font>, font_shader: &TextShader, rect_shader: &RectShader) -> InputBox {
        let (text_renderer, rect_renderer) = (TextRenderer::create(font_shader.clone(), 1024 * 10), RectRenderer::create(rect_shader.clone(), 8 * 60));

        let margin = 2;
        let input_box_frame = Frame { anchor: frame.anchor, size: Size::new(frame.size.width, font.row_height() + margin * 4) };
        let input_inner_frame = make_inner_frame(&input_box_frame, margin);
        let ltb = LineTextBox::new(input_box_frame, input_inner_frame, None);

        let list_box_frame = Frame {
            anchor: frame.anchor + Vec2i::new(0, -input_box_frame.size.height),
            size: Size { width: frame.size.width, height: frame.size.height - input_box_frame.size.height },
        };
        let lb = ListBox::new(list_box_frame, font.row_height(), Some((TextRenderSetting::new(1.0, RGBColor::white()), ACTIVE_VIEW_BACKGROUND)));

        InputBox {
            input_box: ltb,
            selection_list: lb,
            visible: false,
            frame,
            text_renderer,
            rect_renderer,
            mode: Mode::CommandInput(CommandTag::Goto),
            needs_update: true,
            font,
        }
    }

    pub fn open(&mut self, mode: Mode) {
        self.mode = mode;
        self.needs_update = true;
        self.draw();
    }

    /// updates the list of possible selections that contains what the user has input into the
    /// input box.
    pub fn update_list_of_files(&mut self) {
        let name = &self.input_box.data.iter().collect::<String>();
        self.selection_list.data = WalkDir::new(".")
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| {
                let e = e.unwrap();
                // this is *odd* behavior. When we pass in a slice to contains(...)
                // it will return true if *any* of the elements in that slice, exists in the string
                if e.path().to_str().unwrap().to_ascii_uppercase().contains(&name.to_uppercase()) {
                    Some(e)
                } else {
                    None
                }
            })
            .map(|de| de.path().display().to_string().chars().collect())
            .collect();
    }

    pub fn update_list_of_commands(&mut self) {
        let name = &self.input_box.data.iter().collect::<String>();
        if !name.is_empty() {
            if let Some(matches) = commands_matching(name) {
                self.selection_list.data = matches.iter().map(|c| CommandTag::name(**c).chars().collect()).collect();
                self.selection_list.selection = Some(0);
            } else {
                self.selection_list.clear();
            }
        } else {
            self.selection_list.selection = Some(0);
            self.selection_list.data = COMMAND_NAMES
                .iter()
                .map(|(_, &tag)| CommandTag::name(tag).chars().collect())
                .collect();
            assert_ne!(0, self.selection_list.data.len());
        }

        self.needs_update = true;
    }

    pub fn draw(&mut self) {
        if !self.visible {
            return;
        }
        /*
            Steps of drawing:
                - prior steps already done. Removed from this list.
                4. todo(ui_feature): add scroll bar functionality to ListBox
        */

        if self.needs_update {
            self.text_renderer.clear_data();
            self.rect_renderer.clear_data();

            match self.mode {
                Mode::CommandInput(cmd) => match cmd {
                    CommandTag::Goto => {
                        self.draw_without_list(cmd);
                    }
                    CommandTag::GotoInFile => todo!(),
                    CommandTag::Find => {
                        self.draw_without_list(cmd);
                    }
                    CommandTag::OpenFile => {
                        self.draw_with_list();
                    }
                    CommandTag::SaveFile => {
                        self.draw_without_list(cmd);
                    }
                    CommandTag::SetFontSize => {
                        self.draw_without_list(cmd);
                    }
                },
                Mode::CommandList => {
                    self.draw_with_list();
                }
            }

            self.needs_update = false;
        }

        self.render_cursor();
        self.rect_renderer.draw();
        self.text_renderer.draw_clipped_list(self.frame);
    }

    fn render_cursor(&mut self) {
        let cursor = self.input_box.cursor;
        let cursor_start = text_renderer::calculate_text_dimensions(&self.input_box.data[..cursor], self.font.as_ref());
        let mut cursor_col = RGBAColor::red();
        cursor_col.a = 0.01;
        let t = BoundingBox::from_frame(&self.input_box.inner_frame);
        let min = Vec2i::new(t.min.x + cursor_start.width, t.min.y + 2);
        let max = min + Vec2i::new(2, cursor_start.height);
        let bb = BoundingBox::new(min, max);
        self.rect_renderer.add_rect(bb, cursor_col);
    }

    fn draw_without_list(&mut self, cmd: CommandTag) {
        const MARGIN: i32 = 2;
        let color = self.input_box.text_render_settings.text_color;
        let input_box_frame = Frame {
            anchor: self.frame.anchor,
            size: Size::new(self.frame.size.width, self.font.row_height() + MARGIN * 4),
        };
        let mut frame_bb = BoundingBox::from_frame(&self.frame);
        frame_bb.min.y = frame_bb.max.y - self.font.row_height() + MARGIN * 4;

        // self.rect_renderer.add_rect(frame_bb, self.selection_list.background_color);

        let input_inner_frame = make_inner_frame(&input_box_frame, MARGIN);

        let text_area_frame = BoundingBox::from_frame(&input_box_frame);
        let text_area = BoundingBox::from_frame(&input_inner_frame);
        let white_border_bb = BoundingBox::expand(&text_area_frame, Margin::Perpendicular { h: 2, v: 2 });
        self.rect_renderer.add_rect(white_border_bb, RGBAColor::gray());
        // frame color, of border around user input text box
        let input_textbox_frame_color = RGBAColor { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };
        // the background color behind the user typed text
        let input_textbox_color = RGBAColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        self.rect_renderer.add_rect(text_area_frame, input_textbox_frame_color);
        self.rect_renderer.add_rect(text_area.clone(), input_textbox_color);

        let text_top_left_anchor = Vec2i::new(text_area.min.x, text_area.max.y);
        if !self.input_box.data.is_empty() {
            self.text_renderer.push_draw_command(
                self.input_box.data.iter().map(|c| *c),
                color,
                text_top_left_anchor.x,
                text_top_left_anchor.y,
                self.font.clone(),
            );
        } else {
            let msg: &'static str = CommandTag::description(cmd);
            self.text_renderer
                .push_draw_command(msg.chars(), color, text_top_left_anchor.x, text_top_left_anchor.y, self.font.clone());
        }
    }

    fn draw_with_list(&mut self) {
        let list_items = self.selection_list.data.len();
        let max_height = if list_items < ListBox::MAX_DISPLAYABLE_ITEMS_HINT {
            // todo: I have to start thinking about re-designing some of the UI elements so I don't have to rely on these magical
            // align constants. This +5, is so that the last item in the list of choices doesn't get hidden.
            // When there's too many choices, that's no problem, but when it's only 3 choices, yet the 3rd gets hidden is not good
            // so this +5 makes sure there's room for the last element.
            list_items as i32 * self.selection_list.item_height + 5
        } else {
            ListBox::MAX_DISPLAYABLE_ITEMS_HINT as i32 * self.selection_list.item_height
        };

        let mut frame_bb = BoundingBox::from_frame(&self.frame);
        let sz = frame_bb.size();
        let diff = crate::diff!(sz.height, max_height) - self.input_box.outer_frame.size.height as usize;
        frame_bb.min.y += diff as i32;

        static BORDER_SIZE: i32 = 2;
        let frame_border_bb = BoundingBox::expand(&frame_bb, Margin::Perpendicular { v: BORDER_SIZE, h: BORDER_SIZE });

        self.rect_renderer.add_rect(frame_border_bb, RGBAColor::white());
        self.rect_renderer.add_rect(frame_bb, self.selection_list.background_color);
        let ltb_frame = BoundingBox::from_frame(&self.input_box.outer_frame);
        let ltb_inner_frame = BoundingBox::from_frame(&self.input_box.inner_frame);
        let t = ltb_inner_frame.clone();

        // frame color, of border around user input text box
        let input_textbox_frame_color = RGBAColor { r: 1.0, g: 0.5, b: 0.0, a: 1.0 };
        // the background color behind the user typed text
        let input_textbox_color = RGBAColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        self.rect_renderer.add_rect(ltb_frame, input_textbox_frame_color);
        self.rect_renderer.add_rect(ltb_inner_frame, input_textbox_color);

        let color = self.input_box.text_render_settings.text_color;
        if !self.input_box.data.is_empty() || !self.selection_list.data.is_empty() {
            self.text_renderer
                .push_draw_command(self.input_box.data.iter().map(|c| *c), color, t.min.x, t.max.y, self.font.clone());
            let color = self.selection_list.text_render_settings.text_color;

            // the bottom edge of each list item in the list box. Decreases with font.row_height() per list item
            let mut list_item_y_anchor = t.min.y;

            let step = self.selection_list.item_height;
            let mut dy = 0;
            let items: Vec<&Vec<char>> = self
                .selection_list
                .data
                .iter()
                .take_while(|_| {
                    dy += step;
                    max_height > dy
                })
                .collect();

            let selected = self.selection_list.selection.unwrap_or(0);
            for (index, item) in items.into_iter().enumerate() {
                if selected == index {
                    let Vec2i { x, .. } = self.selection_list.frame.anchor;
                    let min = Vec2i::new(x, list_item_y_anchor - self.selection_list.item_height);
                    let max = Vec2i::new(x + self.selection_list.frame.size.width, list_item_y_anchor);
                    let mut selection_box = BoundingBox::new(min, max);
                    // we need to "align" the rendered selection box for one major reason;
                    // even though each line, has a y-anchor (bottom edge), some characters in the font set
                    // will have different bearings (start lower, like g, j etc) so they won't be encompassed entirely
                    // by the selection line. This looks bad. So what I do, is instead calculate exactly where the middle of
                    // each list line, and align the bounding box vertically there. It's a bit hackish, but it is what it is.
                    let align_y = list_item_y_anchor - self.font.row_height() + self.font.row_height() / 2 - 3;
                    selection_box.center_vertical_align(align_y);
                    self.rect_renderer.add_rect(selection_box, RGBAColor::new(0.0, 0.65, 0.5, 1.0));
                }

                self.text_renderer
                    .push_draw_command(item.iter().map(|c| *c), color, t.min.x, list_item_y_anchor, self.font.clone());
                list_item_y_anchor -= self.selection_list.item_height;
            }
        } else {
            let color = RGBColor { r: 0.5, g: 0.5, b: 0.5 };
            self.text_renderer
                .push_draw_command(INPUT_BOX_MSG.chars(), color, t.min.x, t.max.y, self.font.clone());
        }
    }

    pub fn clear(&mut self) {
        self.selection_list.clear();
        self.input_box.clear();
        self.needs_update = true;
    }

    fn process_input(&mut self) -> CommandOutput {
        match self.mode {
            Mode::CommandInput(cmd) => match cmd {
                CommandTag::Goto => self
                    .input_box
                    .data
                    .iter()
                    .collect::<String>()
                    .parse()
                    .map(|v| CommandOutput::Goto(v))
                    .unwrap_or(CommandOutput::None),
                CommandTag::Find => CommandOutput::Find(self.input_box.data.iter().collect::<String>()),
                CommandTag::GotoInFile => todo!(),
                CommandTag::OpenFile => todo!(),
                CommandTag::SaveFile => todo!(),
                CommandTag::SetFontSize => todo!(),
            },
            Mode::CommandList => {
                if let Some(item) = self.selection_list.pop_selected() {
                    get_command(&item.iter().collect::<String>())
                        .map(|c| CommandOutput::CommandSelection(*c))
                        .unwrap_or(CommandOutput::None)
                } else {
                    CommandOutput::None
                }
            }
        }
    }

    pub fn update(&mut self) {
        match self.mode {
            Mode::CommandInput(_c) => match _c {
                // these need no interactive updating
                CommandTag::Goto | CommandTag::GotoInFile | CommandTag::Find | CommandTag::SaveFile | CommandTag::SetFontSize => {}
                // these need interactive updating
                CommandTag::OpenFile => self.update_list_of_files(),
            },
            Mode::CommandList => {
                self.update_list_of_commands();
            }
        }
        self.input_box.cursor = self.input_box.cursor.clamp(0, self.input_box.data.len());
        self.needs_update = true;
    }
}

impl InputBehavior for InputBox {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, _modifier: glfw::Modifiers) -> CommandOutput {
        self.selection_list.selection = self.selection_list.selection.or_else(|| Some(0));
        let key_pressed = || action == glfw::Action::Press || action == glfw::Action::Repeat;
        let response = match key {
            glfw::Key::Backspace if key_pressed() => {
                if let Some(_) = self.input_box.data.pop() {
                    self.input_box.cursor -= 1;
                }
                if self.input_box.data.is_empty() {
                    self.selection_list.data.clear();
                } else {
                    self.update_list_of_files();
                }
                CommandOutput::None
            }
            glfw::Key::Up if key_pressed() => {
                self.selection_list.scroll_selection_up();
                CommandOutput::None
            }
            glfw::Key::Down if key_pressed() => {
                self.selection_list.scroll_selection_down();
                CommandOutput::None
            }
            glfw::Key::Enter if key_pressed() => self.process_input(),
            _ => CommandOutput::None,
        };
        self.needs_update = true;
        response
    }

    fn handle_char(&mut self, ch: char) {
        self.input_box.data.insert(self.input_box.cursor, ch);
        self.input_box.cursor += 1;
        self.selection_list.selection = None;
        match self.mode {
            Mode::CommandInput(_cmd) => match _cmd {
                // these do not need interactive updating of the list
                CommandTag::SaveFile | CommandTag::Goto | CommandTag::GotoInFile | CommandTag::Find | CommandTag::SetFontSize => {}
                // these need interactive updating the of the list
                CommandTag::OpenFile => self.update_list_of_files(),
            },
            Mode::CommandList => {
                self.update_list_of_commands();
            }
        }
        if !self.selection_list.data.is_empty() {
            self.selection_list.selection = Some(0);
        }
        self.needs_update = true;
    }

    fn get_uid(&self) -> Option<super::UID> {
        todo!()
    }

    fn move_cursor(&mut self, movement: crate::textbuffer::Movement) {
        match movement {
            crate::textbuffer::Movement::Forward(kind, _) => match kind {
                crate::textbuffer::TextKind::Line => {
                    self.selection_list.scroll_selection_down();
                }
                _ => {}
            },
            crate::textbuffer::Movement::Backward(kind, _) => match kind {
                crate::textbuffer::TextKind::Line => {
                    self.selection_list.scroll_selection_up();
                }
                _ => {}
            },
            crate::textbuffer::Movement::Begin(..) => {
                self.input_box.cursor = 0;
            }
            crate::textbuffer::Movement::End(..) => {
                self.input_box.cursor = self.input_box.data.len();
            }
        }
        self.needs_update = true;
    }

    fn context(&self) -> super::eventhandling::input::KeyboardInputContext {
        super::eventhandling::input::KeyboardInputContext::InputBox
    }

    fn select_move_cursor(&mut self, _movement: crate::textbuffer::Movement) {
        todo!()
    }

    fn delete(&mut self, _movement: crate::textbuffer::Movement) {
        match _movement {
            crate::textbuffer::Movement::Forward(.., _) => {}
            crate::textbuffer::Movement::Backward(.., _) => {
                if let Some(_) = self.input_box.data.pop() {
                    self.input_box.cursor -= 1;
                }
                if self.input_box.data.is_empty() {
                    self.selection_list.data.clear();
                } else {
                    self.update_list_of_files();
                }
            }
            crate::textbuffer::Movement::Begin(_) => {
                self.input_box.data.clear();
                self.input_box.cursor = 0;
                self.update_list_of_files();
            }
            crate::textbuffer::Movement::End(_) => {}
        }
        self.needs_update = true;
    }

    fn copy(&self) -> Option<String> {
        todo!()
    }

    fn cut(&self) -> Option<String> {
        todo!()
    }
}

impl Viewable for InputBox {
    fn resize(&mut self, size: Size) {
        let margin = 4;
        self.frame.size = size;

        let input_box_frame = Frame {
            anchor: self.frame.anchor,
            size: Size::new(self.frame.size.width, self.selection_list.item_height + margin * 4),
        };
        let input_inner_frame = make_inner_frame(&input_box_frame, margin);
        self.input_box.outer_frame = input_box_frame;
        self.input_box.inner_frame = input_inner_frame;

        let list_box_frame = Frame {
            anchor: self.frame.anchor + Vec2i::new(0, -input_box_frame.size.height),
            size: Size { width: self.frame.size.width, height: self.frame.size.height - input_box_frame.size.height },
        };
        self.selection_list.frame = list_box_frame;
        self.needs_update = true;
    }

    fn set_anchor(&mut self, anchor: Vec2i) {
        self.frame.anchor = anchor;

        let margin = 4;
        let input_box_frame = Frame {
            anchor: self.frame.anchor,
            size: Size::new(self.frame.size.width, self.selection_list.item_height + margin * 4),
        };
        let input_inner_frame = make_inner_frame(&input_box_frame, margin);
        self.input_box.outer_frame = input_box_frame;
        self.input_box.inner_frame = input_inner_frame;

        let list_box_frame = Frame {
            anchor: self.frame.anchor + Vec2i::new(0, -input_box_frame.size.height),
            size: Size { width: self.frame.size.width, height: self.frame.size.height - input_box_frame.size.height },
        };
        self.selection_list.frame = list_box_frame;
        self.needs_update = true;
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_frame(&self.frame)
    }

    fn mouse_clicked(&mut self, _screen_coordinate: Vec2i) {
        todo!()
    }

    fn mouse_dragged(&mut self, _begin_coordinate: Vec2i, _current_coordinated: Vec2i) -> Option<Vec2i> {
        todo!()
    }
}
