use std::io::Read;

use super::{boundingbox::BoundingBox, coordinate::Anchor, view::View};

pub struct DebugView<'app> {
    pub view: View<'app>,
    pub visibile: bool,
}
#[derive(Debug)]
pub struct ProcessInfo {
    // name
    name: String,
    // process id
    pid: usize,
    // virtual memory usage, peak
    virtual_mem_usage_peak: usize,
    // virtual memory usage
    virtual_mem_usage: usize,
    // shared library code size
    shared_lib_code: usize,
}

impl ProcessInfo {
    pub fn new() -> std::io::Result<ProcessInfo> {
        let mut file = std::fs::File::open("/proc/self/status").expect("failed to open statm");
        let mut buf = String::with_capacity(1024);
        file.read_to_string(&mut buf)?; // .expect("failed to read data");
        let to_find = vec![0, 5, 16, 17, 28];
        let mut items: Vec<String> = buf
            .lines()
            .enumerate()
            .filter(|(line_no, _)| to_find.contains(line_no))
            .map(|(i, line)| line.chars().filter(|c| if i == 0 { true } else { c.is_digit(10) }).collect())
            .collect();
        let name = items.remove(0).chars().skip(6).collect();
        Ok(ProcessInfo {
            name,
            pid: items.remove(0).parse().expect("failed to parse pid"),
            virtual_mem_usage_peak: items.remove(0).parse().expect("failed to parse peak virtual memory usage"),
            virtual_mem_usage: items.remove(0).parse().expect("failed to parse virtual memory usage"),
            shared_lib_code: items.remove(0).parse().expect("failed to parse shared library code size"),
        })
    }
}

impl<'app> DebugView<'app> {
    pub fn new(view: View<'app>) -> DebugView<'app> {
        DebugView { view, visibile: false }
    }

    pub fn update(&mut self) {
        self.view.window_renderer.clear_data();
        self.view
            .window_renderer
            .add_rect(BoundingBox::from((self.view.anchor, self.view.size)), self.view.bg_color);
    }

    pub fn do_update_view(&mut self, fps: f64, frame_time: f64) {
        let Anchor(top_x, top_y) = self.view.anchor;

        let proc_info = ProcessInfo::new();
        let ProcessInfo { name, pid, virtual_mem_usage_peak, virtual_mem_usage, shared_lib_code } = proc_info.unwrap();

        let r: Vec<_> = format!(
"
Debug Information
 |  Application 
 |  > name              [{}] 
 |  > pid:              [{}]
 |  Memory: 
 |  > Usage:            [{:.2}MB]
 |  > Peak usage:       [{:.2}MB]
 |  > Shared lib code   [{:.2}MB]
 |  Timing  
 |  > Frame time:       [{:.5}ms]
 |  > Frame speed       [{:.2}f/s]",
            name,
            pid,
            virtual_mem_usage as f64 / 1024.0,
            virtual_mem_usage_peak as f64 / 1024.0,
            shared_lib_code as f64 / 1024.0,
            frame_time,
            fps
        )
        .chars()
        .collect();
        self.view.text_renderer.prepare_data_iter(r.iter(), top_x, top_y);
        self.view.set_need_redraw();
    }

    pub fn draw(&mut self) {
        self.view.window_renderer.draw();
        self.view.text_renderer.draw();
    }
}
