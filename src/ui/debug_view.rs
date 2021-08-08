use crate::{
    datastructure::generic::Vec2i,
    debuginfo::{process_info::ProcessInfo, DebugInfo},
    opengl::{
        polygon_renderer::{PolygonType, Texture},
        types::RGBColor,
    },
};

use crate::opengl::text_renderer as gltxt;

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
    pub handle_key_time: u128,
}

impl DebugView {
    pub fn new(view: View, debug_info: DebugInfo, bg_texture: Texture) -> DebugView {
        DebugView { view, visibile: false, debug_info, bg_texture, handle_key_time: 0 }
    }

    pub fn resize(&mut self, size: Size) {
        self.view.resize(size);
    }

    pub fn update(&mut self) {
        self.view.window_renderer.clear_data();
        self.view.text_renderer.clear_data();
        let bg_color = self.view.bg_color;
        // draw title bar
        self.view.window_renderer.make_bordered_rect(
            BoundingBox::expand(&self.view.title_frame.to_bb(), Margin::Vertical(2)).translate_mut(Vec2i::new(0, -4)),
            bg_color.uniform_scale(-0.1),
            (1, bg_color.uniform_scale(-1.0)),
            PolygonType::RoundedUndecorated { corner_radius: 5.0 },
        );
        let mut view_bb = self.view.view_frame.to_bb();
        view_bb.max.x = self.view.title_frame.anchor.x + self.view.title_frame.width();
        self.view
            .window_renderer
            .make_bordered_rect(view_bb, bg_color, (2, bg_color.uniform_scale(-1.0)), PolygonType::Undecorated);
        let image_bb = BoundingBox::shrink(&self.view.view_frame.to_bb(), Margin::Perpendicular { h: 20, v: 20 });
        let mut see_through_bg = bg_color;
        see_through_bg.a = 0.1;
        self.view
            .window_renderer
            .push_draw_command(image_bb, see_through_bg, PolygonType::Decorated { texture: self.bg_texture });
    }

    pub fn do_update_view(&mut self, fps: f64, frame_time: f64) {
        if self.visibile {
            let Vec2i { x: top_x, y: top_y } = self.view.view_frame.anchor;
            let proc_info = ProcessInfo::new();
            let ProcessInfo { name, pid, virtual_mem_usage_peak, virtual_mem_usage, rss, shared_lib_code } = proc_info.unwrap();
            let title = "Debug Information";
            let all_debug_info_string = format!(
                "
   Application 
   > name                                       [{}] 
   > pid:                                       [{}]
   Memory: 
   > Allocated Virtual Memory:                  [{:.2}MB]
   > Peak allocated VM:                         [{:.2}MB]
   > Shared lib code                            [{:.2}MB]
   > RSS (actual physical mem allocated)        [{:.2}MB]
   > Allocated heap since start                 [{:.2}MB]
   Timing           
   > Frame time:                                [{:.5}ms]
   > Frame speed                                [{:.2}f/s]
   > Key translation time                       [{:.5}ms]",
                name,
                pid,
                virtual_mem_usage as f64 / 1024.0,
                virtual_mem_usage_peak as f64 / 1024.0,
                shared_lib_code as f64 / 1024.0,
                rss as f64 / 1024.0,
                self.debug_info.heap_allocated_since_begin() as f64 / (1024.0 * 1024.0), // we read *actual* heap addresses, and these obviously are measured in bytes. The others are values from syscall proc, and they return in KB
                frame_time,
                fps,
                self.handle_key_time as f64 / 1000.0
            );

            let mut size = gltxt::calculate_text_dimensions_iter(&all_debug_info_string, &self.view.edit_font);
            size.height += self.view.title_frame.size.height + 40;
            size.width += 20;
            self.resize(size);
            self.update();

            let Vec2i { x: tx, y: ty } = self.view.title_frame.anchor;
            let text_title_rect = gltxt::calculate_text_dimensions_iter(title, &self.view.title_font);
            let half = text_title_rect.width / 2;
            let title_frame_half = self.view.title_frame.width() / 2;
            let start_x = title_frame_half - half;

            self.view
                .text_renderer
                .push_draw_command(title.chars(), RGBColor::black(), tx + start_x, ty, self.view.title_font.clone());
            let color = RGBColor::white();
            self.view
                .text_renderer
                .push_draw_command(all_debug_info_string.chars(), color, top_x, top_y, self.view.edit_font.clone());
            self.view.set_need_redraw();
        }
    }

    pub fn draw(&mut self) {
        if !self.visibile {
            return;
        }
        self.view.window_renderer.execute_draw_list();
        self.view.text_renderer.execute_draw_list();
    }
}
