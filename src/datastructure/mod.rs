pub mod generic {
    use std::ops::{Add, AddAssign, Mul, MulAssign};

    use crate::ui::basic::boundingbox::BoundingBox;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Vec2<T> {
        pub x: T,
        pub y: T,
    }

    impl<T> Vec2<T> {
        #[inline(always)]
        pub fn new(x: T, y: T) -> Vec2<T> {
            Vec2 { x, y }
        }
    }

    impl Vec2<i32> {
        pub fn to_f64(&self) -> Vec2<f64> {
            Vec2d { x: self.x as f64, y: self.y as f64 }
        }

        pub fn clamped(&self, bb: &BoundingBox) -> Vec2i {
            let &Vec2i { x, y } = self;
            let BoundingBox { min, max } = bb;
            Vec2i::new(x.clamp(min.x, max.x), y.clamp(min.y, max.y))
        }
    }

    impl Vec2<f64> {
        pub fn to_i32(&self) -> Vec2i {
            Vec2i { x: self.x as i32, y: self.y as i32 }
        }
    }

    impl Vec2i {
        pub fn to_f32(&self) -> Vec2<gl::types::GLfloat> {
            Vec2::new(self.x as gl::types::GLfloat, self.y as gl::types::GLfloat)
        }
    }

    pub type Vec2i = Vec2<i32>;
    pub type Vec2f = Vec2<f32>;
    pub type Vec2d = Vec2<f64>;

    impl<T> std::ops::Add for Vec2<T>
    where
        T: Add + AddAssign,
    {
        type Output = Vec2<<T as Add>::Output>;

        fn add(self, rhs: Self) -> Self::Output {
            let Vec2 { x, y } = self;
            Vec2::new(x + rhs.x, y + rhs.y)
        }
    }

    impl<T> std::ops::AddAssign for Vec2<T>
    where
        T: Add + AddAssign,
    {
        fn add_assign(&mut self, rhs: Self) {
            self.x += rhs.x;
            self.y += rhs.y;
        }
    }

    impl<T> std::ops::Mul for Vec2<T>
    where
        T: Mul + MulAssign,
    {
        type Output = Vec2<<T as Mul>::Output>;

        fn mul(self, rhs: Self) -> Self::Output {
            Vec2::new(self.x * rhs.x, self.y * rhs.y)
        }
    }
}
