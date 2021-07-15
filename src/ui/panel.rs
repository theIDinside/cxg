use super::coordinate::{Anchor, Coordinate, Layout, PointArithmetic, Size};
use super::view::{View, ViewId};
use crate::ui::Vec2i;

use std::fmt::Formatter;

#[derive(PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct PanelId(pub u32);

impl std::ops::Deref for PanelId {
    type Target = u32;

    fn deref<'a>(&'a self) -> &'a u32 {
        &self.0
    }
}

impl Into<PanelId> for u32 {
    fn into(self) -> PanelId {
        PanelId(self)
    }
}

/// A panel is a top container, that contains children of Views. Views are essentially panels where
/// text can be rendered
pub struct Panel<'app> {
    pub id: PanelId,
    pub layout: Layout,
    pub margin: Option<i32>,
    pub border: Option<i32>,
    pub size: Size,
    pub anchor: Anchor,
    pub children: Vec<View<'app>>,
}

impl<'app> std::fmt::Debug for Panel<'app> {
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

impl<'app> Panel<'app> {
    pub fn new(id: u32, layout: Layout, margin: Option<i32>, border: Option<i32>, width: i32, height: i32, anchor: Anchor) -> Panel<'app> {
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
    pub fn set_anchor(&mut self, x: i32, y: i32) {
        self.anchor = Anchor(x, y);
    }

    pub fn add_view(&mut self, mut view: View<'app>) {
        if self.children.is_empty() {
            let adjusted_anchor = self
                .margin
                .map(|margin| Anchor::vector_add(self.anchor, Vec2i::new(margin, -margin)))
                .unwrap_or(self.anchor);

            view.resize(Size::shrink_by_margin(self.size, self.margin.unwrap_or(0)));
            view.set_anchor(adjusted_anchor);
            view.set_manager_panel(self.id);
            self.children.push(view);
        } else {
            view.set_manager_panel(self.id);
            self.children.push(view);
            let sub_space_count = self.children.len();
            let margin = self.margin.unwrap_or(0);
            let child_sizes = self.size.divide(sub_space_count as _, margin, self.layout);
            println!("sizes: {:?}. Margin: {}", child_sizes, margin);
            match self.layout {
                Layout::Vertical(space) => {
                    let mut anchor_iter = Anchor::vector_add(self.anchor, Vec2i::new(margin, -margin));
                    for (c, size) in self.children.iter_mut().zip(child_sizes.into_iter()) {
                        c.resize(size);
                        c.set_anchor(anchor_iter);
                        anchor_iter = Anchor::vector_add(anchor_iter, Vec2i::new(0, -size.height - *space as i32));
                    }
                }
                Layout::Horizontal(space) => {
                    let mut anchor_iter = Anchor::vector_add(self.anchor, Vec2i::new(margin, -margin));
                    for (c, size) in self.children.iter_mut().zip(child_sizes.into_iter()) {
                        c.resize(size);
                        c.set_anchor(anchor_iter);
                        anchor_iter = Anchor::vector_add(anchor_iter, Vec2i::new(size.width + *space as i32, 0));
                    }
                }
            }
        }
        for v in self.children.iter_mut() {
            v.update();
        }
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        let old_size = self.size;
        self.size = Size::new(w, h);
        self.size_changed(old_size);
    }

    pub fn get_view(&mut self, view_id: ViewId) -> Option<*mut View<'app>> {
        for v in self.children.iter_mut() {
            if *v.id() == *view_id {
                return Some(v);
            }
        }
        None
    }

    fn size_changed(&mut self, old_size: Size) {
        let Anchor(ax, ay) = self.anchor;
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
                    let size = Size::new(self.size.width - margin * 2, view.size.height + dh);
                    view.resize(size);
                    view.set_anchor((edge_left, anchor_y_shift).into());
                    anchor_y_shift -= view.size.height + *spacing as i32;
                }
            }
            Layout::Horizontal(spacing) => {
                for (view, (dw, _)) in self
                    .children
                    .iter_mut()
                    .zip(views_width_changes.into_iter().zip(views_height_changes))
                {
                    let size = Size::new(view.size.width + dw, self.size.height - margin * 2);
                    view.resize(size);
                    view.set_anchor((anchor_x_shift, edge_top).into());
                    anchor_x_shift += view.size.width + *spacing as i32;
                }
            }
        }
        for v in self.children.iter_mut() {
            v.update();
        }
    }
}
