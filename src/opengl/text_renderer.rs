use std::rc::Rc;

use super::{
    types::{RGBColor, TextVertex as TVertex},
    Primitive,
};
use crate::{
    datastructure::generic::Vec2i,
    ui::{basic::coordinate::Size, basic::frame::Frame, font::Font},
};

#[derive(PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct RendererId(pub u32);

impl std::ops::Deref for RendererId {
    type Target = u32;

    fn deref<'a>(&'a self) -> &'a Self::Target {
        &self.0
    }
}

impl Into<RendererId> for u32 {
    fn into(self) -> RendererId {
        RendererId(self)
    }
}

pub struct BufferIndex {
    pub idx_buffer_idx: usize,
    pub idx_count: usize,
}

impl BufferIndex {
    pub fn new(buffer_index: usize, index_count: usize) -> BufferIndex {
        BufferIndex { idx_buffer_idx: buffer_index, idx_count: index_count }
    }
}

pub struct TextDrawCommand {
    font: Rc<Font>,
    data_indices: BufferIndex,
}

impl TextDrawCommand {
    pub fn new(font: Rc<Font>, data_indices: BufferIndex) -> TextDrawCommand {
        TextDrawCommand { font, data_indices }
    }
}

pub struct TextRenderer {
    gl_handle: super::glinit::OpenGLHandle,
    pub pristine: bool,
    vtx_data: Vec<TVertex>,
    indices: Vec<u32>,
    pub shader: super::shaders::TextShader,
    reserved_vertex_count: isize,
    reserved_index_count: isize,
    pub draw_commands: Vec<TextDrawCommand>,
}

/// Public interface
impl TextRenderer {
    pub fn create(shader: super::shaders::TextShader, reserve_quads: usize) -> TextRenderer {
        use std::mem::size_of;
        let stride = size_of::<TVertex>() as gl::types::GLsizei;

        let reserve_primitive = Primitive::CharacterQuad(reserve_quads as _);
        let (vertices_count, reserved_indices) = reserve_primitive.request_reserve();

        let reserved_vtx_bytes = vertices_count.bytes_len();
        let reserved_indices_bytes = reserved_indices.bytes_len();

        // in the buffer of TVertices, each color attribute is 16 bytes in, namely 4 * sizeof(float) = 4 * 4
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        let indices = Vec::with_capacity(reserved_indices.value());
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, reserved_vtx_bytes, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Coordinate & texture coordinate attributes
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            // Color attribute
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, 16 as _);
            gl::EnableVertexAttribArray(1);
            // Unbind this buffer

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, reserved_indices_bytes, std::ptr::null(), gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        let gl_handle = super::glinit::OpenGLHandle { vao, vbo, ebo };

        let tdb = TextRenderer {
            gl_handle,
            shader,
            pristine: false,
            vtx_data: Vec::with_capacity(vertices_count.value()),
            indices,
            reserved_vertex_count: vertices_count.value() as _,
            reserved_index_count: reserved_indices.value() as _,
            draw_commands: Vec::with_capacity(10),
        };
        tdb
    }

    pub fn bind(&self) {
        self.gl_handle.bind();
        self.shader.bind();
    }

    pub fn push_draw_command(&mut self, text: impl Iterator<Item = char>, color: RGBColor, x: i32, y: i32, font: Rc<Font>) {
        use TextDrawCommand as DC;
        let mut current_x = x;
        let mut current_y = y - font.row_height();
        // we need to be able to peek ahead
        let mut text = text.peekable();
        let ebo_idx = self.indices.len();
        while let Some(c) = text.next() {
            if c == '\n' {
                current_x = x;
                current_y -= font.row_height();
                continue;
            }

            let c = {
                let resulting_unicode = match text.peek() {
                    Some('=') => match c {
                        '<' => unsafe { std::char::from_u32_unchecked(0x2264) },
                        '>' => unsafe { std::char::from_u32_unchecked(0x2265) },
                        '!' => unsafe { std::char::from_u32_unchecked(0x2260) },
                        _ => c,
                    },
                    _ => c,
                };
                if resulting_unicode != c {
                    text.next();
                }
                resulting_unicode
            };

            if let Some(g) = font.get_glyph(c) {
                let RGBColor { r: red, g: green, b: blue } = color;
                let xpos = current_x as f32 + g.bearing.x as f32;
                let ypos = current_y as f32 - (g.size.y - g.bearing.y) as f32;
                let x0 = g.x0 as f32 / font.texture_width() as f32;
                let x1 = g.x1 as f32 / font.texture_width() as f32;
                let y0 = g.y0 as f32 / font.texture_height() as f32;
                let y1 = g.y1 as f32 / font.texture_height() as f32;

                let w = g.width();
                let h = g.height();

                let vtx_index = self.vtx_data.len() as u32;
                // Todo(optimization, avx, simd): TVertex has been padded with an extra float, (sizeof TVertex == 8 * 4 bytes == 128 bit. Should be *extremely* friendly for SIMD purposes now)

                self.vtx_data.push(TVertex::new(xpos, ypos + h, x0, y0, red, green, blue));
                self.vtx_data.push(TVertex::new(xpos, ypos, x0, y1, red, green, blue));
                self.vtx_data.push(TVertex::new(xpos + w, ypos, x1, y1, red, green, blue));
                self.vtx_data.push(TVertex::new(xpos + w, ypos + h, x1, y0, red, green, blue));

                self.indices.extend_from_slice(&[
                    vtx_index,
                    vtx_index + 1,
                    vtx_index + 2,
                    vtx_index,
                    vtx_index + 2,
                    vtx_index + 3,
                ]);
                current_x += g.advance;
            } else {
                let mut buf = [0; 4];
                c.encode_utf16(&mut buf);
                panic!("Could not find glyph for {}, {:?}", c, buf);
            }
        }

        let elem_count = self.indices.len() - ebo_idx;
        self.draw_commands.push(DC::new(font, BufferIndex::new(ebo_idx, elem_count)));
        self.pristine = false;
    }

    pub fn draw_list(&mut self) {
        self.gl_handle.bind();
        if !self.pristine {
            self.reserve_gpu_memory_if_needed();
            self.upload_cpu_data();
            self.pristine = true;
        }
        self.shader.bind();
        // todo(optimization): this means we can smash together consecutive DrawCommands that use the same settings & configurations, thus reducing the draw calls
        for TextDrawCommand { font, data_indices: BufferIndex { idx_buffer_idx, idx_count }, .. } in self.draw_commands.iter() {
            font.bind();
            unsafe {
                gl::DrawElements(gl::TRIANGLES, (*idx_count) as _, gl::UNSIGNED_INT, (std::mem::size_of::<u32>() * *idx_buffer_idx) as _);
            }
        }
    }

    pub fn draw_clipped_list(&mut self, clip_frame: Frame) {
        let Frame { anchor: Vec2i { x: top_x, y: top_y }, size } = clip_frame;
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(top_x, top_y - size.height, size.width, size.height);
        }
        self.draw_list();
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}

/// Private interface
impl TextRenderer {
    fn upload_cpu_data(&self) {
        unsafe {
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.vtx_data.len() * std::mem::size_of::<TVertex>()) as _, self.vtx_data.as_ptr() as _);
            gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0, (self.indices.len() * std::mem::size_of::<u32>()) as _, self.indices.as_ptr() as _);
        }
    }

    pub fn clear_data(&mut self) {
        self.vtx_data.clear();
        self.indices.clear();
        self.draw_commands.clear();
    }

    fn reserve_gpu_memory_if_needed(&mut self) {
        if self.reserved_vertex_count <= self.vtx_data.len() as _ {
            self.reserved_vertex_count = self.vtx_data.capacity() as _;
            unsafe {
                gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<TVertex>() * self.vtx_data.capacity()) as _, std::ptr::null(), gl::DYNAMIC_DRAW);
            }
        }

        if self.reserved_index_count <= self.indices.len() as _ {
            self.reserved_index_count = self.indices.capacity() as _;
            unsafe {
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (std::mem::size_of::<u32>() * self.indices.capacity()) as _, std::ptr::null(), gl::DYNAMIC_DRAW);
            }
        }
    }
}

// Calculates the size required for the bounding box to cover to be able to hold this text
pub fn calculate_text_dimensions(text: &[char], font: &Font) -> Size {
    let mut size = Size { width: 0, height: font.row_height() };
    let mut max_x = 0;
    for (index, &c) in text.iter().enumerate() {
        if c == '\n' {
            size.height += font.row_height();
            size.width = 0;
        } else {
            let c = if c == '<' || c == '>' || c == '!' {
                if let Some('=') = text.get(index + 1) {
                    let resulting_unicode_char = if c == '<' {
                        unsafe { std::char::from_u32_unchecked(0x2264) }
                    } else if c == '>' {
                        unsafe { std::char::from_u32_unchecked(0x2265) }
                    } else {
                        unsafe { std::char::from_u32_unchecked(0x2260) }
                    };
                    resulting_unicode_char
                } else {
                    c
                }
            } else {
                c
            };
            if c == '=' {
                size.width += match text.get(index - 1) {
                    Some('<') | Some('>') | Some('!') => None,
                    _ => font.get_glyph(c),
                }
                .map_or(0, |g| g.advance);
            } else {
                size.width += font.get_glyph(c).unwrap().advance;
            }
        }
        max_x = std::cmp::max(size.width, max_x);
    }

    size.width = max_x;
    size
}
