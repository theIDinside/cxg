use glfw::{Window, Key, Action, Modifiers};
use std::sync::mpsc::Receiver;

use crate::opengl::rect::{RectRenderer};
use crate::opengl::shaders::{RectShader, TextShader};
use crate::opengl::types::RGBAColor;
use crate::ui::UID;
use crate::ui::panel::{Panel};
use crate::ui::coordinate::{Layout, Coordinate, Size, PointArithmetic, Anchor};
use crate::ui::statusbar::StatusBar;
use crate::ui::view::{View, Popup};

use crate::opengl::text::{TextRenderer};
use crate::ui::font::Font;


pub struct TextBuffer {
    buf: Vec<char>
}

#[allow(unused)]
enum ActiveInput {
    TextFile(usize),
    Application
}



pub struct Application<'app> {
    _title_bar: String,
    window_size: Size,
    panel_space_size: Size,
    buf: TextBuffer,
    active_input: ActiveInput,
    fonts: &'app Vec<Font>,
    status_bar: StatusBar<'app>,
    font_shader: TextShader,
    rect_shader: RectShader,
    panels: Vec<Panel<'app>>,
    popup: Option<Popup<'app>>,
    active_ui_element: UID,
    debug: bool,
}

impl<'app> Application<'app> {

    pub fn open_text_view(&mut self, parent_panel: u32, view_name: Option<String>, view_size: Size) {
        let view_id = self.panels.iter().flat_map(|panel| panel.children.iter().map(|v| *v.id)).max().unwrap_or(0);
        if let Some(p) = self.panels.iter_mut().find(|panel| panel.id == parent_panel) {
            let font = &self.fonts[0];
            let Size {width, height} = view_size;
            let view_name = view_name.as_ref().map(|name| name.as_ref()).unwrap_or("unnamed view");
            let view = View::new(view_name, view_id.into(), TextRenderer::create(self.font_shader.clone(), font, 64 * 1024 * 100).expect("Failed to create TextRenderer"), RectRenderer::create(self.rect_shader.clone(), 8 * 60).expect("failed to create rectangle renderer"), 0, width, height, font.row_height());
            self.active_ui_element = UID::View(*view.id);
            p.add_view(view);
        }
    }

    pub fn create(fonts: &'app Vec<Font>, font_shader: super::opengl::shaders::TextShader, rect_shader: RectShader) -> Application<'app> {
        let mut active_view_id = 0;
        font_shader.bind();
        let mvp = super::opengl::glinit::screen_projection_matrix(1024, 768, 0);
        font_shader.set_projection(&mvp);

        rect_shader.bind();
        rect_shader.set_projection(&mvp);

        let sb_tr = TextRenderer::create(font_shader.clone(), &fonts[0], 64 * 1024 * 100).expect("Failed to create TextRenderer");
        let mut sb_wr = RectRenderer::create(rect_shader.clone(), 8 * 60).expect("failed to create rectangle renderer");
        sb_wr.set_color(RGBAColor::new(0.5,0.5,0.5, 1.0));
        let sb_size = Size::new(1024, fonts[0].row_height() + 4);
        let sb_anchor = Anchor(0, 768);
        let mut status_bar = StatusBar::new(sb_tr, sb_wr, sb_anchor, sb_size);
        status_bar.update();

        let panel = Panel::new(0, Layout::Horizontal(10.into()), Some(15), None, 1024 / 2, 768 - sb_size.height, (0, 768 - sb_size.height).into());
        let panel2 = Panel::new(1, Layout::Vertical(10.into()), Some(15), None, 1024 / 2, 768 - sb_size.height, (1024 / 2, 768 - sb_size.height).into());
        let mut panels = vec![panel, panel2];
        
        let view = View::new("Left view", active_view_id.into(), TextRenderer::create(font_shader.clone(), &fonts[0], 64 * 1024 * 100).expect("Failed to create TextRenderer"), RectRenderer::create(rect_shader.clone(), 8 * 60).expect("failed to create rectangle renderer"), 0, 1024, 768, fonts[0].row_height());
        panels[0].add_view(view);
        active_view_id += 1;
        
        let view = View::new("Right Top", active_view_id.into(), TextRenderer::create(font_shader.clone(), &fonts[0], 64 * 1024 * 100).expect("Failed to create TextRenderer"),RectRenderer::create(rect_shader.clone(), 8 * 60).expect("failed to create window renderer"), 0, 1024, 768, fonts[0].row_height());
        panels[1].add_view(view);
        active_view_id += 1;

        let view = View::new("Right Bottom", active_view_id.into(), TextRenderer::create(font_shader.clone(), &fonts[0], 64 * 1024 * 100).expect("Failed to create TextRenderer"),RectRenderer::create(rect_shader.clone(), 8 * 60).expect("failed to create rectangle renderer"), 0, 1024, 768, fonts[0].row_height());
        panels[1].add_view(view);

        let mut popup = View::new("Popup view", (active_view_id + 1).into(), TextRenderer::create(font_shader.clone(), &fonts[0], 64 * 1024 * 100).expect("Failed to create TextRenderer"),RectRenderer::create(rect_shader.clone(), 8 * 60).expect("failed to create rectangle renderer"), 0, 524, 518, fonts[0].row_height());
        
        popup.set_anchor((250, 768-250).into());
        popup.update();
        &popup.window_renderer.set_color(RGBAColor{r: 0.3, g: 0.34, b: 0.48, a: 0.8});

        

        let popup = Some( Popup { visible: false, view: popup });
        
        Application {
            _title_bar: "cxgledit".into(),
            window_size: Size::new(1024, 768),
            panel_space_size: Size::new(1024, 768 - sb_size.height),
            buf: TextBuffer { buf: Vec::new() },
            active_input: ActiveInput::TextFile(0),
            fonts,
            status_bar,
            font_shader,
            rect_shader,
            panels,
            popup, 
            active_ui_element: UID::View(active_view_id),
            debug: false,
        }
    }

    pub fn char_insert(&mut self, ch: char) {
        match self.active_input {
            ActiveInput::TextFile(_) => {
                self.buf.buf.push(ch);
            },
            _ => {}
        }
    }

    fn sync_shader(&mut self) {
        let (width, height) = self.window_size.values();
        let mvp = super::opengl::glinit::screen_projection_matrix(*width as _, *height as _, 0);
        self.font_shader.set_projection(&mvp);
        self.rect_shader.set_projection(&mvp);
    }

    pub fn split_panel(&mut self) {}

    // NOTE: not the same version as in common.rs!
    pub fn process_events(&mut self, window: &mut Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // make sure the viewport matches the new window dimensions; note that width and
                    // height will be significantly larger than specified on retina displays.
                    println!("App window {:?} ===> {}x{}", self.window_size, width, height);
                    let new_panel_space_size = Size::new(width, height - self.status_bar.size.height);
                    let size_change_factor = new_panel_space_size / self.panel_space_size;

                    for p in self.panels.iter_mut() {
                        let old_size = p.size;
                        let new_size = Size::vector_multiply(p.size, size_change_factor);
                        p.resize(new_size.width, new_size.height);
                        let Anchor(x, y) = Anchor::vector_multiply(p.anchor, size_change_factor);
                        p.set_anchor(x, y);
                        p.size_changed(old_size);

                        for v in p.children.iter_mut() {
                            v.update();
                        }
                    }

                    Application::set_dimensions(self, width, height);
                    self.status_bar.size.width = width;
                    self.status_bar.anchor = Anchor(0, height);
                    self.status_bar.update();

                    unsafe {
                        gl::Viewport(0, 0, width, height) 
                    }
                },
                glfw::WindowEvent::Char(ch) => {
                    self.char_insert(ch);
                    println!("char input: {}", ch)
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    println!("Dumping buf contents: {:?}", self.buf.buf);
                    window.set_should_close(true);
                },
                glfw::WindowEvent::Key(Key::F1, _, Action::Press, _) => {
                    self.debug = !self.debug;
                    println!("Opening debug interface...");
                    println!("Application window: {:?}", self.window_size);
                    for p in self.panels.iter() {
                        println!("{:?}", p);
                    }
                },
                glfw::WindowEvent::Key(Key::P, _, Action::Press, Modifiers::Control) => {
                    println!("Open popup panel");
                    if let Some(p) = self.popup.as_mut() {
                        p.visible = !p.visible;
                    }
                }
                glfw::WindowEvent::Key(Key::Q, _, Action::Press, Modifiers::Control) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::Enter, _, Action::Press, _) => {
                    match self.active_input {
                        ActiveInput::Application => println!("Handle execution of commands etc"),
                        ActiveInput::TextFile(_) => self.char_insert('\n'),
                    }
                }
                glfw::WindowEvent::Key(_, _, Action::Press, _) => {
                    // println!("Key input handler - Key: {}  Scancode: {}", k.get_name().unwrap(), k.get_scancode().unwrap());
                },
                _ => {}
            }
        }
    }

    pub fn set_dimensions(&mut self, width: i32, height: i32) {
        self.window_size = Size::new(width, height);
        self.panel_space_size = Size::new(width, height - self.status_bar.size.height);
        self.sync_shader();
    }

    #[inline]
    pub fn width(&self) -> i32 {
        self.window_size.width
    }

    #[inline]
    pub fn height(&self) -> i32 {
        self.window_size.height
    }

    pub fn update_window(&mut self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Viewport(0, 0, self.width() as _, self.height() as _);
        }

        for p in self.panels.iter_mut() {
            for v in p.children.iter_mut() {
                
                
                if self.debug {
                    unsafe {
                        gl::ClearColor(0.4, 0.7, 0.3, 1.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }
                    v.draw();
                    unsafe {
                        gl::Disable(gl::SCISSOR_TEST);
                    }
                } else {
                    v.draw();
                }
            }
        }

        if let Some(v) = self.popup.as_mut() {
            if v.visible {
                v.window_renderer.draw();
                v.view.draw();
            }
        }
        self.status_bar.draw();
    }
}