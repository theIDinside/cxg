use super::types::{Matrix, Vec4f};
use crate::MainInitError;

use gl::{CompileShader, CreateProgram, GetProgramInfoLog, GetProgramiv, GetShaderInfoLog, GetShaderiv, ShaderSource};
use std::ffi::CString;

pub struct OpenGLHandle {
    pub vao: gl::types::GLuint,
    pub vbo: gl::types::GLuint,
    pub ebo: gl::types::GLuint,
}

impl OpenGLHandle {
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        }
    }
}

pub unsafe fn init_gl() {
    gl::Enable(gl::BLEND);
    gl::Enable(gl::CULL_FACE);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    let mut flags = -1;
    gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags);
    if flags & gl::CONTEXT_FLAG_DEBUG_BIT as i32 == 0 {
        println!("Setting debug output function");
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_output), std::ptr::null());
        gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, std::ptr::null(), gl::TRUE);
    }
}

pub fn screen_projection_matrix(width: u32, height: u32, scrolled: i32) -> Matrix {
    let a = Vec4f::new(2.0f32 / width as f32, 0f32, 0f32, 0f32);
    let b = Vec4f::new(0f32, 2f32 / (height as i32 - scrolled) as f32, 0f32, 0f32);
    let c = Vec4f::new(0f32, 0f32, -1f32, 0f32);
    let d = Vec4f::new(
        -1f32,
        -((height as i32 + scrolled) as f32 / (height as i32 - scrolled) as f32),
        0f32,
        1f32,
    );
    Matrix { data: [a, b, c, d] }
}

pub fn create_shader_program(vertex_source: &str, frag_source: &str) -> Result<gl::types::GLuint, MainInitError> {
    println!("Compiling shader:");
    println!("{}", vertex_source);
    println!("{}", frag_source);
    let program = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let v_src = CString::new(vertex_source.as_bytes()).unwrap();
        ShaderSource(vertex_shader, 1, &v_src.as_ptr(), std::ptr::null());
        CompileShader(vertex_shader);

        let mut ok = gl::FALSE as gl::types::GLint;
        let mut log = Vec::with_capacity(512);

        log.set_len(512 - 1);
        GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut ok);
        if ok != gl::TRUE as gl::types::GLint {
            GetShaderInfoLog(
                vertex_shader,
                512,
                std::ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "Compilation of vertex shader failed:\n{}",
                std::str::from_utf8(&log).unwrap_or("Failed to retrieve error message from OpenGL")
            );
            return Err(MainInitError::Shader(String::from_utf8(log).unwrap()));
        }
        log.clear();

        let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let f_src = CString::new(frag_source.as_bytes()).unwrap();
        ShaderSource(frag_shader, 1, &f_src.as_ptr(), std::ptr::null());
        CompileShader(frag_shader);

        GetShaderiv(frag_shader, gl::COMPILE_STATUS, &mut ok);
        if ok != gl::TRUE as gl::types::GLint {
            GetShaderInfoLog(
                frag_shader,
                512,
                std::ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "Compilation of fragment shader failed:\n{}",
                std::str::from_utf8(&log).unwrap_or("Failed to retrieve error message from OpenGL")
            );
            return Err(MainInitError::Shader(String::from_utf8(log).unwrap()));
        }
        log.clear();

        let shader_program = CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, frag_shader);
        gl::LinkProgram(shader_program);

        GetProgramiv(shader_program, gl::LINK_STATUS, &mut ok);

        if ok != gl::TRUE as gl::types::GLint {
            GetProgramInfoLog(
                shader_program,
                512,
                std::ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "Linking of shader program failed:\n{}",
                std::str::from_utf8(&log).unwrap_or("Failed to retrieve error message from OpenGL")
            );
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(frag_shader);
        shader_program
    };

    Ok(program)
}

pub extern "system" fn gl_debug_output(
    source: gl::types::GLenum, source_type: gl::types::GLenum, id: u32, severity: gl::types::GLenum, _length: gl::types::GLsizei,
    message: *const std::os::raw::c_char, _user_param: *mut std::ffi::c_void,
) {
    if id == 131169 || id == 131185 || id == 131218 || id == 131204 {
        return; // ignore these non-significant error codes
    }

    let message = unsafe { std::ffi::CStr::from_ptr(message).to_str().expect("Failed to cast CStr to String") };

    println!("---------------");
    println!("Debug message ({}): {}", id, message);

    match source {
        gl::DEBUG_SOURCE_API => println!("Source: API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => println!("Source: Window System"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
        gl::DEBUG_SOURCE_THIRD_PARTY => println!("Source: Third Party"),
        gl::DEBUG_SOURCE_APPLICATION => println!("Source: Application"),
        gl::DEBUG_SOURCE_OTHER => println!("Source: Other"),
        _ => {
            println!("Unknown source");
        }
    }

    match source_type {
        gl::DEBUG_TYPE_ERROR => println!("Type: Error"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("Type: Undefined Behaviour"),
        gl::DEBUG_TYPE_PORTABILITY => println!("Type: Portability"),
        gl::DEBUG_TYPE_PERFORMANCE => println!("Type: Performance"),
        gl::DEBUG_TYPE_MARKER => println!("Type: Marker"),
        gl::DEBUG_TYPE_PUSH_GROUP => println!("Type: Push Group"),
        gl::DEBUG_TYPE_POP_GROUP => println!("Type: Pop Group"),
        gl::DEBUG_TYPE_OTHER => println!("Type: Other"),
        _ => {}
    }

    match severity {
        gl::DEBUG_SEVERITY_HIGH => println!("Severity: high"),
        gl::DEBUG_SEVERITY_MEDIUM => println!("Severity: medium"),
        gl::DEBUG_SEVERITY_LOW => println!("Severity: low"),
        gl::DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
        _ => {}
    }
    println!("\n");
}
