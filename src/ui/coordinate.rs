use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, Mul};

use crate::datastructure::generic::{Vec2d, Vec2i};

pub enum Margin {
    /// Margin on either side of top and bottom
    Vertical(i32),
    /// Margin on either side of left and right
    Horizontal(i32),
    /// Margin on all sides, left, right, top and bottom
    Perpendicular { horizontal: i32, vertical: i32 },
}

pub trait Coordinate {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn values(&self) -> (&i32, &i32);
    fn values_mut(&mut self) -> (&mut i32, &mut i32);
    fn new(a: i32, b: i32) -> Self;
}

pub trait PointArithmetic: Copy + Clone + Coordinate {
    fn vector_add(v: Self, vec: Vec2i) -> Self {
        let r = Coordinate::new(v.x() + vec.x, v.y() + vec.y);
        r
    }

    fn vector_multiply(v: Self, vec: Vec2d) -> Self {
        Coordinate::new(((v.x() as f64 * vec.x).round()) as i32, (v.y() as f64 * vec.y).round() as i32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spacing(pub i16);

impl Deref for Spacing {
    type Target = i16;
    fn deref(&self) -> &Self::Target {
        let Spacing(x) = self;
        x
    }
}

#[derive(Clone, Copy)]
pub struct Anchor(pub i32, pub i32);

impl std::fmt::Debug for Anchor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Anchor(x, y) = self;
        f.write_fmt(format_args!("({}, {})", x, y))
    }
}

impl Coordinate for Anchor {
    #[inline(always)]
    fn x(&self) -> i32 {
        let Anchor(a, _) = self;
        *a
    }

    fn y(&self) -> i32 {
        let Anchor(.., b) = self;
        *b
    }

    fn values(&self) -> (&i32, &i32) {
        let Anchor(x, y) = self;
        (x, y)
    }

    fn values_mut(&mut self) -> (&mut i32, &mut i32) {
        let Anchor(x, y) = self;
        (x, y)
    }

    fn new(a: i32, b: i32) -> Self {
        Anchor(a, b)
    }
}

impl PointArithmetic for Anchor {}

impl Coordinate for Size {
    #[inline]
    fn x(&self) -> i32 {
        self.width
    }
    #[inline]
    fn y(&self) -> i32 {
        self.height
    }

    fn values(&self) -> (&i32, &i32) {
        let Size { width, height } = self;
        (width, height)
    }

    fn values_mut(&mut self) -> (&mut i32, &mut i32) {
        let Size { width, height } = self;
        (width, height)
    }

    fn new(a: i32, b: i32) -> Self {
        Size { width: a, height: b }
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl std::ops::Div<Size> for Size {
    type Output = Vec2d;
    fn div(self, rhs: Self) -> Self::Output {
        Size::change_factor(&self, &rhs)
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}x{} px", self.width, self.height))
    }
}

impl PointArithmetic for Size {}

impl Size {
    pub fn change_factor(lhs: &Size, rhs: &Size) -> Vec2d {
        let x = lhs.x() as f64 / rhs.x() as f64;
        let y = lhs.y() as f64 / rhs.y() as f64;
        Vec2d { x, y }
    }

    pub fn divide(&self, divisor: u32, margin: i32, layout: Layout) -> Vec<Size> {
        assert_ne!(divisor, 0);

        let divisor = divisor as i32;
        match layout {
            Layout::Horizontal(Spacing(space)) => {
                let total_width = self.width - (margin * 2) - space as i32 * (divisor - 1);
                assert!(total_width > 0, "Margin & spacing taking up more space than dimension can handle");
                let element_width = total_width / divisor;
                // we're dealing with integers... so we need all elements to actually cover, so one element might get a bit larger
                let diff_width = total_width - (divisor * element_width);
                let mut result = vec![];
                for _ in 0..(divisor - 1) {
                    result.push(Size { width: element_width, height: self.height - margin * 2 })
                }
                result.push(Size { width: element_width + diff_width, height: self.height - margin * 2 });
                result
            }
            Layout::Vertical(Spacing(space)) => {
                let width = self.width - (margin * 2);
                let total_height = self.height - (margin * 2) - space as i32 * (divisor - 1);
                assert!(total_height > 0, "Margin & spacing taking up more space than dimension can handle");
                let element_height = total_height / divisor;
                // we're dealing with integers... so we need all elements to actually cover, so one element might get a bit larger
                let diff_height = total_height - (divisor * element_height);
                let mut result = vec![];
                for _ in 0..(divisor - 1) {
                    result.push(Size { width, height: element_height })
                }
                result.push(Size { width, height: element_height + diff_height });
                result
            }
        }
    }

    pub fn shrink_by_margin(size: Size, margin: i32) -> Size {
        let width = size.width - (margin * 2);
        let height = size.height - (margin * 2);
        Size { width, height }
    }

    pub fn shrink_axis_aligned(size: Size, margin: Margin) -> Size {
        match margin {
            Margin::Vertical(margin) => Size { width: size.width, height: size.height - margin.mul(2) },
            Margin::Horizontal(margin) => Size { width: size.width - margin.mul(2), height: size.height },
            Margin::Perpendicular { horizontal, vertical } => Size { width: size.width - horizontal.mul(2), height: size.height - vertical.mul(2) },
        }
    }
}

impl Anchor {
    pub fn new(x: i32, y: i32) -> Anchor {
        Anchor(x, y)
    }
}

impl Into<Anchor> for (i32, i32) {
    fn into(self) -> Anchor {
        let (x, y) = self;
        Anchor(x, y)
    }
}

impl std::ops::Add<(i32, i32)> for &Anchor {
    type Output = Anchor;
    fn add(self, rhs: (i32, i32)) -> Self::Output {
        let Anchor(x, y) = &self;
        let (dx, dy) = rhs;
        Anchor(x + dx, y + dy)
    }
}

impl Into<Spacing> for i16 {
    fn into(self) -> Spacing {
        Spacing(self)
    }
}

/// Layout of panels inside a panel.
/// u16 value is spacing in pixels between laid out child items
#[derive(Clone, Copy)]
pub enum Layout {
    Vertical(Spacing),
    Horizontal(Spacing),
}

impl std::fmt::Debug for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (style, space) = match self {
            Layout::Vertical(Spacing(s)) => ("Vertical", s),
            Layout::Horizontal(Spacing(s)) => ("Horizontal", s),
        };
        f.write_fmt(format_args!("{} {}px", style, space))
    }
}
