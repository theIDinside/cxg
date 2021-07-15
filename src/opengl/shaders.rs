use super::types::RGBAColor;

/// Default shader sources, compiled into the binary
pub mod source {
    pub const RECT_VERTEX_SHADER: &str = include_str!("../assets/rect.vs.glsl");
    pub const RECT_FRAGMENT_SHADER: &str = include_str!("../assets/rect.fs.glsl");
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
            Err(_) => {
                println!("Error creating font shader program. Exiting application.");
                std::process::exit(1);
            }
        };
        let projection_uniform = unsafe {
            let uniform_name = std::ffi::CString::new("projection").expect("Failed to create CString");
            gl::GetUniformLocation(font_program, uniform_name.as_ptr())
        };

        println!("uniform location of projection: {}", projection_uniform);
        assert_ne!(projection_uniform, -1);
        TextShader {
            id: font_program,
            projection_uniform,
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
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());
            // gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, d.as_ptr() as *const _);
        }
    }
}

#[derive(Clone)]
pub struct RectShader {
    id: gl::types::GLuint,
    projection_uniform: gl::types::GLint,
    color_uniform: gl::types::GLint,
}

impl RectShader {
    pub fn new() -> RectShader {
        let font_program = match super::glinit::create_shader_program(source::RECT_VERTEX_SHADER, source::RECT_FRAGMENT_SHADER) {
            Ok(program) => program,
            Err(_) => {
                println!("Error creating font shader program. Exiting application.");
                std::process::exit(1);
            }
        };
        let (projection_uniform, color_uniform) = unsafe {
            let projection_uniform_name = std::ffi::CString::new("projection").expect("Failed to create CString");
            let color_uniform_name = std::ffi::CString::new("fillcolor").expect("Failed to create CString");
            (
                gl::GetUniformLocation(font_program, projection_uniform_name.as_ptr()),
                gl::GetUniformLocation(font_program, color_uniform_name.as_ptr()),
            )
        };

        println!("uniform location of projection: {}", projection_uniform);
        assert_ne!(projection_uniform, -1);
        RectShader {
            id: font_program,
            projection_uniform,
            color_uniform,
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
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());
            // gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, d.as_ptr() as *const _);
        }
    }

    pub fn set_color(&self, color: RGBAColor) {
        unsafe {
            gl::Uniform4fv(self.color_uniform, 1, &color as *const _ as _);
        }
    }
}
