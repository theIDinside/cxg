use glfw::{Action, Key, Modifiers, Window};
use std::sync::mpsc::Receiver;

use crate::opengl::rect::RectRenderer;
use crate::opengl::shaders::{RectShader, TextShader};
use crate::opengl::types::RGBAColor;
use crate::textbuffer::{CharBuffer, Movement, TextKind};
use crate::ui::coordinate::{Anchor, Coordinate, Layout, PointArithmetic, Size};
use crate::ui::panel::Panel;
use crate::ui::statusbar::StatusBar;
use crate::ui::view::{Popup, View};
use crate::ui::UID;

use crate::opengl::text::TextRenderer;
use crate::ui::font::Font;

pub struct TextBuffer {
    buf: Vec<char>,
}

#[allow(unused)]
enum ActiveInput {
    TextFile(usize),
    Application,
}

static TEST_DATA: &str = include_str!("./textbuffer/simple/simplebuffer.rs");

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
    active_view: *mut View<'app>,
    active_views: Vec<*mut View<'app>>,
}

impl<'app> Application<'app> {
    pub fn open_text_view(&mut self, parent_panel: u32, view_name: Option<String>, view_size: Size) {
        let view_id = self
            .panels
            .iter()
            .flat_map(|panel| panel.children.iter().map(|v| *v.id))
            .max()
            .unwrap_or(0);
        if let Some(p) = self
            .panels
            .iter_mut()
            .find(|panel| panel.id == parent_panel)
        {
            let font = &self.fonts[0];
            let Size { width, height } = view_size;
            let view_name = view_name
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or("unnamed view");
            let view = View::new(
                view_name,
                view_id.into(),
                TextRenderer::create(self.font_shader.clone(), font, 1024 * 10)
                    .expect("Failed to create TextRenderer"),
                RectRenderer::create(self.rect_shader.clone(), 1024 * 10)
                    .expect("failed to create rectangle renderer"),
                0,
                width,
                height,
                font.row_height(),
            );
            self.active_ui_element = UID::View(*view.id);
            p.add_view(view);
        }
    }

    pub fn create(fonts: &'app Vec<Font>,font_shader: super::opengl::shaders::TextShader,rect_shader: RectShader) -> Application<'app> {
        let mut active_view_id = 0;
        font_shader.bind();
        let mvp = super::opengl::glinit::screen_projection_matrix(1024, 768, 0);
        font_shader.set_projection(&mvp);

        rect_shader.bind();
        rect_shader.set_projection(&mvp);

        let sb_tr = TextRenderer::create(font_shader.clone(), &fonts[0], 1024)
            .expect("Failed to create TextRenderer");
        let mut sb_wr = RectRenderer::create(rect_shader.clone(), 8 * 60)
            .expect("failed to create rectangle renderer");
        sb_wr.set_color(RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        let sb_size = Size::new(1024, fonts[0].row_height() + 4);
        let sb_anchor = Anchor(0, 768);
        let mut status_bar = StatusBar::new(sb_tr, sb_wr, sb_anchor, sb_size);
        status_bar.update();

        let panel = Panel::new(
            0,
            Layout::Horizontal(10.into()),
            Some(15),
            None,
            1024 / 2,
            768 - sb_size.height,
            (0, 768 - sb_size.height).into(),
        );
        let panel2 = Panel::new(
            1,
            Layout::Vertical(10.into()),
            Some(15),
            None,
            1024 / 2,
            768 - sb_size.height,
            (1024 / 2, 768 - sb_size.height).into(),
        );
        let mut panels = vec![panel, panel2];

        let view = View::new(
            "Left view",
            active_view_id.into(),
            TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10)
                .expect("Failed to create TextRenderer"),
            RectRenderer::create(rect_shader.clone(), 8 * 60)
                .expect("failed to create rectangle renderer"),
            0,
            1024,
            768,
            fonts[0].row_height(),
        );
        panels[0].add_view(view);
        active_view_id += 1;

        let mut view = View::new(
            "Right Bottom",
            active_view_id.into(),
            TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10).unwrap(),
            RectRenderer::create(rect_shader.clone(), 8 * 60).unwrap(),
            0,
            1024,
            768,
            fonts[0].row_height(),
        );
        view.insert_str("\n\nabcd\n123");
        // view.insert_str("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16");
        panels[1].add_view(view);

        let mut popup = View::new(
            "Popup view",
            (active_view_id + 1).into(),
            TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10).unwrap(),
            RectRenderer::create(rect_shader.clone(), 8 * 60).unwrap(),
            0,
            524,
            518,
            fonts[0].row_height(),
        );

        popup.set_anchor((250, 768 - 250).into());
        popup.update();
        popup.window_renderer.set_color(RGBAColor {
            r: 0.3,
            g: 0.34,
            b: 0.48,
            a: 0.8,
        });

        let popup = Some(Popup {
            visible: false,
            view: popup,
        });

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
            active_view: std::ptr::null_mut(),
            active_views: vec![],
        }
    }

    pub fn init<'b>(&'b mut self) {
        match self.active_ui_element {
            UID::View(id) => {
                if let Some(v) = self.panels[1].get_view(id.into()) {
                    self.active_view = v;
                }
                self.active_views.push(self.active_view);
                for v in self.active_views.iter() {
                    println!("View: {:?}", unsafe { &(**v) });
                }
            }
            UID::Panel(_id) => todo!(),
        }
    }

    pub fn char_insert(&mut self, ch: char) {
        match self.active_input {
            ActiveInput::TextFile(_) => {
                self.buf.buf.push(ch);
            }
            _ => {}
        }
    }

    fn sync_shader(&mut self) {
        let (width, height) = self.window_size.values();
        let mvp = super::opengl::glinit::screen_projection_matrix(*width as _, *height as _, 0);
        self.font_shader.set_projection(&mvp);
        self.rect_shader.set_projection(&mvp);
    }

    fn handle_resize_event(&mut self, width: i32, height: i32) {
        println!(
            "App window {:?} ===> {}x{}",
            self.window_size, width, height
        );
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

        unsafe { gl::Viewport(0, 0, width, height) }
    }

    // NOTE: not the same version as in common.rs!
    pub fn process_events(&mut self, window: &mut Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    self.handle_resize_event(width, height);
                }
                glfw::WindowEvent::Char(ch) => {
                    if let Some(v) = unsafe { self.active_view.as_mut() } {
                        v.insert_ch(ch);
                    }
                }
                glfw::WindowEvent::Key(key, _, action, m) => {
                    self.handle_key_event(window, key, action, m);
                }
                _ => {}
            }
        }
    }

    pub fn handle_key_event(
        &mut self,
        window: &mut Window,
        key: glfw::Key,
        action: glfw::Action,
        modifier: glfw::Modifiers,
    ) {
        let v = unsafe { self.active_view.as_mut().unwrap() };

        match key {
            Key::Home => match modifier {
                Modifiers::Control => v.cursor_goto(crate::textbuffer::metadata::Index(0)),
                _ => v.move_cursor(Movement::Begin(TextKind::Line)),
            },
            Key::End => match modifier {
                Modifiers::Control => {
                    v.cursor_goto(crate::textbuffer::metadata::Index(v.buffer.len()))
                }
                _ => v.move_cursor(Movement::End(TextKind::Line)),
            },
            Key::Right if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    v.move_cursor(Movement::Forward(TextKind::Word, 1));
                } else {
                    v.move_cursor(Movement::Forward(TextKind::Char, 1));
                }
            }
            Key::Left if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    v.move_cursor(Movement::Backward(TextKind::Word, 1));
                } else {
                    v.move_cursor(Movement::Backward(TextKind::Char, 1));
                }
            }
            Key::Up if action == Action::Repeat || action == Action::Press => {
                v.move_cursor(Movement::Backward(TextKind::Line, 1));
            }
            Key::Down if action == Action::Repeat || action == Action::Press => {
                v.move_cursor(Movement::Forward(TextKind::Line, 1));
            }
            Key::Backspace if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    v.delete(Movement::Backward(TextKind::Word, 1));
                } else if modifier.is_empty() {
                    v.delete(Movement::Backward(TextKind::Char, 1));
                }
            }
            Key::Delete if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    v.delete(Movement::Forward(TextKind::Word, 1));
                } else if modifier.is_empty() {
                    v.delete(Movement::Forward(TextKind::Char, 1));
                }
            }

            Key::F1 => {
                if modifier == Modifiers::Control {
                    // v.insert_slice(&vec[..]);
                    v.insert_str(TEST_DATA);
                } else {
                    self.debug = !self.debug;
                    println!("Opening debug interface...");
                    println!("Application window: {:?}", self.window_size);
                    for p in self.panels.iter() {
                        println!("{:?}", p);
                    }

                    #[cfg(debug_assertions)]
                    {
                        v.buffer.debug_metadata();
                    }

                    v.debug_viewcursor();
                }
            },
            Key::F2 => if modifier == Modifiers::Control {
                    let vec: Vec<char> = TEST_DATA.chars().collect();
                    v.insert_slice(&vec[..]);
            },
            Key::P if modifier == Modifiers::Control => {
                if let Some(p) = self.popup.as_mut() {
                    if p.visible {
                        if let Some(v) = self.active_views.pop() {
                            self.active_view = v;
                        }
                    } else {
                        self.active_views.push(self.active_view);
                        self.active_view = &mut p.view as _;
                    }
                    p.visible = !p.visible;
                }
            }
            Key::Q if modifier == Modifiers::Control => {
                window.set_should_close(true);
            }
            Key::Enter if action == Action::Press || action == Action::Repeat => {
                v.insert_ch('\n');
            }
            _ => {}
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
                v.draw();
            }
        }

        if let Some(v) = self.popup.as_mut() {
            if v.visible {
                v.view.draw();
            }
        }
        self.status_bar.draw();
    }
}
