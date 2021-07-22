use crate::opengl::types::RGBAColor;
// Default active background color
use crate::ui::ACTIVE_VIEW_BACKGROUND;

use super::Frame;
use super::TextRenderSetting;

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

    /// Returns selected item, if any selection has been made (and there's any available choices in the list)
    pub fn get_selected(&self) -> Option<&Vec<char>> {
        self.selection.and_then(|index| self.data.get(index))
    }

    pub fn pop_selected(&mut self) -> Option<Vec<char>> {
        self.selection
            .and_then(|index| if self.data.len() > index { Some(self.data.remove(index)) } else { None })
    }

    /// Resets the text input and the generated item choices
    pub fn clear(&mut self) {
        self.selection = None;
        self.data.clear();
    }

    pub fn scroll_selection_up(&mut self) {
        self.selection = self.selection.map(|f| if f == 0 { self.data.len() - 1 } else { f - 1 }).or(Some(0));
    }

    pub fn scroll_selection_down(&mut self) {
        self.selection = self.selection.map(|f| if f + 1 >= self.data.len() { 0 } else { f + 1 }).or(Some(0));
    }
}
