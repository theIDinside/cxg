#![feature(core_intrinsics)]
#[rustfmt::skip::macros(debugger_catch)]
extern crate freetype as ft;
extern crate gl;
extern crate glfw;
extern crate libc;
extern crate png;

#[macro_use]
pub mod opengl;
pub mod app;
pub mod datastructure;
pub mod textbuffer;
pub mod ui;

#[macro_use]
pub mod utils;

use self::glfw::Context;
use opengl::glinit;

pub use utils::macros::*;

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
static mut TRAP_HANDLER: fn(i32) = |_| {};

pub fn init_debug_break(f: fn(i32)) {
    unsafe {
        TRAP_HANDLER = f as fn(i32);
    };
}

pub fn foo() {
    init_debug_break(|i: i32| {
        println!("trap handler executing. Signal: {}", i);
    });
    let virtual_address = unsafe { TRAP_HANDLER as usize };
    let ptr = virtual_address as *const ();
    let code: extern "C" fn(i32) = unsafe { std::mem::transmute(ptr) };

    unsafe {
        libc::signal(libc::SIGTRAP, code as _);
    }
}

fn main() -> Main {
    let width = 1024;
    let height = 768;
    let font_path = std::path::Path::new("fonts/SourceCodePro-Bold.ttf");
    assert_eq!(font_path.exists(), true);
    let mut glfw_handle = glfw::init(glfw::FAIL_ON_ERRORS)?;
    foo();

    glfw_handle.window_hint(glfw::WindowHint::ContextVersion(4, 3));
    glfw_handle.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    let (mut window, events) = glfw_handle
        .create_window(width, height, "Testing GLFW on Rust", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW Window");

    window.make_current();
    window.set_char_polling(true);
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_refresh_polling(true); // We need to redraw everything
    glfw_handle.set_swap_interval(glfw::SwapInterval::None);
    gl::load_with(|sym| window.get_proc_address(sym) as *const _);
    unsafe {
        glinit::init_gl();
    };

    let font_program = opengl::shaders::TextShader::new();
    let rectangle_program = opengl::shaders::RectShader::new();

    font_program.bind();
    let char_range = (0..0x00f6u8).map(|x| x as char).collect();
    let font = ui::font::Font::new(font_path, 18, char_range).expect("Failed to create font");
    let fonts = vec![font];

    // let mut text_renderer = opengl::text::TextRenderer::create(font_program.clone(), &fonts[], 64 * 1024 * 100).expect("Failed to create TextRenderer");
    let mut app = app::Application::create(&fonts, font_program, rectangle_program);

    let _last_update = glfw_handle.get_time();
    let mut _frame_counter = 0.0;

    let _updatefps = |last_update: &mut f64, glfw_handle: &mut glfw::Glfw, frame_counter: &mut f64| {
        if *frame_counter > 20000.0 {
            let now_time = glfw_handle.get_time();
            let diff_time = now_time - *last_update;
            *last_update = now_time;
            println!("FPS: {}", *frame_counter / diff_time);
            *frame_counter = 0.0;
        }
        *frame_counter += 1.0;
    };

    while !window.should_close() {
        // updatefps(&mut last_update, &mut glfw_handle, &mut window, &mut frame_counter);
        app.process_events(&mut window, &events);
        app.update_window();
        window.swap_buffers();
        glfw_handle.poll_events();
        // glfw_handle.wait_events_timeout(1.0 / 90.0);
    }

    Ok(())
}
