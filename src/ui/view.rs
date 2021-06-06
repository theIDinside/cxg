/// Layout of panels inside a panel. 
/// u16 value is spacing in pixels between laid out child items
pub enum Layout {
    Vertical(u16), 
    Horizontal(u16)
}

#[allow(unused)]
pub struct Panel {
    margin: u16,
    width: u32,
    height: u32,
    layout: Layout,
    border: Option<u8>,
    id: usize,
    pid: Option<usize>,
    children: Vec<usize>
}

impl Panel {
    pub fn new(id: usize, width: u32, height: u32, margin: u16, layout: Layout) -> Panel {
        Panel { 
            margin, 
            width, 
            height, 
            layout, 
            border: None, 
            id, 
            pid: None, 
            children: vec![]
        } 
    }

    pub fn set_parent(&mut self, pid: usize) {
        self.pid = Some(pid);
    }

    pub fn add_child(&mut self, p: &mut Panel) {
        p.set_parent(self.id);
        self.children.push(p.id);
        let child_count = self.children.len();
        match self.layout {
            Layout::Horizontal(spacing) => {
                let total_spacing = spacing as usize * (child_count - 1);
                let uninhabitable_region_x = total_spacing + self.margin as usize * 2;
            },
            Layout::Vertical(spacing) => {
                let total_spacing = spacing as usize * (child_count - 1);
                let uninhabitable_region_y = total_spacing + self.margin as usize * 2;
            },
        }
    }
}