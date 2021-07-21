use crate::opengl::shaders;
use crate::opengl::{rect::RectRenderer, text::TextRenderer, types::RGBAColor};
use crate::ui::debug_view::DebugView;
use crate::ui::eventhandling::event::{Input, InvalidInput};
use crate::ui::frame::Frame;
use crate::ui::inputbox::{InputBox, InputBoxMode};
use crate::ui::panel::PanelId;
use crate::ui::{
    coordinate::{Anchor, Coordinate, Layout, PointArithmetic, Size},
    font::Font,
    panel::Panel,
    statusbar::StatusBar,
    view::{Popup, View},
    UID,
};


use glfw::{Action, Key, Modifiers, Window};
use std::sync::mpsc::Receiver;

pub static TEST_DATA: &str = include_str!("./textbuffer/simple/simplebuffer.rs");

static VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.021, g: 0.52, b: 0.742123, a: 1.0 };
static ACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.071, g: 0.102, b: 0.1242123, a: 1.0 };

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
    pub panels: Vec<Panel<'app>>,
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
    pub active_input: &'app mut dyn Input,
    /// We keep running the application until close_requested is true. If true, Application will see if all data and views are in an acceptably quittable state, such as,
    /// all files are saved to disk (aka pristine) or all files are cached to disk (unsaved, but stored in permanent medium in newest state) etc. If App is not in acceptably quittable state,
    /// close_requested will be set to false again, so that user can respond to Application asking the user about actions needed to quit.
    close_requested: bool,

    input_box: InputBox<'app>,
    pub debug_view: DebugView<'app>,
}

static mut INVALID_INPUT: InvalidInput = InvalidInput {};

impl<'app> Application<'app> {
    pub fn create(fonts: &'app Vec<Font>, font_shader: shaders::TextShader, rect_shader: shaders::RectShader) -> Application<'app> {
        let active_view_id = 0;
        font_shader.bind();
        let mvp = super::opengl::glinit::screen_projection_matrix(1024, 768, 0);
        font_shader.set_projection(&mvp);
        // utility renderer creation
        rect_shader.bind();
        rect_shader.set_projection(&mvp);

        let make_renderers = || (TextRenderer::create(font_shader.clone(), &fonts[0], 1024 * 10), RectRenderer::create(rect_shader.clone(), 8 * 60));

        // Create the status bar UI element
        let (sb_tr, mut sb_wr) = make_renderers();
        sb_wr.set_color(RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        let sb_size = Size::new(1024, fonts[0].row_height() + 4);
        let sb_anchor = Anchor(0, 768);
        let mut status_bar = StatusBar::new(sb_tr, sb_wr, sb_anchor, sb_size, RGBAColor::new(0.5, 0.5, 0.5, 1.0));
        status_bar.update();

        // Create default 1st panel to hold views in
        let panel = Panel::new(0, Layout::Horizontal(10.into()), Some(15), None, 1024, 768 - sb_size.height, (0, 768 - sb_size.height).into());
        let mut panels = vec![panel];

        // Create the default 1st view
        let (tr, rr) = make_renderers();
        let view = View::new("Unnamed view", active_view_id.into(), tr, rr, 0, 1024, 768, ACTIVE_VIEW_BACKGROUND);
        panels[0].add_view(view);

        // Create the popup UI
        let (tr, rr) = make_renderers();
        let mut popup = View::new("Popup view", (active_view_id + 1).into(), tr, rr, 0, 524, 518, ACTIVE_VIEW_BACKGROUND);
        popup.set_anchor((250, 768 - 250).into());
        popup.update();
        popup.window_renderer.set_color(RGBAColor { r: 0.3, g: 0.34, b: 0.48, a: 0.8 });
        let popup = Popup { visible: false, view: popup };

        // Creating the Debug View UI
        let (tr, rr) = make_renderers();
        let dbg_view_bg_color = RGBAColor { r: 0.35, g: 0.7, b: 1.0, a: 0.95 };
        let mut debug_view = View::new("debug_view", 10.into(), tr, rr, 0, 1014, 758, dbg_view_bg_color);
        debug_view.set_anchor(Anchor(5, 763));
        debug_view.update();
        debug_view.window_renderer.set_color(RGBAColor { r: 0.35, g: 0.7, b: 1.0, a: 0.95 });
        let debug_view = DebugView::new(debug_view);


        let ib_frame = Frame {
            anchor: Anchor(250, 700),
            size: Size { width: 500, height: 650 }
        };

        let input_box = InputBox::new(ib_frame, &fonts[0], &font_shader, &rect_shader);

        let mut res = 
        Application {
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
            active_input: unsafe { &mut INVALID_INPUT as &mut dyn Input },
            close_requested: false,
            input_box,
            debug_view,
        };
        let v = res.panels.last_mut().and_then(|p| p.children.last_mut()).unwrap() as *mut _;
        res.active_input = unsafe { &mut (*v) as &'app mut dyn Input };
        
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
            self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn Input };
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
            view.bg_color = VIEW_BACKGROUND;
            view.set_need_redraw();
            view.window_renderer.set_color(VIEW_BACKGROUND);
            view.id
        };

        let mut iter = self.panels.iter().flat_map(|p| p.children.iter()).cycle();
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
        self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn Input };
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

        self.debug_view.view.set_anchor(Anchor(10, self.height() - 10));
        self.debug_view.view.size = Size { width: self.width() - 20, height: self.height() - 20 };
        self.debug_view.update();

        let ib_center = self.input_box.frame.size.width / 2;
        let app_window_width_center = width / 2;
        self.input_box.set_anchor(Anchor(app_window_width_center - ib_center, height - 25));

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
                _ => {}
            }
        }
    }

    pub fn handle_key_event(&mut self, _window: &mut Window, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) {
        match key {
            Key::W if modifier == Modifiers::Control && action == Action::Press => {
                self.close_active_view();
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
                        self.active_input = unsafe { &mut *(&mut self.input_box as *mut _) as &'app mut dyn Input };
                        self.input_box.visible = true;
                    }
                } else if modifier == (Modifiers::Control | Modifiers::Shift) {
                    if self.input_box.visible {
                        self.active_input = cast_ptr_to_input(self.active_view);
                        self.input_box.visible = false;
                    } else {
                        self.input_box.mode = InputBoxMode::FileList;
                        self.active_input = unsafe { &mut *(&mut self.input_box as *mut _) as &'app mut dyn Input };
                        self.input_box.visible = true;
                    }
                }
            }
            Key::Tab if modifier == Modifiers::Control && action == Action::Press => {
                self.cycle_focus();
            }
            Key::Q if modifier == Modifiers::Control => {
                self.close_requested = true;
            }
            Key::F1 => if action == Action::Press {
                if modifier == Modifiers::Control {
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
            Key::D if modifier == Modifiers::Control && action == Action::Press => {
                self.debug_view.visibile = !self.debug_view.visibile;
            }
            Key::N if modifier == Modifiers::Control && action == Action::Press => {
                let size = self.window_size;
                self.open_text_view(self.active_panel(), Some("new view".into()), size);
            }
            _ => {
                self.active_input.handle_key(key, action, modifier);
            }
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

        if self.popup.visible {
            self.popup.view.draw();
        }

        self.status_bar.draw();

        if self.input_box.visible {
            self.input_box.draw();
        }

        // always draw the debug interface last, as it should overlay everything
        if self.debug_view.visibile {
            self.debug_view.draw();
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
        if self.panels.iter().map(|p| p.children.len()).sum::<usize>() == 1 {
            // We only have 1 view/window open. Close the program. In the future, we might have some file browser or whatever, that'll be a "main/unclosable" that will be displayed instead. Until then though
            // we just shut shit down.
            self.close_requested = true;
            return;
        }

        let view = unsafe { self.active_view.as_mut().unwrap() };

        let view_id = view.id;
        let panel_id = view.panel_id.unwrap();
        let panel = self.panels.get_mut(*panel_id as usize).unwrap();

        self.active_view = {
            let v = panel.remove_view(view_id).unwrap();
            println!("Closing and dropping resources of view: {:?}", v);
            panel.children.last_mut().unwrap() as _
        };

        panel.layout();
        self.decorate_active_view();
        self.active_ui_element = UID::View(*self.get_active_view().id);
    }

    pub fn set_debug(&mut self, set: bool) {
        self.debug = set;
    }
}

pub fn cast_ref_to_input<'app, T: Input>(t: &'app mut T) -> &'app mut dyn Input where T: 'app {
    unsafe {
        let a = t as *mut T;
        &mut (*a) as &'app mut dyn Input
    }
}

pub fn cast_ptr_to_input<'app, T: Input>(t: *mut T) -> &'app mut dyn Input where T: 'app {
        unsafe { &mut (*t) as &'app mut dyn Input }
}