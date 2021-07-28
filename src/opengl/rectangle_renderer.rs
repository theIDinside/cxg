use crate::{
    datastructure::generic::{Vec2, Vec2f},
    ui::basic::{boundingbox::BoundingBox, coordinate::Margin},
};

use super::{
    glinit::OpenGLHandle,
    shaders::RectShader,
    text_renderer::BufferIndex,
    types::{RGBAColor, RectVertex},
    Primitive,
};

#[derive(Clone, Copy)]
pub enum RectangleType {
    Undecorated,
    Rounded { radius: f32 },
}

pub enum RectDrawCommand {
    Undecorated {
        data_indices: BufferIndex,
    },
    RoundedCorners {
        data_indices: BufferIndex,
        corner_radius: f32,
        rect_size: Vec2f,
        rect_center_screen_pos: Vec2<gl::types::GLfloat>,
    },
}

pub struct RectRenderer {
    gl_handle: OpenGLHandle,
    vtx_data: Vec<RectVertex>,
    indices: Vec<u32>,
    pub shader: RectShader,
    reserved_vertex_count: isize,
    reserved_index_count: isize,
    pub needs_update: bool,
    pub draw_commands: Vec<RectDrawCommand>,
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

            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, 8 as _);
            gl::EnableVertexAttribArray(1);

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, reserved_indices_bytes, std::ptr::null(), gl::DYNAMIC_DRAW);

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
            // color: RGBAColor {r: 0.3,g: 0.34,b: 0.48,a: 1.0,},
            needs_update: true,
            draw_commands: Vec::with_capacity(10),
        }
    }

    pub fn bind(&self) {
        self.gl_handle.bind();
        self.shader.bind();
    }

    pub fn clear_data(&mut self) {
        self.vtx_data.clear();
        self.indices.clear();
        self.draw_commands.clear();
        self.needs_update = true;
    }

    pub fn push_draw_command(&mut self, rect: BoundingBox, color: RGBAColor, rect_type: RectangleType) {
        let ebo_idx = self.indices.len();
        self.add_rect(rect.clone(), color);
        let elem_count = self.indices.len() - ebo_idx;
        let data_indices = BufferIndex::new(ebo_idx, elem_count);
        match rect_type {
            RectangleType::Undecorated => self.draw_commands.push(RectDrawCommand::Undecorated { data_indices }),
            RectangleType::Rounded { radius } => self.draw_commands.push(RectDrawCommand::RoundedCorners {
                data_indices,
                corner_radius: radius,
                rect_size: rect.size_f32(),
                rect_center_screen_pos: rect.min.to_f32(),
            }),
        }
    }

    pub fn add_rect(&mut self, rect: BoundingBox, color: RGBAColor) {
        let BoundingBox { min, max } = rect;
        let vtx_index = self.vtx_data.len() as u32;
        self.vtx_data.push(RectVertex::new(min.x, max.y, color));
        self.vtx_data.push(RectVertex::new(min.x, min.y, color));
        self.vtx_data.push(RectVertex::new(max.x, min.y, color));
        self.vtx_data.push(RectVertex::new(max.x, max.y, color));
        self.indices.extend_from_slice(&[
            vtx_index,
            vtx_index + 1,
            vtx_index + 2,
            vtx_index,
            vtx_index + 2,
            vtx_index + 3,
        ]);
        self.needs_update = true;
    }

    pub fn push_rect(&mut self, rect: BoundingBox, fill_color: RGBAColor, border: Option<(i32, RGBAColor)>, rect_type: RectangleType) {
        if let Some((border_thickness, border_color)) = border {
            let inner_rect = BoundingBox::shrink(&rect, Margin::Perpendicular { h: border_thickness, v: border_thickness });
            self.push_draw_command(rect, border_color, rect_type);
            self.push_draw_command(inner_rect, fill_color, rect_type);
        } else {
            self.push_draw_command(rect, fill_color, rect_type);
        }
    }

    pub fn set_rect(&mut self, rect: BoundingBox, color: RGBAColor) {
        self.clear_data();
        self.add_rect(rect, color);
    }

    pub fn set_color(&mut self, color: RGBAColor) {
        for v in self.vtx_data.iter_mut() {
            v.color = color;
        }
        self.needs_update = true;
    }

    pub fn draw_list(&mut self) {
        self.bind();
        if self.needs_update {
            self.reserve_gpu_memory_if_needed();
            self.upload_cpu_data();
            self.needs_update = false;
        }
        for dc in self.draw_commands.iter() {
            let indices = match dc {
                RectDrawCommand::Undecorated { data_indices } => {
                    self.shader.set_radius(0.0);
                    data_indices
                }
                RectDrawCommand::RoundedCorners { data_indices, corner_radius, rect_size, rect_center_screen_pos } => {
                    // todo(feature) handle different setup and options
                    // that we can pass to this draw command. right now it does nothing.
                    self.shader.set_radius(*corner_radius);
                    self.shader.set_rect_pos(*rect_center_screen_pos);
                    self.shader.set_rectangle_size(rect_size.clone());
                    data_indices
                }
            };
            let BufferIndex { idx_buffer_idx, idx_count } = indices;
            unsafe {
                gl::DrawElements(gl::TRIANGLES, (*idx_count) as _, gl::UNSIGNED_INT, (std::mem::size_of::<u32>() * *idx_buffer_idx) as _);
            }
        }
    }

    pub fn draw(&mut self) {
        self.bind();
        self.shader.set_radius(0.0);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        if self.needs_update {
            self.reserve_gpu_memory_if_needed();
            self.upload_cpu_data();
            self.needs_update = false;
        }
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as _, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
}

/// Private interface
impl RectRenderer {
    fn upload_cpu_data(&mut self) {
        unsafe {
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.vtx_data.len() * std::mem::size_of::<RectVertex>()) as _, self.vtx_data.as_ptr() as _);
            gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0, (self.indices.len() * std::mem::size_of::<u32>()) as _, self.indices.as_ptr() as _);
        }
        self.needs_update = false;
    }

    fn reserve_gpu_memory_if_needed(&mut self) {
        if self.reserved_vertex_count <= self.vtx_data.len() as _ {
            self.reserved_vertex_count = self.vtx_data.capacity() as _;
            unsafe {
                gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<RectVertex>() * self.vtx_data.capacity()) as _, std::ptr::null(), gl::DYNAMIC_DRAW);
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
