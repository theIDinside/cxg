use super::{Primitive, types::TextVertex as TVertex};
use crate::ui::{font::{Font, GlyphInfo}};

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
    font: &'a Font,
    pub pristine: bool,
    vtx_data: Vec<TVertex>,
    indices: Vec<u32>,
    shader: super::shaders::TextShader,
    reserved_vertex_count: isize,
    reserved_index_count: isize,
}

pub struct CharRectInfo<'a> {
    g: &'a crate::ui::font::GlyphInfo,
    font: &'a Font,
    pos: crate::datastructure::generic::Vec2f,
    color: &'a super::types::RGBColor
}

pub fn create_char_rect_vertices(info: CharRectInfo) -> [TVertex; 4] {
    use crate::datastructure::generic::Vec2f;
    let CharRectInfo {g, font, pos, color} = info;
    let &super::types::RGBColor {r: red,g: green,b: blue} = color;
    let Vec2f { x, y} = pos;
    let xpos = x;
    let ypos = y;
    let x0 = g.x0 as f32 / font.texture_width() as f32;
    let x1 = g.x1 as f32 / font.texture_width() as f32;
    let y0 = g.y0 as f32 / font.texture_height() as f32;
    let y1 = g.y1 as f32 / font.texture_height() as f32;

    let w = g.width();
    let h = g.height();

    [TVertex { x: xpos,      y: ypos + h,    u: x0, v: y0, r: red, g: green, b: blue }, 
     TVertex { x: xpos,      y: ypos,        u: x0, v: y1, r: red, g: green, b: blue },
     TVertex { x: xpos + w,  y: ypos + h,    u: x1, v: y0, r: red, g: green, b: blue },
     TVertex { x: xpos + w,  y: ypos + h,    u: x1, v: y0, r: red, g: green, b: blue }]
}

/// Public interface
impl<'a> TextRenderer<'a> {
    pub fn create(shader: super::shaders::TextShader, font: &Font, reserve_quads: usize) -> Result<TextRenderer, ()> {
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
            reserved_index_count: reserved_indices.value() as _ };


        Ok(tdb)
    }

    pub fn bind(&self) {
        self.gl_handle.bind();
        self.shader.bind();
        self.font.bind();
    }

    pub fn draw(&self) {
        self.bind();
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as _, gl::UNSIGNED_INT, std::ptr::null());
            // gl::DrawArrays(gl::TRIANGLES, 0, self.vtx_data.len() as i32);
        }
    }

    /// Takes a reference to the range of text, currently displayable on the view and renders it. 
    pub fn push_data(&mut self, text: &[char], x: i32, y: i32) {
        self.bind();
        let color = super::types::RGBColor { r: 1.0f32, g: 0.0, b: 0.3 };
        let mut current_x = x;
        let mut current_y = y - self.font.row_height();
        self.clear_data();
        for c in text {
            let c = *c;
            if c == '\n' {
                current_x = x;
                current_y -= self.font.row_height();
                continue;
            } 
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
                self.vtx_data.push(TVertex { x: xpos, y: ypos + h,  u: x0, v: y0, r: red, g: green, b: blue});
                self.vtx_data.push(TVertex { x: xpos, y: ypos, u: x0, v: y1, r: red, g: green, b: blue});
                self.vtx_data.push(TVertex { x: xpos + w, y: ypos, u: x1, v: y1, r: red, g: green, b: blue});
                self.vtx_data.push(TVertex { x: xpos + w, y: ypos + h,  u: x1, v: y0, r: red, g: green, b: blue});

                self.indices.extend_from_slice(&[vtx_index, vtx_index+1, vtx_index+2, vtx_index, vtx_index+2, vtx_index+3]);
                current_x += g.advance;
            } else {
                panic!("Could not find glyph for {}", c);
            }
        }
        self.reserve_gpu_memory_if_needed();
        self.upload_cpu_data();
        self.pristine = false;
    } 

    pub fn get_cursor_width_size(&self) -> i32 {
        self.font.get_max_glyph_width()
    }

    pub fn get_glyph(&self, ch: char) -> Option<&GlyphInfo> {
        self.font.get_glyph(ch)
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

    fn clear_data(&mut self) {
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