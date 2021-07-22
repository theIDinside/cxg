use std::io::Read;

use crate::debuginfo::{process_info::ProcessInfo, DebugInfo};

use super::{boundingbox::BoundingBox, coordinate::Anchor, view::View};

pub struct DebugView<'app> {
    pub view: View<'app>,
    pub visibile: bool,
    debug_info: DebugInfo,
}

impl<'app> DebugView<'app> {
    pub fn new(view: View<'app>, debug_info: DebugInfo) -> DebugView<'app> {
        DebugView { view, visibile: false, debug_info }
    }

    pub fn update(&mut self) {
        self.view.window_renderer.clear_data();
        self.view
            .window_renderer
            .add_rect(BoundingBox::from((self.view.anchor, self.view.size)), self.view.bg_color);
    }

    pub fn do_update_view(&mut self, fps: f64, frame_time: f64) {
        if self.visibile {
            let Anchor(top_x, top_y) = self.view.anchor;
            let proc_info = ProcessInfo::new();
            let ProcessInfo { name, pid, virtual_mem_usage_peak, virtual_mem_usage, rss, shared_lib_code } = proc_info.unwrap();

            let r: Vec<_> = format!(
                "
Debug Information
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
            )
            .chars()
            .collect();
            self.view.text_renderer.prepare_data_iter(r.iter(), top_x, top_y);
            self.view.set_need_redraw();
        }
    }

    pub fn draw(&mut self) {
        self.view.window_renderer.draw();
        self.view.text_renderer.draw();
    }
}
