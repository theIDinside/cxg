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
/// Converts a vec of u32 to Vec<char>, unsafely. If you fuck up the code points, it's on you.
fn convert_vec_of_u32_utf(data: &[u32]) -> Vec<char> {
    unsafe {
        data.iter().map(|&c| std::char::from_u32_unchecked(c) ).collect()
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
    // let char_range = (0..=0x0F028u32).filter_map(|c| std::char::from_u32(c)).collect();
                                                                            //       ___________ these two unicode symbols are the less-than-equal and greater-than-equal ≤ and ≥ symbols
    // let char_range: Vec<char> = (0..=1000u32).filter_map(std::char::from_u32).chain((0x2264..=0x2265).filter_map(std::char::from_u32)).collect();
    let char_range: Vec<char> = (0..=0x0f8u32).filter_map(std::char::from_u32).chain(convert_vec_of_u32_utf(&vec![0x2260, 0x2264, 0x2265])).collect();

    let font = ui::font::Font::new(font_path, 18, char_range).expect("Failed to create font");
    let fonts = vec![font];

    // let mut text_renderer = opengl::text::TextRenderer::create(font_program.clone(), &fonts[], 64 * 1024 * 100).expect("Failed to create TextRenderer");
    let mut app = app::Application::create(&fonts, font_program, rectangle_program);

    let mut last_update = glfw_handle.get_time();
    let mut frame_counter = 0.0;
    let mut once_a_second_update = 10000.0;

    let mut updatefps = move |glfw_handle: &mut glfw::Glfw| -> Option<f64> {
        if frame_counter > once_a_second_update {
            let now_time = glfw_handle.get_time();
            let diff_time = now_time - last_update;
            last_update = now_time;
            let res = frame_counter / diff_time;
            let tmp = once_a_second_update / res;
            once_a_second_update /= tmp;
            frame_counter = 0.0;
            Some(res)
        } else {
            frame_counter += 1.0;
            None
        }
    };

    while app.keep_running() {
        if let Some(fps) = updatefps(&mut glfw_handle) {
            let frame_time = (1.0 / fps) * 1000.0;
            app.debug_view.do_update_view(fps, frame_time);
        }
        app.process_events(&mut window, &events);
        app.update_window();
        window.swap_buffers();
        glfw_handle.poll_events();
        // glfw_handle.wait_events_timeout(1.0 / 90.0);
    }

    Ok(())
}
