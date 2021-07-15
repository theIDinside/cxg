use crate::ui::{
    boundingbox::BoundingBox,
    coordinate::{Anchor, Size},
};

use super::{
    glinit::OpenGLHandle,
    shaders::RectShader,
    types::{RGBAColor, RectVertex},
    Primitive,
};

pub struct RectRenderer {
    gl_handle: OpenGLHandle,
    vtx_data: Vec<RectVertex>,
    indices: Vec<u32>,
    pub shader: RectShader,
    reserved_vertex_count: isize,
    reserved_index_count: isize,
    color: RGBAColor,
}

/// Public interface
impl RectRenderer {
    pub fn create(shader: RectShader, reserve_quads: isize) -> RectRenderer {
        use std::mem::size_of;
        let stride = size_of::<RectVertex>() as gl::types::GLsizei;
        let reserve_primitive = Primitive::RegularQuad(reserve_quads);
        let (vertices_count, reserved_indices) = reserve_primitive.request_reserve();
        let reserved_vtx_bytes = vertices_count.bytes_len();
        let reserved_indices_bytes = reserved_indices.bytes_len();
        let indices = Vec::with_capacity(reserved_indices.value());

        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, reserved_vtx_bytes, std::ptr::null(), gl::DYNAMIC_DRAW);
            // Coordinate & texture coordinate attributes
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            // Unbind this buffer

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                reserved_indices_bytes,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        let gl_handle = OpenGLHandle { vao, vbo, ebo };

        RectRenderer {
            gl_handle,
            vtx_data: Vec::with_capacity(vertices_count.value()),
            indices,
            shader,
            reserved_vertex_count: vertices_count.value() as _,
            reserved_index_count: reserved_indices.value() as _,
            color: RGBAColor {
                r: 0.3,
                g: 0.34,
                b: 0.48,
                a: 1.0,
            },
        }
    }

    pub fn bind(&self) {
        self.gl_handle.bind();
        self.shader.bind();
    }

    pub fn update_rectangle(&mut self, anchor: Anchor, size: Size) {
        self.bind();
        self.clear_data();
        self.push_rect(BoundingBox::from((anchor, size)));
        self.reserve_gpu_memory_if_needed();
        self.upload_cpu_data();
    }

    pub fn push_rect(&mut self, rect: BoundingBox) {
        self.bind();
        let BoundingBox { min, max } = &rect;

        let vtx_index = self.vtx_data.len() as u32;
        self.vtx_data.push(RectVertex::new(min.x, max.y));
        self.vtx_data.push(RectVertex::new(min.x, min.y));
        self.vtx_data.push(RectVertex::new(max.x, min.y));
        self.vtx_data.push(RectVertex::new(max.x, max.y));
        self.indices.extend_from_slice(&[
            vtx_index,
            vtx_index + 1,
            vtx_index + 2,
            vtx_index,
            vtx_index + 2,
            vtx_index + 3,
        ]);

        self.reserve_gpu_memory_if_needed();
        self.upload_cpu_data();
    }

    pub fn set_rect(&mut self, rect: BoundingBox) {
        self.clear_data();
        self.push_rect(rect);
    }

    pub fn set_color(&mut self, color: RGBAColor) {
        self.color = color;
    }

    pub fn draw(&self) {
        self.bind();
        self.shader.set_color(self.color);
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as _, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
}

/// Private interface
impl RectRenderer {
    fn upload_cpu_data(&self) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.vtx_data.len() * std::mem::size_of::<RectVertex>()) as _,
                self.vtx_data.as_ptr() as _,
            );
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                0,
                (self.indices.len() * std::mem::size_of::<u32>()) as _,
                self.indices.as_ptr() as _,
            );
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
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (std::mem::size_of::<RectVertex>() * self.vtx_data.capacity()) as _,
                    std::ptr::null(),
                    gl::DYNAMIC_DRAW,
                );
            }
        }

        if self.reserved_index_count <= self.indices.len() as _ {
            self.reserved_index_count = self.indices.capacity() as _;
            unsafe {
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (std::mem::size_of::<u32>() * self.indices.capacity()) as _,
                    std::ptr::null(),
                    gl::DYNAMIC_DRAW,
                );
            }
        }
    }
}
