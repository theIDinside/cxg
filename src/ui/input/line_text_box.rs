use crate::{
    opengl::types::{RGBAColor, RGBColor},
    ui::{
        coordinate::Anchor,
        frame::{make_inner_frame, Frame},
        inputbox::TextRenderSetting,
        ACTIVE_VIEW_BACKGROUND,
    },
};

/// POD data type LineTextBox. These do not define behavior in any real sense. They just hold the data
/// that InputBox displays. Therefore the behaviors is defined in that struct impl.
pub struct LineTextBox {
    pub data: Vec<char>,
    pub cursor: usize,
    pub outer_frame: Frame,
    pub inner_frame: Frame,
    pub text_render_settings: TextRenderSetting,
    pub background_color: RGBAColor,
}

impl LineTextBox {
    pub fn new(outer_frame: Frame, inner_frame: Frame, render_config: Option<(TextRenderSetting, RGBAColor)>) -> LineTextBox {
        let (text_render_settings, background_color) = render_config.unwrap_or((TextRenderSetting::new(1.0, RGBColor::black()), ACTIVE_VIEW_BACKGROUND));

        LineTextBox {
            data: Vec::with_capacity(100),
            cursor: 0,
            outer_frame,
            inner_frame,
            text_render_settings,
            background_color,
        }
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.outer_frame.anchor = anchor;
        self.inner_frame = make_inner_frame(&self.outer_frame, 4);
    }
}
