extern crate glfw;
extern crate gl;
extern crate freetype as ft;
extern crate png;

pub mod app;
pub mod ui;
pub mod glinit;
pub mod opengl;


use self::glfw::{Context, Key, Action};
use std::sync::mpsc::Receiver;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MainInitError {
    GLFW(glfw::InitError),
    Shader(String),
}

impl From<glfw::InitError> for MainInitError {
    fn from(item: glfw::InitError) -> MainInitError {
        MainInitError::GLFW(item)
    }
}

type Main = Result<(), MainInitError>;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;


fn main() -> Main {
    let font_path = std::path::Path::new("fonts/SourceCodePro-Semibold.ttf");
    assert_eq!(font_path.exists(), true);
    println!("Hello, world!");
    let mut glfw_handle = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw_handle.window_hint(glfw::WindowHint::ContextVersion(4,3));
    glfw_handle.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    let (mut window, events) = glfw_handle.create_window(1024, 768, "Testing GLFW on Rust", glfw::WindowMode::Windowed).expect("Failed to create GLFW Window");

    window.make_current();
    window.set_char_polling(true);
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_refresh_polling(true); // We need to redraw everything 

    gl::load_with(|sym| window.get_proc_address(sym) as * const _);
    
    let shader_program = 
    // match glinit::create_shader_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE) {
    match glinit::create_shader_program(opengl::shaders::source::CURSOR_VERTEX_SHADER, opengl::shaders::source::CURSOR_FRAGMENT_SHADER) {
        Ok(program) => program,
        Err(_) => {
            println!("Error creating shader program. Exiting application.");
            std::process::exit(1);
        }
    };

    let (mut vbo, mut vao) = (0, 0);

    let vertices: [f32; 9] = [
        -0.5, -0.5, 0.0, // left
         0.5, -0.5, 0.0, // right
         0.0,  0.5, 0.0  // top
    ];

    let char_range = (0 .. 0x00f6u8).map(|x| x as char).collect();
    let font = ui::font::Font::new(font_path, 24, char_range);

    let mut app = app::Application::create();
    unsafe {
        println!("Window pointer: {:?}", window.window_ptr());
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
            &vertices[0] as *const f32 as *const std::os::raw::c_void,
            gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<gl::types::GLfloat>() as gl::types::GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        gl::UseProgram(shader_program);
    }

    while !window.should_close() {
        app.process_events(&mut window, &events);
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        window.swap_buffers();
        glfw_handle.poll_events();
    }

    Ok(())
}
