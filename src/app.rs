use crate::opengl::shaders;
use crate::textbuffer::{CharBuffer, Movement, TextKind};
use crate::ui::panel::PanelId;
use crate::ui::view::ViewId;
use crate::ui::{ coordinate::{Anchor, Coordinate, Layout, PointArithmetic, Size}, font::Font, panel::Panel, statusbar::StatusBar, view::{Popup, View}, UID };
use crate::{
    datastructure::generic::Vec2i,
    debugger_catch,
    opengl::{rect::RectRenderer, text::TextRenderer, types::RGBAColor},
    DebuggerCatch,
};

use glfw::{ Action, Key, Modifiers, Window };
use std::sync::mpsc::Receiver;

static TEST_DATA: &str = include_str!("./textbuffer/simple/simplebuffer.rs");

static VIEW_BACKGROUND: RGBAColor = RGBAColor {
    r: 0.781,
    g: 0.52,
    b: 0.742123,
    a: 1.0,
};
static ACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor {
    r: 0.51,
    g: 0.59,
    b: 0.13,
    a: 1.0,
};

pub struct Application<'app> {
    /// Window Title
    _title_bar: String,
    /// Window Size
    window_size: Size,
    /// Total space of window, that can be occupied by panels (status bar for instance, is *not* counted among this space)
    panel_space_size: Size,
    /// Loaded fonts. Must be loaded up front, before application is initialized, as the reference must outlive Application<'app>
    fonts: &'app Vec<Font>,
    /// The statusbar, displays different short info about whatever element we're using, or some other user message/debug data
    status_bar: StatusBar<'app>,
    /// The shader for the font
    font_shader: shaders::TextShader,
    /// Shaders for rectangles/windows/views
    rect_shader: shaders::RectShader,
    /// The panels, which hold the different views, and manages their layout and size
    panels: Vec<Panel<'app>>,
    /// The command popup, an input box similar to that of Clion, or VSCode, or Vim's command input line
    popup: Option<Popup<'app>>,
    /// The active element's id
    active_ui_element: UID,
    /// Whether or not we're in debug interface mode (showing different kinds of debug information)
    debug: bool,
    /// Pointer to the element which is receiving Keyboard Input.
    active_view: *mut View<'app>,
    /// The active/displayed views in the window
    active_views: Vec<ViewId>,
    /// We keep running the application until close_requested is true. If true, Application will see if all data and views are in an acceptably quittable state, such as, 
    /// all files are saved to disk (aka pristine) or all files are cached to disk (unsaved, but stored in permanent medium in newest state) etc. If App is not in acceptably quittable state, 
    /// close_requested will be set to false again, so that user can respond to Application asking the user about actions needed to quit.
    close_requested: bool
}

impl<'app> Application<'app> {
    /// Creates a text view and makes that the focused UI element
    pub fn open_text_view(&mut self, parent_panel: PanelId, view_name: Option<String>, view_size: Size) {
        let parent_panel = parent_panel.into();
        let view_id = self
            .panels
            .iter()
            .flat_map(|panel| panel.children.iter().map(|v| *v.id))
            .max()
            .unwrap_or(0)
            + 1;
        if let Some(p) = self.panels.iter_mut().find(|panel| panel.id == parent_panel) {
            let font = &self.fonts[0];
            let Size { width, height } = view_size;
            let view_name = view_name.as_ref().map(|name| name.as_ref()).unwrap_or("unnamed view");
            let view = View::new(
                view_name,
                view_id.into(),
                TextRenderer::create(self.font_shader.clone(), font, 1024 * 10),
                RectRenderer::create(self.rect_shader.clone(), 1024 * 10),
                0,
                width,
                height,
                font.row_height(),
                ACTIVE_VIEW_BACKGROUND,
            );
            self.active_ui_element = UID::View(*view.id);
            p.add_view(view);
            unsafe {
                (*self.active_view).bg_color = VIEW_BACKGROUND;
                (*self.active_view).window_renderer.set_color(VIEW_BACKGROUND);
                (*self.active_view).update();
            }
            self.active_view = p.get_view(view_id.into()).unwrap() as *mut _;
            self.active_views.push(view_id.into());
        } else {
            panic!("panel with id {} was not found", *parent_panel);
        }
    }

    /// Gets the currently active panel, which always is the parent of the View that is currently active
    pub fn active_panel(&self) -> PanelId {
        unsafe {
            (*self.active_view).panel_id.unwrap()
        }
    }

    pub fn keep_running(&self) -> bool {
        !self.close_requested
    }

    pub fn cycle_focus(&mut self) {
        unsafe {
            (*self.active_view).bg_color = VIEW_BACKGROUND;
            (*self.active_view).set_need_redraw();
            (*self.active_view).window_renderer.set_color(VIEW_BACKGROUND);
        }
        let find_pos = |view_id: &ViewId| *view_id == unsafe { (*self.active_view).id };

        if let Some(idx) = self.active_views.iter().position(find_pos) {
            let next_id = self.active_views.get(idx + 1).unwrap_or(self.active_views.first().unwrap());
            if let Some(view) = self
                .panels
                .iter_mut()
                .flat_map(|p| p.children.iter_mut())
                .find(|v| v.id == *next_id) {
                self.active_view = view as *mut _;
            }
        }
        unsafe {
            (*self.active_view).bg_color = ACTIVE_VIEW_BACKGROUND;
            (*self.active_view).window_renderer.set_color(ACTIVE_VIEW_BACKGROUND);
            (*self.active_view).set_need_redraw();
        }
        let id = unsafe { (*self.active_view).id };
        self.active_ui_element = UID::View(*id);
    }

    /// Updates the string contents of the status bar
    pub fn update_status_bar(&mut self, text: String) {
        self.status_bar.update_string_contents(&text);
        self.status_bar.update();
    }

    pub fn create(fonts: &'app Vec<Font>, font_shader: shaders::TextShader, rect_shader: shaders::RectShader) -> Application<'app> {
        let active_view_id = 0;
        font_shader.bind();
        let mvp = super::opengl::glinit::screen_projection_matrix(1024, 768, 0);
        font_shader.set_projection(&mvp);

        rect_shader.bind();
        rect_shader.set_projection(&mvp);

        let sb_tr = TextRenderer::create(font_shader.clone(), &fonts[0], 1024);
        let mut sb_wr = RectRenderer::create(rect_shader.clone(), 8 * 60);
        sb_wr.set_color(RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        let sb_size = Size::new(1024, fonts[0].row_height() + 4);
        let sb_anchor = Anchor(0, 768);
        let mut status_bar = StatusBar::new(sb_tr, sb_wr, sb_anchor, sb_size, RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        status_bar.update();

        let panel = Panel::new(
            0,
            Layout::Horizontal(10.into()),
            Some(15),
            None,
            1024,
            768 - sb_size.height,
            (0, 768 - sb_size.height).into(),
        );
        let mut panels = vec![panel];

        let view = View::new(
            "Unnamed view",
            active_view_id.into(),
            TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10),
            RectRenderer::create(rect_shader.clone(), 8 * 60),
            0,
            1024,
            768,
            fonts[0].row_height(),
            ACTIVE_VIEW_BACKGROUND,
        );
        panels[0].add_view(view);

        let mut popup = View::new(
            "Popup view",
            (active_view_id + 1).into(),
            TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10),
            RectRenderer::create(rect_shader.clone(), 8 * 60),
            0,
            524,
            518,
            fonts[0].row_height(),
            ACTIVE_VIEW_BACKGROUND,
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

        let mut res = Application {
            _title_bar: "cxgledit".into(),
            window_size: Size::new(1024, 768),
            panel_space_size: Size::new(1024, 768 - sb_size.height),
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
            close_requested: false
        };
        res.init();
        res
    }

    pub fn init<'b>(&'b mut self) {
        match self.active_ui_element {
            UID::View(id) => {
                if let Some(v) = self.panels.last_mut().unwrap().get_view(id.into()) {
                    self.active_view = v;
                }
                self.active_views.push(unsafe { self.active_view.as_ref().unwrap().id });
                for v in self.active_views.iter() {
                    println!("View: {:?}", v );
                }
            }
            UID::Panel(_id) => todo!(),
        }
    }

    fn sync_shader_uniform_projection(&mut self) {
        let (width, height) = self.window_size.values();
        let mvp = super::opengl::glinit::screen_projection_matrix(*width as _, *height as _, 0);
        self.font_shader.set_projection(&mvp);
        self.rect_shader.set_projection(&mvp);
    }

    fn handle_resize_event(&mut self, width: i32, height: i32) {
        println!("App window {:?} ===> {}x{}", self.window_size, width, height);
        let new_panel_space_size = Size::new(width, height - self.status_bar.size.height);
        let size_change_factor = new_panel_space_size / self.panel_space_size;

        for p in self.panels.iter_mut() {
            let Anchor(x, y) = Anchor::vector_multiply(p.anchor, size_change_factor);
            let new_size = Size::vector_multiply(p.size, size_change_factor);
            p.set_anchor(x, y);
            p.resize(new_size.width, new_size.height);
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

    pub fn handle_key_event(&mut self, _window: &mut Window, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) {
        let v = unsafe { self.active_view.as_mut().unwrap() };

        match key {
            Key::Home => match modifier {
                Modifiers::Control => v.cursor_goto(crate::textbuffer::metadata::Index(0)),
                _ => v.move_cursor(Movement::Begin(TextKind::Line)),
            },
            Key::End => match modifier {
                Modifiers::Control => v.cursor_goto(crate::textbuffer::metadata::Index(v.buffer.len())),
                _ => v.move_cursor(Movement::End(TextKind::Line)),
            },
            Key::Right if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    // v.move_cursor(Movement::Forward(TextKind::Word, 1));
                    v.move_cursor(Movement::End(TextKind::Word));
                } else {
                    v.move_cursor(Movement::Forward(TextKind::Char, 1));
                }
            }
            Key::Left if action == Action::Repeat || action == Action::Press => {
                if modifier == Modifiers::Control {
                    // v.move_cursor(Movement::Backward(TextKind::Word, 1));
                    v.move_cursor(Movement::Begin(TextKind::Word));
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
                    v.debug_viewcursor();
                }
            }
            Key::F2 => {
                if modifier == Modifiers::Control {
                    let vec: Vec<char> = TEST_DATA.chars().collect();
                    v.insert_slice(&vec[..]);
                }
            }
            Key::P if modifier == Modifiers::Control => {
                if action == Action::Press {
                    if let Some(p) = self.popup.as_mut() {
                        if p.visible {
                            if let Some(_v) = self.active_views.pop() {
                                // self.active_view = v;
                            }
                        } else {
                            // self.active_views.push(self.active_view);
                            // self.active_view = &mut p.view as _;
                        }
                        p.visible = !p.visible;
                    }
                }
            }
            Key::N if modifier == Modifiers::Control && action == Action::Press => {
                let size = self.window_size;
                self.open_text_view(self.active_panel(), Some("new view".into()), size);
            }
            Key::Tab if modifier == Modifiers::Control && action == Action::Press => {
                self.cycle_focus();
            }
            Key::Q if modifier == Modifiers::Control => {
                self.close_requested = true;
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
        self.sync_shader_uniform_projection();
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

        // TODO: when z-indexing will become a thing, sort these first by that said z-index, back to front, before drawing
        for v in self.panels.iter_mut().flat_map(|p| p.children.iter_mut()) {
            v.draw();
        }

        if let Some(v) = self.popup.as_mut() {
            if v.visible {
                v.view.draw();
            }
        }
        self.status_bar.draw();
    }

    pub fn add_view(&mut self, panel_id: PanelId, mut view: View<'app>) {
        debugger_catch!(
            panel_id == self.panels.iter().find(|p| p.id == panel_id).map(|p| p.id).unwrap_or(std::u32::MAX.into()),
            DebuggerCatch::Handle(format!("Could not find panel with id {}", *panel_id))
        );
        if let Some(panel) = self.panels.iter_mut().find(|p| p.id == panel_id) {
            view.set_manager_panel(panel_id);
            if panel.children.is_empty() {
                let adjusted_anchor = panel
                    .margin
                    .and_then(|margin| Some(Anchor::vector_add(panel.anchor, Vec2i::new(margin, -margin))))
                    .unwrap_or(panel.anchor);
                view.resize(Size::shrink_by_margin(panel.size, panel.margin.unwrap_or(0)));
                view.set_anchor(adjusted_anchor);
                panel.children.push(view);
            } else {
                panel.children.push(view);
                let sub_space_count = panel.children.len();
                let margin = panel.margin.unwrap_or(0);
                let child_sizes = panel.size.divide(sub_space_count as _, margin, panel.layout);
                match panel.layout {
                    Layout::Vertical(space) => {
                        let mut anchor_iter = Anchor::vector_add(panel.anchor, Vec2i::new(margin, -margin));
                        for (c, size) in panel.children.iter_mut().zip(child_sizes.into_iter()) {
                            c.resize(size);
                            c.set_anchor(anchor_iter);
                            anchor_iter = Anchor::vector_add(anchor_iter, Vec2i::new(0, -size.height - *space as i32));
                        }
                    }
                    Layout::Horizontal(space) => {
                        let mut anchor_iter = Anchor::vector_add(panel.anchor, Vec2i::new(margin, -margin));
                        for (c, size) in panel.children.iter_mut().zip(child_sizes.into_iter()) {
                            c.resize(size);
                            c.set_anchor(anchor_iter);
                            anchor_iter = Anchor::vector_add(anchor_iter, Vec2i::new(size.width + *space as i32, 0));
                        }
                    }
                }
            }
            for v in panel.children.iter_mut() {
                v.update();
            }
        }
    }
}
