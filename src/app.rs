use crate::datastructure::generic::{Vec2, Vec2d, Vec2i};
use crate::debugger_catch;
use crate::debuginfo::DebugInfo;
use crate::opengl::{rect::RectRenderer, shaders, text::TextRenderer, types::RGBAColor};
use crate::textbuffer::{buffers::Buffers, CharBuffer};
use crate::ui::basic::{
    coordinate::{Coordinate, Layout, PointArithmetic, Size},
    frame::Frame,
};
use crate::ui::debug_view::DebugView;
use crate::ui::eventhandling::event::{InputBehavior, InvalidInputElement};
use crate::ui::inputbox::{InputBox, InputBoxMode};
use crate::ui::panel::{Panel, PanelId};
use crate::ui::statusbar::{StatusBar, StatusBarContent};
use crate::ui::view::{Popup, View, ViewId};
use crate::ui::{font::Font, UID};
use crate::ui::{MouseState, Viewable};

use glfw::{Action, Key, Modifiers, MouseButton, Window};
use std::sync::mpsc::Receiver;

pub static TEST_DATA: &str = include_str!("./textbuffer/simple/simplebuffer.rs");

static INACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.021, g: 0.62, b: 0.742123, a: 1.0 };
static ACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.071, g: 0.202, b: 0.3242123, a: 1.0 };

pub struct Application<'app> {
    /// Window Title
    _title_bar: String,
    /// Window Size
    window_size: Size,
    /// Total space of window, that can be occupied by panels (status bar for instance, is *not* counted among this space)
    panel_space_size: Size,
    /// Loaded fonts. Must be loaded up front, before application is initialized, as the reference must outlive Application<'app>
    fonts: &'app Vec<Box<Font>>,
    /// The statusbar, displays different short info about whatever element we're using, or some other user message/debug data
    status_bar: StatusBar<'app>,
    /// The shader for the font
    font_shader: shaders::TextShader,
    /// Shaders for rectangles/windows/views
    rect_shader: shaders::RectShader,
    /// The panels, which hold the different views, and manages their layout and size
    pub panels: Vec<Panel<'app>>,

    /// buffers we're editing and is live, yet not open in any view currently
    buffers: Buffers,

    /// The command popup, an input box similar to that of Clion, or VSCode, or Vim's command input line
    popup: Popup<'app>,
    /// The active element's id
    pub active_ui_element: UID,
    /// Whether or not we're in debug interface mode (showing different kinds of debug information)
    debug: bool,
    /// Pointer to the text editor view that is currently active
    pub active_view: *mut View<'app>,
    /// Pointer to the element that's currently receiving user input. This handle, handles the behavior of the application
    /// and dispatches accordingly to the right type, to determine what should be done when user inputs, key strokes or mouse movements, etc
    pub active_input: &'app mut dyn InputBehavior,
    /// We keep running the application until close_requested is true. If true, Application will see if all data and views are in an acceptably quittable state, such as,
    /// all files are saved to disk (aka pristine) or all files are cached to disk (unsaved, but stored in permanent medium in newest state) etc. If App is not in acceptably quittable state,
    /// close_requested will be set to false again, so that user can respond to Application asking the user about actions needed to quit.
    close_requested: bool,
    input_box: InputBox<'app>,
    pub debug_view: DebugView<'app>,
    mouse_state: MouseState,
    rect_animation_renderer: RectRenderer,
}

static mut INVALID_INPUT: InvalidInputElement = InvalidInputElement {};

impl<'app> Application<'app> {
    pub fn create(fonts: &'app Vec<Box<Font>>, font_shader: shaders::TextShader, rect_shader: shaders::RectShader, debug_info: DebugInfo) -> Application<'app> {
        let active_view_id = 0;
        font_shader.bind();
        let mvp = super::opengl::glinit::screen_projection_matrix(1024, 768, 0);
        font_shader.set_projection(&mvp);
        // utility renderer creation
        rect_shader.bind();
        rect_shader.set_projection(&mvp);

        let make_view_renderers = || {
            (
                TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10),
                TextRenderer::create(font_shader.clone(), &fonts[1], 1024 * 10),
                RectRenderer::create(rect_shader.clone(), 8 * 60),
            )
        };

        let make_renderers = || (TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10), RectRenderer::create(rect_shader.clone(), 8 * 60));

        // Create the status bar UI element
        let (sb_tr, mut sb_wr) = make_renderers();
        sb_wr.set_color(RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        let sb_size = Size::new(1024, fonts[0].row_height() + 4);
        let sb_anchor = Vec2i::new(0, 768);
        let mut status_bar = StatusBar::new(sb_tr, sb_wr, sb_anchor, sb_size, RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        status_bar.update();

        let mut buffers = Buffers::new();

        // Create default 1st panel to hold views in
        let panel = Panel::new(0, Layout::Horizontal(0.into()), None, None, 1024, 768 - sb_size.height, Vec2i::new(0i32, 768i32 - sb_size.height));
        let mut panels = vec![panel];

        // Create the default 1st view
        let (tr, mtr, rr) = make_view_renderers();
        let buffer = buffers.request_new_buffer();
        let view = View::new("Unnamed view", active_view_id.into(), tr, mtr, rr, 1024, 768, ACTIVE_VIEW_BACKGROUND, buffer);
        panels[0].add_view(view);

        // Create the popup UI
        let (tr, mtr, rr) = make_view_renderers();
        let mut popup = View::new("Popup view", (active_view_id + 1).into(), tr, mtr, rr, 524, 518, ACTIVE_VIEW_BACKGROUND, Buffers::free_buffer());

        popup.set_anchor(Vec2i::new(250, 768 - 250));
        popup.update();
        popup.window_renderer.set_color(RGBAColor { r: 0.3, g: 0.34, b: 0.48, a: 0.8 });
        let popup = Popup { visible: false, view: popup };

        // Creating the Debug View UI
        let (tr, mtr, rr) = make_view_renderers();
        let dbg_view_bg_color = RGBAColor { r: 0.35, g: 0.7, b: 1.0, a: 0.95 };
        let mut debug_view = View::new("debug_view", 10.into(), tr, mtr, rr, 1014, 758, dbg_view_bg_color, Buffers::free_buffer());
        debug_view.set_anchor(Vec2i::new(5, 763));
        debug_view.update();
        debug_view.window_renderer.set_color(RGBAColor { r: 0.35, g: 0.7, b: 1.0, a: 0.95 });
        let debug_view = DebugView::new(debug_view, debug_info);

        let ib_frame = Frame { anchor: Vec2i::new(250, 700), size: Size { width: 500, height: 500 } };

        let input_box = InputBox::new(ib_frame, &fonts[1], &font_shader, &rect_shader);
        let rect_animation_renderer = RectRenderer::create(rect_shader.clone(), 8 * 60);
        let mut res = Application {
            _title_bar: "cxgledit".into(),
            window_size: Size::new(1024, 768),
            panel_space_size: Size::new(1024, 768 - sb_size.height),
            fonts,
            status_bar,
            font_shader,
            rect_shader,
            panels,
            buffers,
            popup,
            active_ui_element: UID::View(active_view_id),
            debug: false,
            active_view: std::ptr::null_mut(),
            active_input: unsafe { &mut INVALID_INPUT as &mut dyn InputBehavior },
            close_requested: false,
            input_box,
            debug_view,
            mouse_state: MouseState::None,
            rect_animation_renderer,
        };
        let v = res.panels.last_mut().and_then(|p| p.children.last_mut()).unwrap() as *mut _;
        res.active_input = unsafe { &mut (*v) as &'app mut dyn InputBehavior };

        match res.active_ui_element {
            UID::View(id) => {
                if let Some(v) = res.panels.last_mut().unwrap().get_view(id.into()) {
                    res.active_view = v;
                }
            }
            UID::Panel(_id) => todo!(),
            UID::None => todo!(),
        }
        // res.init();
        res
    }

    pub fn decorate_active_view(&mut self) {
        let view = unsafe { self.active_view.as_mut().unwrap() };
        view.bg_color = ACTIVE_VIEW_BACKGROUND;
        view.window_renderer.set_color(ACTIVE_VIEW_BACKGROUND);
        view.update();
    }

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
            let menu_font = &self.fonts[1];
            let Size { width, height } = view_size;
            let view_name = view_name.as_ref().map(|name| name.as_ref()).unwrap_or("unnamed view");
            let view = View::new(
                view_name,
                view_id.into(),
                TextRenderer::create(self.font_shader.clone(), font, 1024 * 10),
                TextRenderer::create(self.font_shader.clone(), menu_font, 1024 * 10),
                RectRenderer::create(self.rect_shader.clone(), 1024 * 10),
                width,
                height,
                ACTIVE_VIEW_BACKGROUND,
                self.buffers.request_new_buffer(),
            );

            self.active_ui_element = UID::View(*view.id);
            p.add_view(view);
            unsafe {
                (*self.active_view).bg_color = INACTIVE_VIEW_BACKGROUND;
                // (*self.active_view).window_renderer.set_color(INACTIVE_VIEW_BACKGROUND);
                (*self.active_view).update();
            }
            self.active_view = p.get_view(view_id.into()).unwrap() as *mut _;
            self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn InputBehavior };
        } else {
            panic!("panel with id {} was not found", *parent_panel);
        }
    }

    /// Gets the currently active panel, which always is the parent of the View that is currently active
    pub fn active_panel(&self) -> PanelId {
        unsafe { (*self.active_view).panel_id.unwrap() }
    }

    pub fn keep_running(&self) -> bool {
        !self.close_requested
    }

    pub fn cycle_focus(&mut self) {
        if self.panels.iter().map(|p| p.children.len()).sum::<usize>() < 2 {
            return;
        }
        let id = {
            let view = self.get_active_view();
            view.bg_color = INACTIVE_VIEW_BACKGROUND;
            view.window_renderer.set_color(INACTIVE_VIEW_BACKGROUND);
            view.update();
            view.id
        };

        let mut iter = self.panels.iter().flat_map(|p| p.children.iter()).filter(|v| v.visible).cycle();
        let mut next = false;
        while let Some(it) = iter.next() {
            if next {
                self.active_view = it as *const _ as *mut _;
                break;
            }
            if (*it).id == id {
                next = true;
            }
        }

        let id = unsafe { (*self.active_view).id };
        self.active_ui_element = UID::View(*id);
        self.decorate_active_view();
        self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn InputBehavior };
        let v = unsafe { self.active_view.as_mut().unwrap() };
        if !v.visible {
            v.visible = true;
        }
    }

    pub fn get_active_view(&mut self) -> &mut View<'app> {
        if self.popup.visible {
            return unsafe { &mut *(&self.popup.view as *const _ as *mut _) };
        } else {
            unsafe { &mut *self.active_view }
        }
    }

    pub fn get_active_view_ptr(&mut self) -> *mut View<'app> {
        if self.popup.visible {
            &mut self.popup.view as *mut _
        } else {
            self.active_view
        }
    }

    pub fn set_active_view(&mut self, view: &View<'app>) {
        self.active_view = view as *const _ as *mut _;
    }

    /// Updates the string contents of the status bar
    pub fn update_status_bar(&mut self, text: String) {
        self.status_bar.update_string_contents(&text);
        self.status_bar.update();
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
            let v = Vec2d::new(p.anchor.x as _, p.anchor.y as _);
            let Vec2 { x, y } = v * size_change_factor;
            let new_size = Size::vector_multiply(p.size, size_change_factor);
            p.set_anchor(Vec2i::new(x.round() as _, y.round() as _));
            p.resize(new_size);
            p.layout();
        }

        Application::set_dimensions(self, width, height);
        self.status_bar.size.width = width;
        self.status_bar.anchor = Vec2i::new(0, height);
        self.status_bar.update();

        self.debug_view.view.set_anchor(Vec2i::new(10, self.height() - 10));
        self.debug_view
            .view
            .resize(Size { width: self.width() - 20, height: self.height() - 20 });
        self.debug_view.update();

        let ib_center = self.input_box.frame.size.width / 2;
        let app_window_width_center = width / 2;
        self.input_box
            .set_anchor(Vec2i::new(app_window_width_center - ib_center, height - 25));

        unsafe { gl::Viewport(0, 0, width, height) }
    }

    pub fn process_events(&mut self, window: &mut Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    self.handle_resize_event(width, height);
                }
                glfw::WindowEvent::Char(ch) => {
                    self.active_input.handle_char(ch);
                    // let v = self.get_active_view();
                    // v.insert_ch(ch);
                }
                glfw::WindowEvent::Key(key, _, action, m) => {
                    self.handle_key_event(window, key, action, m);
                }
                glfw::WindowEvent::MouseButton(mbtn, act, _mods) => {
                    let (x, y) = window.get_cursor_pos();
                    let pos = self.translate_screen_to_application_space(Vec2d::new(x, y));

                    if act == glfw::Action::Press {
                        let new_state = MouseState::Click(mbtn, pos);
                        self.handle_mouse_input(new_state);
                        // self.mouse_state = new_state;
                        // click only performs focus, whatever action that needs to be taken, happens at release
                        // this is, so that we can click on something, and drag it
                    } else {
                        self.handle_mouse_input(MouseState::Released(mbtn, pos));
                        // self.mouse_state = MouseState::None;
                    }
                }
                glfw::WindowEvent::CursorPos(mposx, mposy) => {
                    match self.mouse_state {
                        MouseState::Clicked(view, btn, _pos) => {
                            // Start drag, REMEMBER, MUST translate to Application coordinate space
                            self.mouse_state = MouseState::Drag(view, btn, Vec2d::new(mposx, mposy))
                        }
                        MouseState::Drag(view, btn, _) => {
                            // Continue drag, REMEMBER, MUST translate to Application coordinate space
                            self.mouse_state = MouseState::Drag(view, btn, Vec2d::new(mposx, mposy))
                        }
                        _ => { // Do nothing
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_input(&mut self, new_state: MouseState) {
        match new_state {
            MouseState::Click(btn, p) => {
                if btn == glfw::MouseButton::Button1 {
                    let active_id = self.get_active_view_id();
                    let pos = p.to_i32();
                    let clicked_view = self
                        .panels
                        .iter_mut()
                        .flat_map(|p| p.children.iter_mut())
                        .find(|v| v.bounding_box().box_hit_check(pos));
                    let id = clicked_view.as_ref().map(|v| v.id.clone());
                    let de_activate_old = if let Some(view_clicked) = clicked_view {
                        let id = view_clicked.id;
                        view_clicked.mouse_clicked(pos);
                        self.active_view = &mut (*view_clicked) as *mut _;
                        self.active_input = cast_ptr_to_input(self.active_view); // unsafe { self.active_view.as_mut().unwrap() as &'app mut dyn Input };
                        self.decorate_active_view();
                        // check if the clicked view, was the already active view
                        id != active_id
                    } else {
                        false
                    };
                    // if the clicked view, was not the active view already, decorate the old view => inactive
                    if de_activate_old {
                        if let Some(v) = self
                            .panels
                            .iter_mut()
                            .flat_map(|p| p.children.iter_mut())
                            .find(|v| v.id == active_id)
                        {
                            // decorate view as an inactive one
                            v.bg_color = INACTIVE_VIEW_BACKGROUND;
                            v.set_need_redraw();
                            v.window_renderer.set_color(INACTIVE_VIEW_BACKGROUND);
                            v.update();
                        }
                    }
                    self.mouse_state = MouseState::Clicked(id, MouseButton::Button1, p);
                }
            }
            MouseState::Clicked(_view_id, _btn, _pos) => {}
            MouseState::Drag(_maybe_view, _btn, _pos) => {}
            MouseState::Released(_btn, pos) => {
                match self.mouse_state {
                    MouseState::Drag(dragged_view_id, _, _) => {
                        let view_dropped_on = self
                            .panels
                            .iter_mut()
                            .flat_map(|p| p.children.iter_mut())
                            .find(|v| v.bounding_box().box_hit_check(pos.to_i32()))
                            .map(|v| v.id);
                        if let Some(true) = dragged_view_id.zip(view_dropped_on).map(|(a, b)| a != b) {
                            let p_a = self
                                .panels
                                .iter_mut()
                                .position(|p| p.children.iter().any(|f| f.id == dragged_view_id.unwrap()));
                            let mut panel_a = self.panels.swap_remove(p_a.unwrap());
                            let va = panel_a.children.iter().position(|v| v.id == dragged_view_id.unwrap());

                            let coexist = panel_a.children.iter().any(|v| v.id == view_dropped_on.unwrap());
                            if coexist {
                                let vb = panel_a.children.iter().position(|v| v.id == view_dropped_on.unwrap());
                                panel_a.children.swap(va.unwrap(), vb.unwrap());
                                panel_a.layout();
                                for v in panel_a.children.iter_mut() {
                                    if v.id == dragged_view_id.unwrap() {
                                        v.bg_color = ACTIVE_VIEW_BACKGROUND;
                                        v.window_renderer.set_color(ACTIVE_VIEW_BACKGROUND);
                                        v.update();
                                        self.active_view = v as *mut _;
                                        self.active_input = cast_ptr_to_input(self.active_view);
                                    } else {
                                        v.bg_color = INACTIVE_VIEW_BACKGROUND;
                                        v.window_renderer.set_color(INACTIVE_VIEW_BACKGROUND);
                                        v.update();
                                    }
                                }
                                self.panels.insert(p_a.unwrap(), panel_a);
                            } else {
                                let p_b = self
                                    .panels
                                    .iter_mut()
                                    .position(|p| p.children.iter().any(|f| f.id == view_dropped_on.unwrap()));
                                let mut panel_b = self.panels.swap_remove(p_b.unwrap());

                                let vb = panel_b.children.iter().position(|v| v.id == dragged_view_id.unwrap());
                                std::mem::swap(panel_a.children.get_mut(va.unwrap()).unwrap(), panel_b.children.get_mut(vb.unwrap()).unwrap());
                                self.panels.insert(p_a.unwrap(), panel_a);
                                self.panels.insert(p_b.unwrap(), panel_b);
                            }
                        }
                    }
                    _ => {
                        self.mouse_state = MouseState::None;
                    }
                }
                self.mouse_state = MouseState::None;
            }
            MouseState::None => {}
        }
    }

    fn get_active_view_id(&self) -> ViewId {
        unsafe { self.active_view.as_ref().unwrap().id }
    }

    pub fn handle_key_event(&mut self, _window: &mut Window, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) {
        match key {
            Key::KpAdd => {}
            Key::W if modifier == Modifiers::Control && action == Action::Press => {
                self.close_active_view();
            }
            Key::H if modifier == Modifiers::Control && action == Action::Press => {
                let v_ptr = unsafe { &mut (*self.active_view) };
                self.cycle_focus();
                v_ptr.visible = false;
                for p in self.panels.iter_mut() {
                    p.layout();
                }
            }
            Key::S if modifier == Modifiers::Control && action == Action::Press => {
                self.panels
                    .iter_mut()
                    .flat_map(|p| p.children.iter_mut())
                    .for_each(|v| v.visible = true);
                for p in self.panels.iter_mut() {
                    p.layout();
                }
            }
            Key::P if modifier == Modifiers::Control => {
                if action == Action::Press {
                    self.popup.visible = !self.popup.visible;
                }
            }
            Key::I if action == Action::Press => {
                if modifier == Modifiers::Control {
                    if self.input_box.visible {
                        self.active_input = cast_ptr_to_input(self.active_view);
                        self.input_box.visible = false;
                    } else {
                        self.input_box.mode = InputBoxMode::Command;
                        // self.active_input = &mut self.input_box as &'app mut dyn Input;
                        self.active_input = unsafe { &mut *(&mut self.input_box as *mut _) as &'app mut dyn InputBehavior };
                        self.input_box.visible = true;
                    }
                } else if modifier == (Modifiers::Control | Modifiers::Shift) {
                    if self.input_box.visible {
                        self.active_input = cast_ptr_to_input(self.active_view);
                        self.input_box.visible = false;
                    } else {
                        self.input_box.mode = InputBoxMode::FileList;
                        self.active_input = unsafe { &mut *(&mut self.input_box as *mut _) as &'app mut dyn InputBehavior };
                        self.input_box.visible = true;
                    }
                }
            }
            Key::Tab if action == Action::Press => {
                if modifier == Modifiers::Control {
                    self.cycle_focus();
                } else {
                    self.active_input.handle_char(' ');
                    self.active_input.handle_char(' ');
                    self.active_input.handle_char(' ');
                    self.active_input.handle_char(' ');
                }
            }
            Key::Q if modifier == Modifiers::Control => {
                self.close_requested = true;
            }
            Key::F1 => {
                if action == Action::Press {
                    if modifier == Modifiers::Shift {
                        self.active_input.handle_key(key, action, modifier);
                    } else {
                        self.set_debug(!self.debug);
                        // self.debug = !self.debug;
                        println!("Opening debug interface...");
                        println!("Application window: {:?}", &self.window_size);
                        for p in self.panels.iter() {
                            println!("{:?}", p);
                            for c in p.children.iter() {
                                c.debug_viewcursor();
                            }
                        }
                    }
                }
            }
            Key::D if modifier == Modifiers::Control && action == Action::Press => {
                self.debug_view.visibile = !self.debug_view.visibile;
            }
            Key::N if modifier == Modifiers::Control && action == Action::Press => {
                let size = self.window_size;
                self.open_text_view(self.active_panel(), Some("new view".into()), size);
            }
            _ => match self.active_input.handle_key(key, action, modifier) {
                crate::ui::eventhandling::event::InputResponse::File(path) => {
                    let v = self.get_active_view();
                    if v.buffer.empty() {
                        v.buffer.load_file(&path);
                        v.set_need_redraw();
                        v.update();
                        self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn InputBehavior };
                        self.input_box.visible = false;
                    } else {
                        let p_id = self.get_active_view().panel_id;
                        let f_name = path.file_name();
                        self.open_text_view(p_id.unwrap(), f_name.and_then(|s| s.to_str()).map(|f| f.to_string()), self.window_size);
                        let v = self.get_active_view();
                        debugger_catch!(&path.exists(), crate::DebuggerCatch::Handle("File was not found!".into()));
                        v.buffer.load_file(&path);
                        v.set_need_redraw();
                        v.update();
                        self.input_box.visible = false;
                    }
                    self.input_box.clear();
                }
                _ => {}
            },
        }
    }

    fn translate_screen_to_application_space(&self, glfw_coordinate: Vec2d) -> Vec2d {
        let Vec2d { x, y } = glfw_coordinate;
        Vec2d::new(x, self.height() as f64 - y)
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

        if self.popup.visible {
            self.popup.view.draw();
        }

        let v = unsafe { &mut (*self.active_view) };
        self.status_bar
            .update_text_content(StatusBarContent::FileEdit(v.buffer.meta_data().file_name.as_ref(), (v.buffer.cursor_row(), v.buffer.cursor_col())));

        self.status_bar.draw();
        self.input_box.draw();
        // always draw the debug interface last, as it should overlay everything
        self.debug_view.draw();

        if let MouseState::Drag(.., pos) = self.mouse_state {
            let pos = self.translate_screen_to_application_space(pos).to_i32();
            let v = unsafe { self.active_view.as_mut().unwrap() };
            let mut bb = v.bounding_box();

            bb.center_align_around(pos);

            self.rect_animation_renderer
                .set_rect(bb, RGBAColor { r: 0.75, g: 0.75, b: 0.75, a: 0.25 });
            self.rect_animation_renderer.draw();
        } else {
            self.rect_animation_renderer.clear_data();
        }
    }

    pub fn close_active_view(&mut self) {
        // we never detroy the popup window until the application is exited. So hitting "ctrl+w" or whatever keybinding we might have,
        // is just going to cancel the popup and hide it again
        if self.popup.visible {
            self.popup.visible = false;
            self.popup.reset();
            return;
        }
        // todo: we need to ask user,  what to do with unsaved files etc.
        let view = unsafe { self.active_view.as_mut().unwrap() };

        let view_id = view.id;
        let panel_id = view.panel_id.unwrap();

        if self.panels.last().unwrap().children.len() == 1 {
            self.open_text_view(panel_id, None, self.window_size);
        }

        let panel = self.panels.get_mut(*panel_id as usize).unwrap();

        let v = panel.remove_view(view_id);
        drop(v);
        self.active_view = panel.children.last_mut().unwrap() as _;
        self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn InputBehavior };
        panel.layout();
        self.decorate_active_view();
        self.active_ui_element = UID::View(*self.get_active_view().id);
    }

    pub fn set_debug(&mut self, set: bool) {
        self.debug = set;
    }
}

pub fn cast_ptr_to_input<'app, T: InputBehavior>(t: *mut T) -> &'app mut dyn InputBehavior
where
    T: 'app,
{
    unsafe { &mut (*t) as &'app mut dyn InputBehavior }
}
