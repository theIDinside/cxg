extern crate glfw;
extern crate gl;
extern crate freetype as ft;
extern crate png;


#[macro_use]
pub mod opengl;

pub mod datastructure;
pub mod app;
pub mod ui;
pub mod textbuffer;


use opengl::glinit;

use self::glfw::{Context};

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

fn main() -> Main {
    let width = 1024;
    let height = 768;
    let font_path = std::path::Path::new("fonts/SourceCodePro-Semibold.ttf");
    assert_eq!(font_path.exists(), true);
    let mut glfw_handle = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw_handle.window_hint(glfw::WindowHint::ContextVersion(4,3));
    glfw_handle.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    let (mut window, events) = glfw_handle.create_window(width, height, "Testing GLFW on Rust", glfw::WindowMode::Windowed).expect("Failed to create GLFW Window");

    window.make_current();
    window.set_char_polling(true);
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_refresh_polling(true); // We need to redraw everything 

    gl::load_with(|sym| window.get_proc_address(sym) as * const _);
    unsafe {
        glinit::init_gl();
    };
        
    let font_program = opengl::shaders::TextShader::new();
    let rectangle_program = opengl::shaders::RectShader::new();

    font_program.bind();
    let char_range = (0 .. 0x00f6u8).map(|x| x as char).collect();
    let font = ui::font::Font::new(font_path, 20, char_range).expect("Failed to create font");
    let fonts = vec![font];

    // let mut text_renderer = opengl::text::TextRenderer::create(font_program.clone(), &fonts[], 64 * 1024 * 100).expect("Failed to create TextRenderer");

    let mut app = app::Application::create(&fonts, font_program, rectangle_program);
    
    let mut last_update = glfw_handle.get_time();
    while !window.should_close() {
        let now_time = glfw_handle.get_time();
        if now_time - last_update >= 0.005f64 {
            last_update = now_time;
        }
        app.process_events(&mut window, &events);
        app.update_window();
        window.swap_buffers();
        glfw_handle.wait_events_timeout(1.0 / 90.0);
    }

    Ok(())
}
