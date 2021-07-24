use super::{
    types::{RGBColor, TextVertex as TVertex},
    Primitive,
};
use crate::{
    datastructure::generic::Vec2i,
    debugger_catch,
    ui::{
        basic::coordinate::{PointArithmetic, Size},
        basic::frame::Frame,
        font::{Font, GlyphInfo},
    },
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

pub struct TextRenderer<'a> {
    gl_handle: super::glinit::OpenGLHandle,
    pub font: &'a Font,
    pub pristine: bool,
    vtx_data: Vec<TVertex>,
    indices: Vec<u32>,
    shader: super::shaders::TextShader,
    reserved_vertex_count: isize,
    reserved_index_count: isize,
}

/// Public interface
impl<'a> TextRenderer<'a> {
    pub fn create(shader: super::shaders::TextShader, font: &'a Font, reserve_quads: usize) -> TextRenderer<'a> {
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
            font,
            pristine: false,
            vtx_data: Vec::with_capacity(vertices_count.value()),
            indices,
            reserved_vertex_count: vertices_count.value() as _,
            reserved_index_count: reserved_indices.value() as _,
        };
        tdb
    }

    pub fn bind(&self) {
        self.gl_handle.bind();
        self.shader.bind();
        self.font.bind();
    }

    pub fn draw_clipped(&mut self, clip_frame: Frame) {
        let Frame { anchor: Vec2i { x: top_x, y: top_y }, size } = clip_frame;
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(top_x, top_y - size.height, size.width, size.height);
        }
        self.draw();
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }

    pub fn draw(&mut self) {
        self.bind();
        if !self.pristine {
            self.reserve_gpu_memory_if_needed();
            self.upload_cpu_data();
            self.pristine = true;
        }
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as _, gl::UNSIGNED_INT, std::ptr::null());
            // gl::DrawArrays(gl::TRIANGLES, 0, self.vtx_data.len() as i32);
        }
    }

    pub fn append_data_from_iterator<'b>(&mut self, text: impl ExactSizeIterator<Item = &'b char>, color: RGBColor, x: i32, y: i32) {
        let mut current_x = x;
        let mut current_y = y - self.font.row_height();
        // we need to be able to peek ahead
        let mut text = text.peekable();
        while let Some(c) = text.next() {
            let c = *c;
            if c == '\n' {
                current_x = x;
                current_y -= self.font.row_height();
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

            if let Some(g) = self.font.get_glyph(c) {
                let super::types::RGBColor { r: red, g: green, b: blue } = color;
                let xpos = current_x as f32 + g.bearing.x as f32;
                let ypos = current_y as f32 - (g.size.y - g.bearing.y) as f32;
                let x0 = g.x0 as f32 / self.font.texture_width() as f32;
                let x1 = g.x1 as f32 / self.font.texture_width() as f32;
                let y0 = g.y0 as f32 / self.font.texture_height() as f32;
                let y1 = g.y1 as f32 / self.font.texture_height() as f32;

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
        self.pristine = false;
    }

    pub fn append_data<'b>(&mut self, text: impl ExactSizeIterator<Item = &'b char>, x: i32, y: i32) {
        let color = super::types::RGBColor { r: 1.0f32, g: 1.0, b: 1.3 };
        self.append_data_from_iterator(text, color, x, y);
    }

    pub fn prepare_data_from_iterator<'b>(&mut self, text: impl ExactSizeIterator<Item = &'b char>, text_color: RGBColor, x: i32, y: i32) {
        let color = text_color;
        self.clear_data();

        self.vtx_data.reserve(crate::diff!(self.vtx_data.capacity(), text.len()));

        let mut current_x = x;
        let mut current_y = y - self.font.row_height();
        // we need to be able to peek ahead
        let mut text = text.peekable();
        while let Some(c) = text.next() {
            let c = *c;
            if c == '\n' {
                current_x = x;
                current_y -= self.font.row_height();
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

            if let Some(g) = self.font.get_glyph(c) {
                let super::types::RGBColor { r: red, g: green, b: blue } = color;
                let xpos = current_x as f32 + g.bearing.x as f32;
                let ypos = current_y as f32 - (g.size.y - g.bearing.y) as f32;
                let x0 = g.x0 as f32 / self.font.texture_width() as f32;
                let x1 = g.x1 as f32 / self.font.texture_width() as f32;
                let y0 = g.y0 as f32 / self.font.texture_height() as f32;
                let y1 = g.y1 as f32 / self.font.texture_height() as f32;

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
        self.pristine = false;
    }

    pub fn prepare_data_from_iter<'b>(&mut self, text: impl ExactSizeIterator<Item = &'b char>, x: i32, y: i32) {
        let color = super::types::RGBColor { r: 1.0f32, g: 1.0, b: 1.3 };
        self.prepare_data_from_iterator(text, color, x, y);
    }

    pub fn get_cursor_width_size(&self) -> i32 {
        self.font.get_max_glyph_width()
    }

    pub fn get_glyph(&self, ch: char) -> Option<&GlyphInfo> {
        self.font.get_glyph(ch)
    }

    pub fn calculate_text_dimensions(&self, text: &[char]) -> Size {
        let mut size = Size { width: 0, height: self.font.row_height() };
        let mut max_x = 0;
        for (index, &c) in text.iter().enumerate() {
            if c == '\n' {
                size.height += self.font.row_height();
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
                    let g = match text.get(index - 1) {
                        Some('<') | Some('>') | Some('!') => None,
                        _ => self.get_glyph(c),
                    };
                    size.width += g.unwrap().advance;
                } else {
                    size.width += self.get_glyph(c).unwrap().advance;
                }
            }
            max_x = std::cmp::max(size.width, max_x);
        }

        size.width = max_x + 20;
        size
    }

    pub fn dimensions_of_text_line(&self, text: &[char]) -> Size {
        // todo(feature): implement function so that it can calculate the dimensions of text that spans lines
        debugger_catch!(
            !text.contains(&'\n'),
            crate::DebuggerCatch::Handle("This function can only correctly calculate the dimensions of a single text line".into())
        );

        let parse_special_symbols = |(index, &c): (usize, &char)| {
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
                match text.get(index - 1) {
                    Some('<') | Some('>') | Some('!') => None,
                    _ => self.get_glyph(c),
                }
            } else {
                self.get_glyph(c)
            }
        };

        text.iter()
            .enumerate()
            .filter_map(parse_special_symbols)
            // .map(|&c| self.get_glyph(c).map(|g| g.advance).unwrap_or(self.get_cursor_width_size()))
            .map(|glyph_info| glyph_info.advance)
            .fold(Size { width: 0i32, height: self.font.row_height() }, |acc, v| Size::vector_add(acc, Vec2i { x: v, y: 0 }))
    }
}

/// Private interface
impl<'a> TextRenderer<'a> {
    fn upload_cpu_data(&self) {
        unsafe {
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.vtx_data.len() * std::mem::size_of::<TVertex>()) as _, self.vtx_data.as_ptr() as _);
            gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0, (self.indices.len() * std::mem::size_of::<u32>()) as _, self.indices.as_ptr() as _);
        }
    }

    pub fn clear_data(&mut self) {
        self.vtx_data.clear();
        self.indices.clear();
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
