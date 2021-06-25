use super::coordinate::{Anchor, Layout, Size, PointArithmetic, Coordinate};
use super::view::View;
use crate::ui::{Vec2i};

use std::fmt::{Formatter};

/// A panel is a top container, that contains children of Views. Views are essentially panels where
/// text can be rendered
pub struct Panel<'a> {
    pub id: u32,
    pub layout: Layout,
    pub margin: Option<i32>,
    pub border: Option<i32>,
    pub size: Size,
    pub anchor: Anchor,
    pub children: Vec<View<'a>>,
    active_view: usize
}

impl<'a> std::fmt::Debug for Panel<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel").field("id", &self.id)
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

/// Resize direction, when a panel, view, or window gets resized, describes in what direction the increased width or height goes
#[derive(Debug, Clone, Copy)]
pub enum Resize {
    Left(i32),
    Right(i32),
    Top(i32),
    Bottom(i32),
}

#[derive(Debug, Clone, Copy)]
pub enum SizeChange {
    Shrink(u32, u32),
    Expand(u32, u32),
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

pub enum PanelCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl<'a> Panel<'a> {
    pub fn new(id: u32, layout: Layout, margin: Option<i32>, border: Option<i32>, width: i32, height: i32, anchor: Anchor
    ) -> Panel<'a> {
        Panel {
            id: id,
            layout: layout,
            margin: margin,
            border: border,
            size: Size::new(width, height),
            anchor: anchor,
            children: vec![],
            active_view: 0
        }
    }

    pub fn width(&self) -> i32 { self.size.x() }
    pub fn height(&self) -> i32 { self.size.y() }

    pub fn set_anchor(&mut self, x: i32, y: i32) {
        self.anchor = Anchor(x, y);
    }

    pub fn corner(&self, corner: PanelCorner) -> Anchor {
        let Anchor(x, y) = self.anchor;
        match corner {
            PanelCorner::TopLeft => {
                Anchor(x, y)
            }
            PanelCorner::TopRight => {
                Anchor(x + self.size.width, y)
            }
            PanelCorner::BottomLeft => {
                Anchor(x, y - self.size.height)
            }
            PanelCorner::BottomRight => {
                Anchor(x + self.size.width, y - self.size.height)
            }
        }
    }

    pub fn edge(&self, edge: Edge) -> i32 {
        let Anchor(x, y) = self.anchor;
        match edge {
            Edge::Left => x,
            Edge::Right => x + self.size.width,
            Edge::Top => y,
            Edge::Bottom => y - self.size.height
        }
    }

    pub fn add_view(&mut self, mut view: View<'a>) {
        if self.children.is_empty() {
            let adjusted_anchor = self.margin.and_then(|margin| Some(Anchor::vector_add(self.anchor, Vec2i::new(margin, -margin)))).unwrap_or(self.anchor);
            view.resize(self.width() - self.margin.unwrap_or(0) * 2, self.height() - self.margin.unwrap_or(0) * 2);
            view.set_anchor(adjusted_anchor);
            self.children.push(view);
        } else {
            self.children.push(view);
            let sub_space_count = self.children.len();
            let margin = self.margin.unwrap_or(0);
            let child_sizes = self.size.divide(sub_space_count as _, margin, self.layout);
            match self.layout {
                Layout::Vertical(space) => {
                    let mut anchor_iter = Anchor::vector_add(self.anchor, Vec2i::new(margin, -margin));
                    for (c, Size { width, height }) in self.children.iter_mut().zip(child_sizes.into_iter()) {
                        c.resize(width as _, height as _);
                        c.set_anchor(anchor_iter);
                        anchor_iter = Anchor::vector_add(anchor_iter, Vec2i::new(0, -height - *space as i32));
                    }
                }
                Layout::Horizontal(space) => {
                    let mut anchor_iter = Anchor::vector_add(self.anchor, Vec2i::new(margin, -margin));
                    for (c, Size { width, height }) in self.children.iter_mut().zip(child_sizes.into_iter()) {
                        c.resize(width as _, height as _);
                        c.set_anchor(anchor_iter);
                        anchor_iter = Anchor::vector_add(anchor_iter, Vec2i::new(width + *space as i32, 0));
                    }
                }
            }
        }
        for v in self.children.iter_mut() {
            v.update();
        }
    }

    pub fn resize_panel(&mut self, resize: Resize) {
        let (ax, ay) = self.anchor.values_mut();
        let (width, height) = self.size.values_mut();
        match resize {
            Resize::Left(new_width) => {
                let diff_width = new_width - *width;
                *ax -= diff_width;
                *width = new_width;
                if self.children.is_empty() { return; }
                let views_width_changes = divide_scatter(diff_width, self.children.len());
                let mut tot = 0;
                for (added_width, view) in views_width_changes.iter().zip(self.children.iter_mut()) {
                    let new_anchor = Anchor::vector_add(view.anchor, Vec2i::new(-diff_width + tot, 0));
                    view.set_anchor(new_anchor);
                    view.size = Size::vector_add(view.size, Vec2i { x: *added_width, y: 0 });
                    if let Layout::Horizontal(_) = self.layout {
                        tot += added_width;
                    }
                }
            }
            Resize::Right(new_width) => {
                let diff_width = new_width - *width;
                *width = new_width;
                if self.children.is_empty() { return; }
                let views_width_changes = divide_scatter(diff_width, self.children.len());
                let mut anchor_x_shift = 0;
                for (width_diff, view) in views_width_changes.into_iter().zip(self.children.iter_mut()) {
                    let new_anchor = Anchor::vector_add(view.anchor, Vec2i::new(anchor_x_shift, 0));
                    view.set_anchor(new_anchor);
                    view.size = Size::vector_add(view.size, Vec2i::new(width_diff, 0));
                    if let Layout::Horizontal(_) = self.layout {
                        anchor_x_shift += width_diff;
                    }
                }
            }
            Resize::Top(new_height) => {
                let diff_height = new_height - *height;
                *ay += diff_height;
                *height = new_height;
                if self.children.is_empty() { return; }
                let views_height_changes = divide_scatter(diff_height, self.children.len());
                let mut tot = diff_height;
                for (added_height, view) in views_height_changes.into_iter().zip(self.children.iter_mut()) {
                    let new_anchor = Anchor::vector_add(view.anchor, Vec2i::new(0, diff_height + tot));
                    view.set_anchor(new_anchor);
                    view.size = Size::vector_add(view.size, Vec2i { x: 0, y: added_height });
                    if let Layout::Vertical(_) = self.layout {
                        tot += added_height;
                    }
                }
            }
            Resize::Bottom(new_height) => {
                let diff_height = new_height - *height;
                *ay += diff_height;
                *height = new_height;
                if self.children.is_empty() { return; }
                let views_height_changes = divide_scatter(diff_height, self.children.len());
                let mut anchor_y_shift = diff_height;
                for (added_height, view) in views_height_changes.into_iter().zip(self.children.iter_mut()) {
                    let new_anchor = Anchor::vector_add(view.anchor, Vec2i::new(0, anchor_y_shift));
                    view.set_anchor(new_anchor);
                    view.size = Size::vector_add(view.size, Vec2i { x: 0, y: added_height });
                    if let Layout::Vertical(_) = self.layout {
                        anchor_y_shift -= added_height;
                    }
                }
            }
        }

        // the reason why we iterate twice over children, once in an arm and once here
        // is so that we don't clutter our code the fuck up, with .update() calls to views
        // yet we keep these calls centralized and thus easy to debug / reason about
        for v in self.children.iter_mut() {
            v.update();
        }
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.size = Size::new(w, h);
    }

    pub fn size_changed(&mut self, old_size: Size) {
        let Anchor(ax, ay) = self.anchor;
        let diff_width = self.size.width - old_size.width;
        let diff_height = self.size.height - old_size.height;
        let views_height_changes = divide_scatter(diff_height, self.children.len());
        let views_width_changes = divide_scatter(diff_width, self.children.len());
        let margin = self.margin.unwrap_or(0);

        let mut anchor_y_shift = ay - margin;
        let mut anchor_x_shift = ax + margin;

        let edge_left = self.edge(Edge::Left) + margin;
        let edge_top = self.edge(Edge::Top) - margin;

        match self.layout {
            Layout::Vertical(spacing) => {
                for (view, (_, dh)) in self.children.iter_mut().zip(views_width_changes.into_iter().zip(views_height_changes)) {
                    view.resize(self.size.width - margin * 2, view.size.height + dh);
                    view.set_anchor((edge_left, anchor_y_shift).into());
                    anchor_y_shift -= view.size.height + *spacing as i32;
                }
            }
            Layout::Horizontal(spacing) => {
                for (view, (dw, _)) in self.children.iter_mut().zip(views_width_changes.into_iter().zip(views_height_changes)) {
                    view.resize(view.size.width + dw, self.size.height - margin * 2);
                    view.set_anchor((anchor_x_shift, edge_top).into());
                    anchor_x_shift -= view.size.width + *spacing as i32;
                }
            }
        }
        for v in self.children.iter_mut() {
            v.update();
        }
    }
}