use super::boundingbox::BoundingBox;
use super::coordinate::{Coordinate, Layout, Size};
use super::view::{View, ViewId};
use super::Viewable;
use crate::ui::Vec2i;

use std::fmt::Formatter;

#[derive(PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct PanelId(pub u32);

impl std::ops::Deref for PanelId {
    type Target = u32;
    #[inline(always)]
    fn deref<'a>(&'a self) -> &'a u32 {
        &self.0
    }
}

impl Into<PanelId> for u32 {
    #[inline(always)]
    fn into(self) -> PanelId {
        PanelId(self)
    }
}

/// A panel is a top container, that contains children of Views. Views are essentially panels where
/// text can be rendered
pub struct Panel {
    pub id: PanelId,
    pub layout: Layout,
    pub margin: Option<i32>,
    pub border: Option<i32>,
    pub size: Size,
    pub anchor: Vec2i,
    pub children: Vec<View>,
}

impl std::fmt::Debug for Panel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("anchor", &self.anchor)
            .field("layout", &self.layout)
            .field("margin", &self.margin)
            .finish()?;
        write!(f, "\n\tViews:\n")?;
        for c in self.children.iter() {
            write!(f, "\t\t{:?}\n", c)?;
        }
        write!(f, "")
    }
}

/// Takes a number and divides it with spread_count and creates a vector of elements
/// with the value of result. If the divided value multiplied by spread_count doesn't equal
/// number, the first element of the resulting vector, will get the added difference,
/// assuring that the accumulated sum(result) = number
pub fn divide_scatter(number: i32, spread_count: usize) -> Vec<i32> {
    let mut r = Vec::with_capacity(spread_count);
    let element_val = number / spread_count as i32;
    r.resize(spread_count, element_val);
    let check_total = element_val * spread_count as i32;
    if check_total - number != 0 {
        if number < 0 {
            r[0] += number - check_total;
        } else {
            r[0] += number - check_total;
        }
    }
    r
}

impl Panel {
    pub fn new(id: u32, layout: Layout, margin: Option<i32>, border: Option<i32>, width: i32, height: i32, anchor: Vec2i) -> Panel {
        Panel {
            id: id.into(),
            layout: layout,
            margin: margin,
            border: border,
            size: Size::new(width, height),
            anchor: anchor,
            children: vec![],
        }
    }

    pub fn width(&self) -> i32 {
        self.size.x()
    }
    pub fn height(&self) -> i32 {
        self.size.y()
    }

    pub fn layout(&mut self) {
        if self.children.len() == 1 {
            let adjusted_anchor = self
                .margin
                .map(|margin| self.anchor + Vec2i::new(margin, -margin))
                .unwrap_or(self.anchor);
            let view = self.children.first_mut().unwrap();
            view.resize(Size::shrink_by_margin(self.size, self.margin.unwrap_or(0)));
            view.set_anchor(adjusted_anchor);
        } else {
            let sub_space_count = self.children.iter().filter(|v| v.visible).count();
            let margin = self.margin.unwrap_or(0);
            let child_sizes = self.size.divide(sub_space_count as _, margin, self.layout);
            match self.layout {
                Layout::Vertical(space) => {
                    let mut anchor_iter = self.anchor + Vec2i::new(margin, -margin);
                    for (c, size) in self.children.iter_mut().filter(|v| v.visible).zip(child_sizes.into_iter()) {
                        c.resize(size);
                        c.set_anchor(anchor_iter);
                        anchor_iter += Vec2i::new(0, -size.height - *space as i32);
                    }
                }
                Layout::Horizontal(space) => {
                    let mut anchor = self.anchor + Vec2i::new(margin, -margin);
                    for (c, size) in self.children.iter_mut().filter(|v| v.visible).zip(child_sizes.iter()) {
                        c.set_anchor(anchor);
                        c.resize(*size);
                        anchor += Vec2i::new(size.width + *space as i32, 0);
                    }
                }
            }
        }
        for v in self.children.iter_mut().filter(|v| v.visible) {
            v.update(None);
        }
    }

    pub fn add_view(&mut self, mut view: View) {
        view.set_manager_panel(self.id);
        self.children.push(view);
        self.layout();
    }

    pub fn remove_view(&mut self, view_id: ViewId) -> Option<View> {
        if let Some(pos) = self.children.iter().position(|v| v.id == view_id) {
            let v = self.children.remove(pos);
            Some(v)
        } else {
            None
        }
    }

    pub fn get_view(&mut self, view_id: ViewId) -> Option<*mut View> {
        for v in self.children.iter_mut() {
            if *v.id() == *view_id {
                return Some(v);
            }
        }
        None
    }

    fn size_changed(&mut self, old_size: Size) {
        let Vec2i { x: ax, y: ay } = self.anchor;
        let diff_width = self.size.width - old_size.width;
        let diff_height = self.size.height - old_size.height;
        let views_height_changes = divide_scatter(diff_height, self.children.len());
        let views_width_changes = divide_scatter(diff_width, self.children.len());
        let margin = self.margin.unwrap_or(0);

        let mut anchor_y_shift = ay - margin;
        let mut anchor_x_shift = ax + margin;

        let (edge_left, edge_top) = (ax + margin, ay - margin);

        match self.layout {
            Layout::Vertical(spacing) => {
                for (view, (_, dh)) in self
                    .children
                    .iter_mut()
                    .zip(views_width_changes.into_iter().zip(views_height_changes))
                {
                    let view_size = view.total_size();
                    let size = Size::new(self.size.width - margin * 2, view_size.height + dh);
                    view.resize(size);
                    view.set_anchor(Vec2i::new(edge_left, anchor_y_shift));
                    anchor_y_shift -= view_size.height + *spacing as i32;
                }
            }
            Layout::Horizontal(spacing) => {
                for (view, (dw, _)) in self
                    .children
                    .iter_mut()
                    .zip(views_width_changes.into_iter().zip(views_height_changes))
                {
                    let view_size = view.total_size();
                    let size = Size::new(view_size.width + dw, self.size.height - margin * 2);
                    // let size = Size::new(view.size.width + dw, self.size.height);
                    view.resize(size);
                    // view.resize(Size::shrink_by_margin(size, margin));
                    view.set_anchor(Vec2i::new(anchor_x_shift, edge_top));
                    anchor_x_shift += view_size.width + *spacing as i32;
                }
            }
        }
        for v in self.children.iter_mut() {
            v.update(None);
        }
    }
}

impl Viewable for Panel {
    fn resize(&mut self, size: Size) {
        let old_size = self.size;
        self.size = size;
        self.size_changed(old_size);
    }

    fn set_anchor(&mut self, anchor: Vec2i) {
        self.anchor = anchor;
    }

    fn bounding_box(&self) -> super::boundingbox::BoundingBox {
        BoundingBox::from_info(self.anchor, self.size)
    }

    fn mouse_clicked(&mut self, _pos: Vec2i) {
        todo!()
    }

    fn mouse_dragged(&mut self, _begin_coordinate: Vec2i, _current_coordinated: Vec2i) -> Option<Vec2i> {
        todo!()
    }
}
