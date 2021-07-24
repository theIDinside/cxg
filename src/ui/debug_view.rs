use crate::{
    datastructure::generic::Vec2i,
    debuginfo::{process_info::ProcessInfo, DebugInfo},
};

use super::{
    basic::coordinate::{Margin, Size},
    boundingbox::BoundingBox,
    view::View,
    Viewable,
};

pub struct DebugView<'app> {
    pub view: View<'app>,
    pub visibile: bool,
    debug_info: DebugInfo,
}

impl<'app> DebugView<'app> {
    pub fn new(view: View<'app>, debug_info: DebugInfo) -> DebugView<'app> {
        DebugView { view, visibile: false, debug_info }
    }

    pub fn resize(&mut self, size: Size) {
        self.view.resize(size);
    }

    pub fn update(&mut self) {
        self.view.window_renderer.clear_data();
        self.view.menu_text_renderer.clear_data();
        self.view.text_renderer.clear_data();
        // draw filled rectangle, which will become border
        self.view
            .window_renderer
            .add_rect(self.view.title_frame.to_bb(), self.view.bg_color.uniform_scale(-1.0));
        // fill out the inner, leaving the previous draw as border
        self.view
            .window_renderer
            .add_rect(BoundingBox::shrink(&self.view.title_frame.to_bb(), Margin::Perpendicular { h: 2, v: 2 }), self.view.bg_color.uniform_scale(1.0));
        // draw view rectangle, the background for the text editor,

        self.view
            .window_renderer
            .add_rect(self.view.view_frame.to_bb(), self.view.bg_color.uniform_scale(-1.0));

        self.view
            .window_renderer
            .add_rect(BoundingBox::shrink(&self.view.view_frame.to_bb(), Margin::Perpendicular { h: 2, v: 2 }), self.view.bg_color);
    }

    pub fn do_update_view(&mut self, fps: f64, frame_time: f64) {
        if self.visibile {
            let Vec2i { x: top_x, y: top_y } = self.view.view_frame.anchor;
            let proc_info = ProcessInfo::new();
            let ProcessInfo { name, pid, virtual_mem_usage_peak, virtual_mem_usage, rss, shared_lib_code } = proc_info.unwrap();
            let title = "Debug Information".chars().collect();
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
            let mut size = self.view.text_renderer.calculate_text_dimensions(&it);
            size.height += self.view.title_frame.size.height + 40;
            size.width += 20;
            self.resize(size);
            self.update();
            self.view.draw_title(&title);
            self.view.text_renderer.append_data(it.iter(), top_x, top_y);
            // self.view.text_renderer.prepare_data_from_iter(r.iter(), top_x, top_y);
            self.view.set_need_redraw();
        }
    }

    pub fn draw(&mut self) {
        if !self.visibile {
            return;
        }
        self.view.window_renderer.draw();
        self.view.text_renderer.draw();
        self.view.menu_text_renderer.draw();
    }
}
