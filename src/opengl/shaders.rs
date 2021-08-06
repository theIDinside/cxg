use std::{io::Read, path::Path};

use crate::datastructure::generic::{Vec2, Vec2f};

/// Default shader sources, compiled into the binary
pub mod source {
    // pub const RECT_VERTEX_SHADER: &str = include_str!("../assets/rect.vs.glsl");
    // pub const RECT_FRAGMENT_SHADER: &str = include_str!("../assets/rect.fs.glsl");
    pub const TEXT_VERTEX_SHADER: &str = include_str!("../assets/text.vs.glsl");
    pub const TEXT_FRAGMENT_SHADER: &str = include_str!("../assets/text.fs.glsl");
}

#[derive(Clone)]
pub struct TextShader {
    id: gl::types::GLuint,
    projection_uniform: gl::types::GLint,
}

impl TextShader {
    pub fn new() -> TextShader {
        let font_program = match super::glinit::create_shader_program(source::TEXT_VERTEX_SHADER, source::TEXT_FRAGMENT_SHADER) {
            Ok(program) => program,
            Err(e) => {
                println!("Error creating Rectangle shader program. Exiting application. {:?}", e);
                std::process::exit(1);
            }
        };
        let projection_uniform = unsafe {
            let uniform_name = std::ffi::CString::new("projection").expect("Failed to create CString");
            gl::GetUniformLocation(font_program, uniform_name.as_ptr())
        };
        assert_ne!(projection_uniform, -1);
        TextShader { id: font_program, projection_uniform }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_projection(&self, projection: &super::types::Matrix) {
        self.bind();
        unsafe {
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());
            // gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, d.as_ptr() as *const _);
        }
    }
}

#[derive(Clone, Debug)]
pub struct RectShader {
    pub id: gl::types::GLuint,
    u_projection: gl::types::GLint,
    u_radius: gl::types::GLint,
    u_rect_size: gl::types::GLint,
    u_rect_pos: gl::types::GLint,
    u_use_texture: gl::types::GLint,
}

impl RectShader {
    pub fn panic_if_not_ok(&self, err_msg: &'static str) {
        assert!(self.u_projection >= 0, "{}: projection uniform ID invalid: {}", err_msg, self.u_projection);
        assert!(self.u_radius >= 0, "{}: radius uniform ID invalid: {}", err_msg, self.u_radius);
        assert!(self.u_rect_size >= 0, "{}: rectangle size uniform ID invalid: {}", err_msg, self.u_rect_size);
        assert!(self.u_rect_pos >= 0, "{}: rectangle position uniform ID invalid: {}", err_msg, self.u_rect_pos);
        assert!(self.u_use_texture >= 0, "{}: use_texture uniform ID invalid: {}", err_msg, self.u_use_texture);
        println!("Validated shader uniforms & locations; {:#?}", self);
    }

    pub fn new(vs_path: &Path, fs_path: &Path) -> RectShader {
        let rvs = std::fs::File::open(vs_path).and_then(|mut f| {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            Ok(s)
        });

        let rfs = std::fs::File::open(fs_path).and_then(|mut f| {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            Ok(s)
        });

        let font_program = match super::glinit::create_shader_program(&rvs.expect("failed to read RVS code"), &rfs.expect("failed to read RFS code")) {
            Ok(program) => program,
            Err(_) => {
                println!("Error creating Rectangle shader program. Exiting application.");
                std::process::exit(1);
            }
        };
        let (projection_uniform, radius, rect_size, rect_pos, use_texture) = unsafe {
            let projection_uniform_name = std::ffi::CString::new("projection").expect("Failed to create CString");
            let radius = std::ffi::CString::new("radius").expect("Failed to create CString");
            let rect_size = std::ffi::CString::new("rect_size").expect("Failed to create CString");
            let rect_pos = std::ffi::CString::new("rect_pos").expect("Failed to create CString");
            let use_texture_name = std::ffi::CString::new("use_texture").unwrap();
            (
                gl::GetUniformLocation(font_program, projection_uniform_name.as_ptr()),
                gl::GetUniformLocation(font_program, radius.as_ptr()),
                gl::GetUniformLocation(font_program, rect_size.as_ptr()),
                gl::GetUniformLocation(font_program, rect_pos.as_ptr()),
                gl::GetUniformLocation(font_program, use_texture_name.as_ptr()),
            )
        };

        assert_ne!(projection_uniform, -1);
        RectShader {
            id: font_program,
            u_projection: projection_uniform,
            u_radius: radius,
            u_rect_size: rect_size,
            u_rect_pos: rect_pos,
            u_use_texture: use_texture,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_projection(&self, projection: &super::types::Matrix) {
        self.bind();
        unsafe {
            gl::UniformMatrix4fv(self.u_projection, 1, gl::FALSE, projection.as_ptr());
            // gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, d.as_ptr() as *const _);
        }
    }

    pub fn set_use_texture(&self, use_texture: bool) {
        self.bind();
        unsafe {
            gl::Uniform1i(self.u_use_texture, use_texture as _);
            // gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, d.as_ptr() as *const _);
        }
    }

    pub fn set_radius(&self, radius: f32) {
        self.bind();
        unsafe {
            gl::Uniform1f(self.u_radius, radius);
        }
    }

    pub fn set_rectangle_size(&self, size: Vec2f) {
        self.bind();
        unsafe {
            gl::Uniform2fv(self.u_rect_size, 1, &size as *const _ as _);
        }
    }

    pub fn set_rect_pos(&self, p: Vec2<gl::types::GLfloat>) {
        self.bind();
        unsafe {
            gl::Uniform2fv(self.u_rect_pos, 1, &p as *const _ as _);
        }
    }
}
