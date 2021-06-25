use crate::ui::coordinate::{Anchor, Size};

use super::{glinit::OpenGLHandle, shaders::RectShader, types::{RGBAColor, RectVertex}};


pub struct RectRenderer {
    gl_handle: OpenGLHandle,
    data: Vec<RectVertex>,
    shader: RectShader,
    reserved_gpu_memory: isize,
    color: RGBAColor
}

impl RectRenderer {
    pub fn create(shader: RectShader, reserved_space: isize) -> Result<RectRenderer, ()> {
        use std::mem::size_of;
        let stride = size_of::<RectVertex>() as gl::types::GLsizei;
        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, reserved_space, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Coordinate & texture coordinate attributes
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            // Unbind this buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        let gl_handle = OpenGLHandle { vao, vbo, ebo: 0 };

        Ok(RectRenderer {
            gl_handle,
            data: Vec::with_capacity(reserved_space as usize),
            shader,
            reserved_gpu_memory: reserved_space,
            color: RGBAColor{r: 0.3, g: 0.34, b: 0.48, a: 1.0}
        })
    }

    pub fn bind(&self) {
        self.gl_handle.bind();
        self.shader.bind();
    }

    pub fn update_rectangle(&mut self, anchor: Anchor, size: Size) {
        self.bind();
        self.data.clear();
        let Anchor(xpos, ypos) = anchor;
        let w = size.width;
        let h = size.height;
        self.data.push(RectVertex::new(xpos, ypos));
        self.data.push(RectVertex::new(xpos, ypos - h));
        self.data.push(RectVertex::new(xpos + w, ypos - h));
        self.data.push(RectVertex::new(xpos, ypos));
        self.data.push(RectVertex::new(xpos + w, ypos - h));
        self.data.push(RectVertex::new(xpos + w, ypos));
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER, self.reserved_gpu_memory, std::ptr::null(), gl::DYNAMIC_DRAW);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.data.len() * std::mem::size_of::<RectVertex>()) as _, self.data.as_ptr() as _);
        }
    }

    pub fn set_color(&mut self, color: RGBAColor) {
        self.color = color;
    }

    pub fn draw2(&self) {
        self.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.data.len() as i32);
        }
    }

    pub fn draw(&self) {
        self.bind();
        self.shader.set_color(self.color);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.data.len() as i32);
        }
    }
}