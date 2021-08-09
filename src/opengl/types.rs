use gl::types::GLfloat as glfloat;

use crate::datastructure::generic::Vec2f;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

pub struct UVCoordinates {
    pub u: glfloat,
    pub v: glfloat,
}

#[derive(Clone, Copy)]
pub struct RGBColor {
    pub r: glfloat,
    pub g: glfloat,
    pub b: glfloat,
}

impl std::ops::Add for RGBColor {
    type Output = RGBColor;

    fn add(self, rhs: Self) -> Self::Output {
        RGBColor { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b }
    }
}

impl RGBColor {
    pub fn new(r: f32, g: f32, b: f32) -> RGBColor {
        RGBColor { r, g, b }
    }

    pub fn black() -> RGBColor {
        RGBColor { r: 0.0, g: 0.0, b: 0.0 }
    }

    pub fn white() -> RGBColor {
        RGBColor { r: 1.0, g: 1.0, b: 1.0 }
    }

    pub fn red() -> RGBColor {
        RGBColor { r: 1.0, g: 0.0, b: 0.0 }
    }

    pub fn green() -> RGBColor {
        RGBColor { r: 0.0, g: 1.0, b: 0.0 }
    }

    pub fn blue() -> RGBColor {
        RGBColor { r: 0.0, g: 0.0, b: 1.0 }
    }

    pub fn gray() -> RGBColor {
        RGBColor { r: 0.5, g: 0.5, b: 0.5 }
    }

    pub fn uniform_scale(&self, value: f32) -> RGBColor {
        let &RGBColor { r, g, b } = self;
        Self::new(r + value, g + value, b + value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RGBAColor {
    pub r: glfloat,
    pub g: glfloat,
    pub b: glfloat,
    pub a: glfloat,
}

impl RGBAColor {
    pub fn to_rgb(self) -> RGBColor {
        let RGBAColor { r, g, b, .. } = self;
        RGBColor { r, g, b }
    }

    pub fn new(r: glfloat, g: glfloat, b: glfloat, a: glfloat) -> RGBAColor {
        RGBAColor { r, g, b, a }
    }

    pub fn black() -> RGBAColor {
        RGBAColor { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn white() -> RGBAColor {
        RGBAColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }

    pub fn red() -> RGBAColor {
        RGBAColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    pub fn green() -> RGBAColor {
        RGBAColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
    }

    pub fn blue() -> RGBAColor {
        RGBAColor { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
    }

    pub fn gray() -> RGBAColor {
        RGBAColor { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }
    }

    pub fn uniform_scale(&self, value: f32) -> RGBAColor {
        let &RGBAColor { r, g, b, a } = self;
        Self::new(r + value, g + value, b + value, a)
    }
}

struct RGBAColorVisitor;

impl Serialize for RGBAColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let RGBAColor { r, g, b, a } = self;
        let res = format!("RGBA({}, {}, {}, {})", r, g, b, a);
        serializer.serialize_str(&res)
    }
}

impl<'de> Visitor<'de> for RGBAColorVisitor {
    type Value = RGBAColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "Expecting key combinations to be written in the form [modA +.. modN]+Key, for example: 
        'ctrl+shift+O' or 'ctrl+O' or just 'O' for no modifiers",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        const TAG: &'static str = "RGBA(";
        let v = if &value[0..TAG.len()] == TAG { &value[TAG.len()..] } else { value };

        assert_eq!(&v[v.len() - 1..], ")");
        let v = &v[..v.len() - 1];
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        let mut a = 0.0;
        for (index, sub) in v.split(",").enumerate() {
            match index {
                0 => {
                    r = sub.parse().unwrap();
                }
                1 => {
                    g = sub.parse().unwrap();
                }
                2 => {
                    b = sub.parse().unwrap();
                }
                3 => {
                    a = sub.parse().unwrap();
                }
                _ => {}
            }
        }
        Ok(RGBAColor::new(r, g, b, a))
    }
}

impl<'de> Deserialize<'de> for RGBAColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RGBAColorVisitor)
    }
}

#[derive(Clone, Copy)]
pub struct RectangleVertex {
    pub x: glfloat,
    pub y: glfloat,
    pub u: glfloat,
    pub v: glfloat,
    pub r: glfloat,
    pub g: glfloat,
    pub b: glfloat,
    pub a: glfloat,
}

impl RectangleVertex {
    #[inline(always)]
    pub fn new(x: glfloat, y: glfloat, u: glfloat, v: glfloat, r: glfloat, g: glfloat, b: glfloat, i: glfloat) -> RectangleVertex {
        RectangleVertex { x, y, u, v, r, g, b, a: i }
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
