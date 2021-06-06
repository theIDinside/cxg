use super::MainInitError;
use gl::{CreateProgram, ShaderSource, CompileShader, GetProgramiv, GetShaderiv, GetShaderInfoLog, GetProgramInfoLog};
use std::ffi::CString;

pub unsafe fn init_gl() {
    gl::Enable(gl::BLEND);
    gl::Enable(gl::CULL_FACE);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
}


pub fn create_shader_program(vertex_source: &str, frag_source: &str) -> Result<gl::types::GLuint, MainInitError> {
    let program = unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let v_src = CString::new(vertex_source.as_bytes()).unwrap();
        ShaderSource(vertex_shader, 1, &v_src.as_ptr(), std::ptr::null());
        CompileShader(vertex_shader);
    
        let mut ok = gl::FALSE as gl::types::GLint;
        let mut log = Vec::with_capacity(512);
    
        log.set_len(512-1);
        GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut ok);
        if ok != gl::TRUE as gl::types::GLint {
            GetShaderInfoLog(vertex_shader, 512, std::ptr::null_mut(), log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("Compilation of vertex shader failed:\n{}", std::str::from_utf8(&log).unwrap_or("Failed to retrieve error message from OpenGL"));
            return Err(MainInitError::Shader(String::from_utf8(log).unwrap()));
        }
        log.clear();

        let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let f_src = CString::new(frag_source.as_bytes()).unwrap();
        ShaderSource(frag_shader, 1, &f_src.as_ptr(), std::ptr::null());
        CompileShader(frag_shader);
    
        GetShaderiv(frag_shader, gl::COMPILE_STATUS, &mut ok);
        if ok != gl::TRUE as gl::types::GLint {
            GetShaderInfoLog(frag_shader, 512, std::ptr::null_mut(), log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("Compilation of fragment shader failed:\n{}", std::str::from_utf8(&log).unwrap_or("Failed to retrieve error message from OpenGL"));
            return Err(MainInitError::Shader(String::from_utf8(log).unwrap()));
        }
        log.clear();

        let shader_program = CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, frag_shader);
        gl::LinkProgram(shader_program);

        GetProgramiv(shader_program, gl::LINK_STATUS, &mut ok);

        if ok != gl::TRUE as gl::types::GLint {
            GetProgramInfoLog(shader_program, 512, std::ptr::null_mut(), log.as_mut_ptr() as *mut gl::types::GLchar);
            println!("Linking of shader program failed:\n{}", std::str::from_utf8(&log).unwrap_or("Failed to retrieve error message from OpenGL"));

        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(frag_shader);
        shader_program
    };

    Ok(program)
}