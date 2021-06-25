use super::types::TextVertex as TVertex;
use crate::ui::{view::View, font::Font, coordinate::Anchor};

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
    data: Vec<TVertex>,
    shader: super::shaders::TextShader,
    reserved_gpu_memory: isize,
}

impl<'a> TextRenderer<'a> {
    pub fn create(shader: super::shaders::TextShader, font: &Font, reserved_space: isize) -> Result<TextRenderer, ()> {
        use std::mem::size_of;
        let stride = size_of::<TVertex>() as gl::types::GLsizei;
        // in the buffer of TVertices, each color attribute is 16 bytes in, namely 4 * sizeof(float) = 4 * 4
        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, reserved_space, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Coordinate & texture coordinate attributes
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            // Color attribute
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, 16 as _);
            gl::EnableVertexAttribArray(1);
            // Unbind this buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        let gl_handle = super::glinit::OpenGLHandle { vao, vbo, ebo: 0 };

        let tdb = TextRenderer { gl_handle, shader, font, pristine: false, data: Vec::with_capacity(reserved_space as usize), reserved_gpu_memory: reserved_space };
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
            gl::DrawArrays(gl::TRIANGLES, 0, self.data.len() as i32);
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32) {
        self.bind();
        let color = super::types::RGBColor { r: 1.0f32, g: 0.0, b: 0.3 };
        let mut current_x = x;
        let current_y = y - self.font.row_height();
        if !self.pristine {
            self.data.clear();
            for c in text.chars() {
                if c == '\n' {
                    todo!("newline characters not yet implemented");
                } 
                if let Some(g) = self.font.get_glyph(c) {
                    // TODO: handle c == '\n'
                    let super::types::RGBColor { r: red, g: green, b: blue } = color;
                    let xpos = current_x as f32 + g.bearing.x as f32;
                    let ypos = current_y as f32 - (g.size.y - g.bearing.y) as f32;
                    let x0 = g.x0 as f32 / self.font.texture_width() as f32;
                    let x1 = g.x1 as f32 / self.font.texture_width() as f32;
                    let y0 = g.y0 as f32 / self.font.texture_height() as f32;
                    let y1 = g.y1 as f32 / self.font.texture_height() as f32;
    
                    let w = g.width();
                    let h = g.height();
                    self.data.push(TVertex::new(xpos, ypos + h, x0, y0, red, green, blue));
                    self.data.push(TVertex::new(xpos, ypos, x0, y1, red, green, blue));
                    self.data.push(TVertex::new(xpos + w, ypos, x1, y1, red, green, blue));
                    self.data.push(TVertex::new(xpos, ypos + h, x0, y0, red, green, blue));
                    self.data.push(TVertex::new(xpos + w, ypos, x1, y1, red, green, blue));
                    self.data.push(TVertex::new(xpos + w, ypos + h, x1, y0, red, green, blue));
                    current_x += g.advance;
                } else {
                    panic!("Could not find glyph for {}", c);
                }
            }
    
            if self.reserved_gpu_memory <= (self.data.len() * std::mem::size_of::<TVertex>()) as _ {
                self.reserved_gpu_memory = (self.data.len() * std::mem::size_of::<TVertex>() * 2) as _;
                unsafe {
                    gl::BufferData(gl::ARRAY_BUFFER, self.reserved_gpu_memory, std::ptr::null(), gl::DYNAMIC_DRAW);
                }
            }
            unsafe {
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.data.len() * std::mem::size_of::<TVertex>()) as _, self.data.as_ptr() as _);
                gl::DrawArrays(gl::TRIANGLES, 0, self.data.len() as i32);
            }
        } else {
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, self.data.len() as i32);
            }
        }
    }

    pub fn draw_view(&mut self, view: &View) {
        self.bind();
        let Anchor(top_x, top_y) = view.anchor;
        self.draw_text("fee fi fo fum, MOTHERFUCKER!", top_x, top_y);
    }

}