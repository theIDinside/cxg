use walkdir::WalkDir;

use super::{
    boundingbox::BoundingBox,
    coordinate::*,
    eventhandling::event::Input,
    font::Font,
    frame::{make_inner_frame, Frame},
    input::line_text_box::LineTextBox,
    ACTIVE_VIEW_BACKGROUND,
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
    scale: f32,
    text_color: RGBColor,
}

impl TextRenderSetting {
    pub fn new(scale: f32, text_color: RGBColor) -> TextRenderSetting {
        TextRenderSetting { scale, text_color }
    }
}

impl Default for TextRenderSetting {
    fn default() -> Self {
        TextRenderSetting { scale: 1.0, text_color: RGBColor { r: 0.0, g: 1.0, b: 1.0 } }
    }
}

#[derive(PartialEq, Eq)]
pub enum InputBoxMode {
    // todo(feature): add SymbolList
    Command,
    FileList,
}

/// POD data type ListBox. These do not define behavior in any real sense. They just hold the data
/// that InputBox displays. Therefore the behaviors is defined in that struct impl.
pub struct ListBox {
    pub data: Vec<Vec<char>>,
    pub selection: Option<usize>,
    pub frame: Frame,
    pub text_render_settings: TextRenderSetting,
    pub background_color: RGBAColor,
    pub item_height: i32,
}

impl ListBox {
    pub fn new(frame: Frame, list_item_height: i32, render_config: Option<(TextRenderSetting, RGBAColor)>) -> ListBox {
        let (text_render_settings, background_color) = render_config.unwrap_or((TextRenderSetting::default(), ACTIVE_VIEW_BACKGROUND));
        ListBox {
            data: Vec::with_capacity(10),
            selection: None,
            frame,
            text_render_settings,
            background_color,
            item_height: list_item_height,
        }
    }
}

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

        let margin = 4;
        let input_box_frame = Frame { anchor: frame.anchor, size: Size::new(frame.size.width, font.row_height() + margin * 4) };
        let input_inner_frame = make_inner_frame(&input_box_frame, margin);
        let ltb = LineTextBox::new(input_box_frame, input_inner_frame, None);

        let list_box_frame = Frame {
            anchor: Anchor::vector_add(frame.anchor, Vec2i::new(0, -input_box_frame.size.height)),
            size: Size { width: frame.size.width, height: frame.size.height - input_box_frame.size.height },
        };
        let lb = ListBox::new(list_box_frame, font.row_height(), None);

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

    pub fn set_anchor(&mut self, anchor: Anchor) {
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
            anchor: Anchor::vector_add(self.frame.anchor, Vec2i::new(0, -input_box_frame.size.height)),
            size: Size { width: self.frame.size.width, height: self.frame.size.height - input_box_frame.size.height },
        };
        self.selection_list.frame = list_box_frame;

        self.needs_update = true;
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

    pub fn draw(&mut self) {
        /*
            Steps of drawing:
                1. Draw the outer frame box
                2a. Draw the LineTextBox outer frame
                2b. Draw the LineTextBox inner frame
                2c. Draw the text contents of LineTextBox on top of inner frame
                ^---- Done this far ----^
                2d. Draw text cursor on top of text contents, on top of inner frame
                3a. Draw ListBox frame
                3b. On top of ListBox frame, iterate over items and draw their text contents respectively, on separate lines
                todo(ui_feature): add scroll bar functionality to ListBox
        */

        if self.needs_update {
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
            self.text_renderer
                .prepare_data_from_iterator(self.input_box.data.iter(), color, t.min.x, t.max.y);
            let color = self.selection_list.text_render_settings.text_color;

            let mut displace_y = 0;

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

            for item in items.into_iter() {
                self.text_renderer
                    .append_data_from_iterator(item.iter(), color, t.min.x, t.min.y - displace_y);
                displace_y += self.selection_list.item_height;
            }
            self.needs_update = false;
        }

        self.rect_renderer.draw();
        self.text_renderer.draw_clipped(self.frame);
    }

    pub fn render(&mut self) {}
}

impl<'app> Input for InputBox<'app> {
    fn handle_key(&mut self, key: glfw::Key, action: glfw::Action, _modifier: glfw::Modifiers) -> InputResponse {
        let key_pressed = || action == glfw::Action::Press;
        match key {
            glfw::Key::Backspace if key_pressed() => {
                if let Some(_) = self.input_box.data.pop() {
                    self.input_box.cursor -= 1;
                    let found_files: Vec<Vec<char>> = WalkDir::new(".")
                        .into_iter()
                        .filter_map(|e| {
                            let e = e.unwrap();
                            if e.path().to_str().unwrap().contains(&self.input_box.data[..]) {
                                Some(e)
                            } else {
                                None
                            }
                        })
                        .map(|de| de.path().display().to_string().chars().collect())
                        .collect();
                    self.selection_list.data = found_files;
                }

                if self.input_box.data.is_empty() {
                    self.selection_list.data.clear();
                }
            }
            glfw::Key::Up => {}
            glfw::Key::Down => {}
            glfw::Key::Enter => match self.mode {
                InputBoxMode::Command => {}
                InputBoxMode::FileList => {}
            },
            _ => {}
        }
        self.needs_update = true;
        InputResponse::None
    }

    fn handle_char(&mut self, ch: char) {
        self.input_box.data.insert(self.input_box.cursor, ch);
        self.input_box.cursor += 1;

        match self.mode {
            InputBoxMode::Command => {}
            InputBoxMode::FileList => {
                let found_files: Vec<Vec<char>> = WalkDir::new(".")
                    .into_iter()
                    .filter_map(|e| {
                        let e = e.unwrap();
                        // this is *odd* behavior. When we pass in a slice to contains(...)
                        // it will return true if *any* of the elements in that slice, exists in the string
                        if e.path().to_str().unwrap().contains(&self.input_box.data.iter().collect::<String>()) {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .map(|de| de.path().display().to_string().chars().collect())
                    .collect();
                self.selection_list.data = found_files;
            }
        }
        self.needs_update = true;
    }

    fn get_uid(&self) -> Option<super::UID> {
        todo!()
    }
}
