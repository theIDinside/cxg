pub mod line_text_box;
pub mod listbox;

use line_text_box::LineTextBox;
use listbox::ListBox;

use std::{iter::FromIterator, path::Path};

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
    datastructure::generic::Vec2i,
    opengl::{
        rect::RectRenderer,
        shaders::{RectShader, TextShader},
        text::TextRenderer,
        types::{RGBAColor, RGBColor},
    },
    ui::eventhandling::event::InputResponse,
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
pub enum InputBoxMode {
    // todo(feature): add SymbolList
    Command,
    FileList,
}

const INPUT_BOX_MSG: &str = "Search by file name in project folder...";

pub struct InputBox<'app> {
    /// Contains the user input. Might as well use String, input won't be long and this is just easier
    pub visible: bool,
    input_box: LineTextBox,
    selection_list: ListBox,
    pub frame: Frame,
    text_renderer: TextRenderer<'app>,
    rect_renderer: RectRenderer,
    pub mode: InputBoxMode,
    needs_update: bool,
}

impl<'app> InputBox<'app> {
    pub fn new(frame: Frame, font: &'app Font, font_shader: &TextShader, rect_shader: &RectShader) -> InputBox<'app> {
        let (text_renderer, rect_renderer) = (TextRenderer::create(font_shader.clone(), &font, 1024 * 10), RectRenderer::create(rect_shader.clone(), 8 * 60));

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
            mode: InputBoxMode::Command,
            needs_update: true,
        }
    }

    pub fn open(&mut self, mode: InputBoxMode) {
        match mode {
            InputBoxMode::Command => {}
            InputBoxMode::FileList => {}
        }
        if mode != self.mode {}
        self.mode = mode;
        self.needs_update = true;
        self.draw();
    }

    /// updates the list of possible selections that contains what the user has input into the
    /// input box.
    pub fn update_list(&mut self) {
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

    pub fn draw(&mut self) {
        if !self.visible {
            return;
        }
        /*
            Steps of drawing:
                1. Draw the outer frame box
                2a. Draw the LineTextBox outer frame
                2b. Draw the LineTextBox inner frame
                2c. Draw the text contents of LineTextBox on top of inner frame
                2d. Draw text cursor on top of text contents, on top of inner frame
                3a. Draw ListBox frame
                3b. On top of ListBox frame, iterate over items and draw their text contents respectively, on separate lines
                ^---- Done this far ----^
                4. todo(ui_feature): add scroll bar functionality to ListBox
        */

        if self.needs_update {
            self.text_renderer.clear_data();
            self.rect_renderer.clear_data();
            let frame_bb = BoundingBox::from_frame(&self.frame);
            let frame_border_bb = BoundingBox::expand(&frame_bb, Margin::Perpendicular { v: 2, h: 2 });
            self.rect_renderer.add_rect(frame_border_bb, RGBAColor::white());
            self.rect_renderer.add_rect(frame_bb, self.selection_list.background_color);
            let ltb_frame = BoundingBox::from_frame(&self.input_box.outer_frame);
            let ltb_inner_frame = BoundingBox::from_frame(&self.input_box.inner_frame);
            let t = ltb_inner_frame.clone();
            self.rect_renderer.add_rect(ltb_frame, RGBAColor { r: 1.0, g: 0.5, b: 0.0, a: 1.0 });
            self.rect_renderer
                .add_rect(ltb_inner_frame, RGBAColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });

            // Testing the drawing of text, inside the input box
            let color = self.input_box.text_render_settings.text_color;
            if !self.input_box.data.is_empty() {
                self.text_renderer
                    .push_draw_command(self.input_box.data.iter().map(|c| *c), color, t.min.x, t.max.y, self.text_renderer.font);
                let color = self.selection_list.text_render_settings.text_color;

                let mut displace_y = 3;

                let height = self.selection_list.frame.size.height;
                let step = self.selection_list.item_height;
                let mut dy = 0;
                let items: Vec<&Vec<char>> = self
                    .selection_list
                    .data
                    .iter()
                    .take_while(|_| {
                        dy += step;
                        height > dy
                    })
                    .collect();

                let selected = self.selection_list.selection.unwrap_or(0);
                let font = self.text_renderer.font;
                for (index, item) in items.into_iter().enumerate() {
                    if selected == index {
                        let Vec2i { x, .. } = self.selection_list.frame.anchor;
                        let min = Vec2i::new(x, t.min.y - displace_y - self.selection_list.item_height - 4);
                        let max = Vec2i::new(x + self.selection_list.frame.size.width, t.min.y - displace_y);
                        let selection_box = BoundingBox::new(min, max);
                        self.rect_renderer.add_rect(selection_box, RGBAColor::new(0.0, 0.65, 0.5, 1.0));
                    }

                    self.text_renderer
                        .push_draw_command(item.iter().map(|c| *c), color, t.min.x, t.min.y - displace_y, font);
                    displace_y += self.selection_list.item_height;
                }
            } else {
                let color = RGBColor { r: 0.5, g: 0.5, b: 0.5 };
                let font = self.text_renderer.font;
                self.text_renderer
                    .push_draw_command(INPUT_BOX_MSG.chars(), color, t.min.x, t.max.y, font);
            }
            self.needs_update = false;
        }

        self.rect_renderer.draw();
        self.text_renderer.draw_clipped_list(self.frame);
    }

    pub fn clear(&mut self) {
        self.selection_list.clear();
        self.input_box.clear();
        self.needs_update = true;
    }

    fn handle_file_selection(&mut self) -> InputResponse {
        if let Some(item) = self.selection_list.pop_selected() {
            let name = String::from_iter(&item);
            let p = Path::new(&name).to_path_buf();
            if p.is_dir() {
                self.input_box.data = item;
                self.input_box.data.push('/');
                self.input_box.cursor = self.input_box.data.len();
                self.selection_list.selection = None;
                self.update_list();
                self.needs_update = true;
                InputResponse::None
            } else {
                if p.exists() {
                    InputResponse::File(p)
                } else {
                    InputResponse::None
                }
            }
        } else {
            InputResponse::None
        }
    }
}

impl<'app> InputBehavior for InputBox<'app> {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, _modifier: glfw::Modifiers) -> InputResponse {
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
                    self.update_list();
                }
                InputResponse::None
            }
            glfw::Key::Up if key_pressed() => {
                self.selection_list.scroll_selection_up();
                InputResponse::None
            }
            glfw::Key::Down if key_pressed() => {
                self.selection_list.scroll_selection_down();
                InputResponse::None
            }
            glfw::Key::Enter if key_pressed() => match self.mode {
                InputBoxMode::Command => InputResponse::None,
                InputBoxMode::FileList => self.handle_file_selection(),
            },
            _ => InputResponse::None,
        };
        self.needs_update = true;
        response
    }

    fn handle_char(&mut self, ch: char) {
        self.input_box.data.insert(self.input_box.cursor, ch);
        self.input_box.cursor += 1;
        self.selection_list.selection = None;
        match self.mode {
            InputBoxMode::Command => {}
            InputBoxMode::FileList => {
                self.update_list();
            }
        }
        self.needs_update = true;
    }

    fn get_uid(&self) -> Option<super::UID> {
        todo!()
    }
}

impl<'app> Viewable for InputBox<'app> {
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
}
