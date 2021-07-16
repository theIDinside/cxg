pub mod shaders;
pub mod types;

/// Rect renderer module. Renders simple rectangles, such as windows/borders and cursors
pub mod rect;
/// Text renderer module. Renders text, using Views
pub mod text;
#[macro_use]
pub mod glinit;


pub enum Primitive {
    /// used when dealing with TextVertex data quads
    CharacterQuad(isize),
    /// used when dealing with RectVertex data quads
    RegularQuad(isize),
}

impl Primitive {
    pub fn request_reserve(&self) -> (GPUDataType, GPUDataType) {
        match *self {
            Primitive::CharacterQuad(count) => (
                GPUDataType::TextVertex(4 * count as usize),
                GPUDataType::Index(6 * count as usize),
            ),
            Primitive::RegularQuad(count) => (
                GPUDataType::RectVertex(4 * count as usize),
                GPUDataType::Index(6 * count as usize),
            ),
        }
    }
}

pub enum GPUDataType {
    TextVertex(usize),
    RectVertex(usize),
    Index(usize),
}

impl GPUDataType {
    pub fn bytes_len(&self) -> isize {
        match self {
            GPUDataType::TextVertex(count) => (std::mem::size_of::<types::TextVertex>() * *count) as _,
            GPUDataType::RectVertex(count) => (std::mem::size_of::<types::RectVertex>() * *count) as _,
            GPUDataType::Index(count) => (count * std::mem::size_of::<u32>()) as _,
        }
    }

    pub fn value(&self) -> usize {
        match *self {
            GPUDataType::TextVertex(c) => c,
            GPUDataType::RectVertex(c) => c,
            GPUDataType::Index(c) => c,
        }
    }
}
