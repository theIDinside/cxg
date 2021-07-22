pub mod generic {

    #[derive(Copy, Clone, Debug)]
    pub struct Vec2<T> {
        pub x: T,
        pub y: T,
    }

    impl<T> Vec2<T> {
        pub fn new(x: T, y: T) -> Vec2<T> {
            Vec2 { x, y }
        }
    }

    impl Vec2<f64> {
        pub fn to_i32(&self) -> Vec2i {
            Vec2i { x: self.x as i32, y: self.y as i32 }
        }
    }

    pub type Vec2i = Vec2<i32>;
    pub type Vec2f = Vec2<f32>;
    pub type Vec2d = Vec2<f64>;
}
