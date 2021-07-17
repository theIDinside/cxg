use gl::types::GLfloat as glfloat;

use crate::datastructure::generic::Vec2f;

pub struct UVCoordinates {
    pub u: glfloat,
    pub v: glfloat,
}

pub struct RGBColor {
    pub r: glfloat,
    pub g: glfloat,
    pub b: glfloat,
}

#[derive(Debug, Copy, Clone)]
pub struct RGBAColor {
    pub r: glfloat,
    pub g: glfloat,
    pub b: glfloat,
    pub a: glfloat,
}

impl RGBAColor {
    pub fn new(r: glfloat, g: glfloat, b: glfloat, a: glfloat) -> RGBAColor {
        RGBAColor { r, g, b, a }
    }
}

pub struct RectVertex {
    pub coord: Vec2f,
    pub color: RGBAColor,
}

impl RectVertex {
    pub fn new(x: i32, y: i32, color: RGBAColor) -> RectVertex {
        let coord = Vec2f { x: x as glfloat, y: y as glfloat };
        RectVertex { coord, color }
    }
}

#[derive(Clone, Copy)]
pub struct TextVertex {
    pub x: glfloat,
    pub y: glfloat,
    pub u: glfloat,
    pub v: glfloat,
    pub r: glfloat,
    pub g: glfloat,
    pub b: glfloat,
    _padding: glfloat,
}

impl TextVertex {
    #[inline(always)]
    pub fn new(x: glfloat, y: glfloat, u: glfloat, v: glfloat, r: glfloat, g: glfloat, b: glfloat) -> TextVertex {
        TextVertex { x, y, u, v, r, g, b, _padding: 0.0 }
    }

    pub fn create(coords: Vec2f, uv: UVCoordinates, color: RGBColor) -> TextVertex {
        let Vec2f { x, y } = coords;
        let UVCoordinates { u, v } = uv;
        let RGBColor { r, g, b } = color;
        TextVertex { x, y, u, v, r, g, b, _padding: 0.0 }
    }
}

pub struct Vec4f {
    pub a: glfloat,
    pub b: glfloat,
    pub c: glfloat,
    pub d: glfloat,
}

impl Vec4f {
    pub fn new(a: glfloat, b: glfloat, c: glfloat, d: glfloat) -> Vec4f {
        Vec4f { a, b, c, d }
    }
}

pub struct Matrix {
    pub data: [Vec4f; 4],
}

impl Matrix {
    pub unsafe fn as_ptr(&self) -> *const f32 {
        &self.data[0].a as *const _
    }
}
