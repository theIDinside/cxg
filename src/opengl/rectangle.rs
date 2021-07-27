use crate::{
    datastructure::generic::Vec2f,
    opengl::Primitive,
    ui::basic::{boundingbox::BoundingBox, coordinate::Margin},
};

use super::{
    glinit::OpenGLHandle,
    shaders::RectShader,
    text::BufferIndex,
    types::{RGBColor, RectangleVertex},
};

pub struct Texture {
    pub id: gl::types::GLuint,
}

pub enum PolygonType {
    /// When we see this enum value, we set interpolation float to 0.0, thus sampling 0% from whatever texture is bound
    Undecorated,
    /// When we see this enum value, we set interpolation float to 1.0, thus *only* sampling from the texture (i.e. mix is 100% texture), with id of the Texture parameter in this value
    Decorated {
        /// texture ID, to be bound when drawing the draw command
        texture: Texture,
    },
    /// sample 0% from whatever texture is bound, and use rounded corners, defined by parameter corner_radius
    RoundedUndecorated {
        /// radius of the corners in this polygon, used in the signed distance field calculations
        corner_radius: f32,
    },
    /// sample 100% from the texture bound (texture id passed as parameter) and decorate with rounded corners
    RoundedDecorated {
        /// radius of the corners in this polygon, used in the signed distance field calculations
        corner_radius: f32,
        /// texture ID, to be bound when drawing the draw command
        texture: Texture,
    },
}

/// The draw command, constructed, so that we know what data in the buffer on the GPU looks like, what it requests of us (like, what textures need to be bound, what should the uniforms be set to etc)
pub enum PolygonDrawCommand {
    Undecorated {
        /// Indices into the uploaded memory, so we know what range to draw, in our glDrawElements calls
        indices: BufferIndex,
    },
    RoundedUndecorated {
        /// Indices into the uploaded memory, so we know what range to draw, in our glDrawElements calls
        indices: BufferIndex,
        /// corner radius uniform. Name in shader rectangle.fs.glsl -> radius
        corner_radius: f32,
        /// Uniform for setting the size of the rectangle that is currently being drawn. Is there a better way to do this? Probably fuck yeah. But for now we use a uniform
        /// Name in shader rectangle.vs.glsl -> rect_size
        rect_size: Vec2f,
        bl_rect_screen_pos: Vec2f,
    },
    Decorated {
        /// Indices into the uploaded memory, so we know what range to draw, in our glDrawElements calls
        indices: BufferIndex,
        texture: Texture,
    },
    RoundedDecorated {
        /// Indices into the uploaded memory, so we know what range to draw, in our glDrawElements calls
        indices: BufferIndex,
        corner_radius: f32,
        rect_size: Vec2f,
        bl_rect_screen_pos: Vec2f,
        texture: Texture, // texture id, so that we know which texture to bind, before drawing. Later on we might expand on this Texture type, to involve more optimized, atlassing, somewhat like we do with the fonts
    },
}

pub struct PolygonRenderer {
    gl_handle: OpenGLHandle,
    vtx_data: Vec<RectangleVertex>,
    indices: Vec<u32>,
    pub shader: RectShader,
    reserved_vertex_count: isize,
    reserved_index_count: isize,
    pub needs_update: bool,
    pub draw_commands: Vec<PolygonDrawCommand>,
}

impl PolygonRenderer {
    pub fn create(shader: RectShader, reserve_quads: isize) -> PolygonRenderer {
        use std::mem::size_of;
        let stride = size_of::<RectangleVertex>() as gl::types::GLsizei;
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
            // Screen position vec2<x, y> and Texture coordinates vec2<u, v>, laid out in memory like: vec4[vec2 pos, vec2 uv]
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // Color & interpolation data, laid out in a vec4 like so: vec4[vec3 color, vec1/float interpolation]
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, (4 * size_of::<f32>()) as _);
            gl::EnableVertexAttribArray(1);

            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, reserved_indices_bytes, std::ptr::null(), gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        let gl_handle = OpenGLHandle { vao, vbo, ebo };

        PolygonRenderer {
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

    pub fn push_draw_command(&mut self, rect: BoundingBox, color: RGBColor, poly_type: PolygonType) {
        match poly_type {
            PolygonType::Undecorated => {
                let indices = self.make_vertex_data(rect, color, None);
                self.draw_commands.push(PolygonDrawCommand::Undecorated { indices });
            }
            PolygonType::Decorated { texture } => {
                let indices = self.make_vertex_data(rect, color, Some(&texture));
                self.draw_commands.push(PolygonDrawCommand::Decorated { indices, texture });
            }
            PolygonType::RoundedUndecorated { corner_radius } => {
                let rect_size = rect.size_f32();
                let bl_rect_screen_pos = rect.min.to_f32();
                let indices = self.make_vertex_data(rect, color, None);
                self.draw_commands
                    .push(PolygonDrawCommand::RoundedUndecorated { indices, corner_radius, rect_size, bl_rect_screen_pos });
            }
            PolygonType::RoundedDecorated { corner_radius, texture } => {
                let rect_size = rect.size_f32();
                let bl_rect_screen_pos = rect.min.to_f32();
                let indices = self.make_vertex_data(rect, color, Some(&texture));
                self.draw_commands
                    .push(PolygonDrawCommand::RoundedDecorated { indices, corner_radius, rect_size, bl_rect_screen_pos, texture });
            }
        }
    }

    pub fn make_vertex_data(&mut self, rect: BoundingBox, color: RGBColor, texture: Option<&Texture>) -> BufferIndex {
        let BoundingBox { min, max } = rect;
        let RGBColor { r, g, b } = color;
        let ebo_idx = self.indices.len();
        let vtx_index = self.vtx_data.len() as u32;
        let interpolation = texture.map(|_| 1.0).unwrap_or(0.0);
        self.vtx_data
            .push(RectangleVertex::new(min.x as f32, max.y as f32, 0.0, 1.0, r, g, b, interpolation));
        self.vtx_data
            .push(RectangleVertex::new(min.x as f32, min.y as f32, 0.0, 0.0, r, g, b, interpolation));
        self.vtx_data
            .push(RectangleVertex::new(max.x as f32, min.y as f32, 1.0, 0.0, r, g, b, interpolation));
        self.vtx_data
            .push(RectangleVertex::new(max.x as f32, max.y as f32, 1.0, 1.0, r, g, b, interpolation));
        self.indices.extend_from_slice(&[
            vtx_index,
            vtx_index + 1,
            vtx_index + 2,
            vtx_index,
            vtx_index + 2,
            vtx_index + 3,
        ]);
        self.needs_update = true;
        let elem_count = self.indices.len() - ebo_idx;
        BufferIndex::new(ebo_idx, elem_count)
    }

    pub fn make_bordered_rect(&mut self, rect: BoundingBox, fill_color: RGBColor, border_info: (i32, RGBColor), rect_type: PolygonType) {
        let (border_thickness, border_color) = border_info;
        debug_assert!(border_thickness >= 1, "Border thickness must be set to at least 1 when creating a bordered rectangle");
        let inner_rect = BoundingBox::shrink(&rect, Margin::Perpendicular { h: border_thickness, v: border_thickness });

        let border_polygon_type = match rect_type {
            PolygonType::Undecorated | PolygonType::Decorated { .. } => PolygonType::Undecorated,
            PolygonType::RoundedUndecorated { corner_radius } | PolygonType::RoundedDecorated { corner_radius, .. } => {
                PolygonType::RoundedUndecorated { corner_radius }
            }
        };

        self.push_draw_command(rect, border_color, border_polygon_type);
        self.push_draw_command(inner_rect, fill_color, rect_type);
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
                PolygonDrawCommand::Undecorated { indices } => {
                    self.shader.set_radius(0.0);
                    indices
                }
                PolygonDrawCommand::RoundedUndecorated { indices, corner_radius, rect_size, bl_rect_screen_pos } => {
                    self.shader.set_radius(*corner_radius);
                    self.shader.set_rect_pos(*bl_rect_screen_pos);
                    self.shader.set_rectangle_size(rect_size.clone());
                    indices
                }
                PolygonDrawCommand::Decorated { indices, texture } => {
                    self.shader.set_radius(0.0);
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, texture.id);
                    }
                    indices
                }
                PolygonDrawCommand::RoundedDecorated { indices, corner_radius, rect_size, bl_rect_screen_pos, texture } => {
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, texture.id);
                    }
                    self.shader.set_radius(*corner_radius);
                    self.shader.set_rect_pos(*bl_rect_screen_pos);
                    self.shader.set_rectangle_size(rect_size.clone());
                    indices
                }
            };
            let &BufferIndex { idx_buffer_idx, idx_count } = indices;
            unsafe {
                gl::DrawElements(gl::TRIANGLES, idx_count as _, gl::UNSIGNED_INT, (std::mem::size_of::<u32>() * idx_buffer_idx) as _);
            }
        }
    }
}

/// Private interface
impl PolygonRenderer {
    fn upload_cpu_data(&mut self) {
        unsafe {
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (self.vtx_data.len() * std::mem::size_of::<RectangleVertex>()) as _, self.vtx_data.as_ptr() as _);
            gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0, (self.indices.len() * std::mem::size_of::<u32>()) as _, self.indices.as_ptr() as _);
        }
        self.needs_update = false;
    }

    fn reserve_gpu_memory_if_needed(&mut self) {
        if self.reserved_vertex_count <= self.vtx_data.len() as _ {
            self.reserved_vertex_count = self.vtx_data.capacity() as _;
            unsafe {
                gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<RectangleVertex>() * self.vtx_data.capacity()) as _, std::ptr::null(), gl::DYNAMIC_DRAW);
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
