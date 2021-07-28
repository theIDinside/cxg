use crate::{
    datastructure::generic::Vec2i,
    debuginfo::{process_info::ProcessInfo, DebugInfo},
    opengl::{
        rectangle::{PolygonType, Texture},
        types::{RGBAColor, RGBColor},
    },
};

use crate::opengl::text as gltxt;

use super::{
    basic::{
        boundingbox::BoundingBox,
        coordinate::{Margin, Size},
    },
    view::View,
    Viewable,
};

pub struct DebugView {
    pub view: View,
    pub visibile: bool,
    debug_info: DebugInfo,
    pub bg_texture: Texture,
}

impl DebugView {
    pub fn new(view: View, debug_info: DebugInfo, bg_texture: Texture) -> DebugView {
        DebugView { view, visibile: false, debug_info, bg_texture }
    }

    pub fn resize(&mut self, size: Size) {
        self.view.resize(size);
    }

    pub fn update(&mut self, texture: Option<Texture>) {
        self.view.window_renderer.clear_data();
        self.view.text_renderer.clear_data();
        let RGBAColor { r, g, b, .. } = self.view.bg_color;
        let bg_color = RGBColor::new(r, g, b);

        // draw title bar
        self.view.window_renderer.make_bordered_rect(
            BoundingBox::expand(&self.view.title_frame.to_bb(), Margin::Vertical(2)).translate_mut(Vec2i::new(0, -4)),
            bg_color.uniform_scale(-0.1),
            (1, bg_color.uniform_scale(-1.0)),
            PolygonType::RoundedUndecorated { corner_radius: 15.0 },
        );

        if let Some(texture) = texture {
            // draw content pane
            self.view.window_renderer.make_bordered_rect(
                self.view.view_frame.to_bb(),
                bg_color,
                (2, bg_color.uniform_scale(-1.0)),
                PolygonType::RoundedUndecorated { corner_radius: 15.0 },
            );
        } else {
            self.view.window_renderer.make_bordered_rect(
                self.view.view_frame.to_bb(),
                bg_color,
                (2, bg_color.uniform_scale(-1.0)),
                PolygonType::RoundedUndecorated { corner_radius: 15.0 },
            );
        }
        let Size { width, height } = self.view.view_frame.size;
        let mut image_bb = BoundingBox::shrink(&self.view.view_frame.to_bb(), Margin::Perpendicular { h: 10, v: 10 });
        self.view
            .window_renderer
            .push_draw_command(image_bb, bg_color, PolygonType::Decorated { texture: self.bg_texture });
    }

    pub fn do_update_view(&mut self, fps: f64, frame_time: f64) {
        if self.visibile {
            let Vec2i { x: top_x, y: top_y } = self.view.view_frame.anchor;
            let proc_info = ProcessInfo::new();
            let ProcessInfo { name, pid, virtual_mem_usage_peak, virtual_mem_usage, rss, shared_lib_code } = proc_info.unwrap();
            let title = "Debug Information";
            let r = format!(
                "
 |  Application 
 |  > name                          [{}] 
 |  > pid:                          [{}]
 |  Memory: 
 |  > Usage:                        [{:.2}MB]
 |  > Peak usage:                   [{:.2}MB]
 |  > Shared lib code               [{:.2}MB]
 |  > RSS                           [{:.2}MB]
 |  > Allocated heap since start    [{:.2}MB]
 |  Timing  
 |  > Frame time:                   [{:.5}ms]
 |  > Frame speed                   [{:.2}f/s]",
                name,
                pid,
                virtual_mem_usage as f64 / 1024.0,
                virtual_mem_usage_peak as f64 / 1024.0,
                shared_lib_code as f64 / 1024.0,
                rss as f64 / 1024.0,
                self.debug_info.heap_increase_since_start() as f64 / (1024.0 * 1024.0), // we read *actual* heap addresses, and these obviously are measured in bytes. The others are values from syscall proc, and they return in KB
                frame_time,
                fps
            );

            let it: Vec<char> = r.chars().collect();
            let mut size = gltxt::calculate_text_dimensions(&it, &self.view.edit_font);
            size.height += self.view.title_frame.size.height + 40;
            size.width += 20;
            self.resize(size);
            self.update(Some(self.bg_texture.clone()));

            let Vec2i { x: tx, y: ty } = self.view.title_frame.anchor;
            self.view
                .text_renderer
                .push_draw_command(title.chars().map(|c| c), RGBColor::black(), tx + 3, ty, self.view.title_font.clone());
            let color = RGBColor::white();
            self.view
                .text_renderer
                .push_draw_command(it.iter().map(|c| *c), color, top_x, top_y, self.view.edit_font.clone());
            self.view.set_need_redraw();
        }
    }

    pub fn draw(&mut self) {
        if !self.visibile {
            return;
        }
        self.view.window_renderer.draw_list();
        self.view.text_renderer.draw_list();
    }
}
