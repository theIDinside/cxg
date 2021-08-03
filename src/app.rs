use crate::cmd::translation::{translate_key_input, InputTranslation};
use crate::cmd::CommandTag;
use crate::datastructure::generic::{Vec2, Vec2d, Vec2i};
use crate::debugger_catch;
use crate::debuginfo::DebugInfo;
use crate::opengl::{
    polygon_renderer::{PolygonRenderer, TextureMap, TextureType},
    rectangle_renderer::RectRenderer,
    shaders::{RectShader, TextShader},
    text_renderer::TextRenderer,
    types::RGBAColor,
};
use crate::textbuffer::operations::LineOperation;
use crate::textbuffer::{buffers::Buffers, CharBuffer, Movement};
use crate::ui::basic::{
    coordinate::{Coordinate, Layout, PointArithmetic, Size},
    frame::Frame,
};
use crate::ui::eventhandling::event::key_press;
use crate::ui::{
    clipboard::ClipBoard,
    debug_view::DebugView,
    eventhandling::event::{InputBehavior, InputResponse, InvalidInputElement},
    font::Font,
    inputbox::{InputBox, Mode},
    panel::{Panel, PanelId},
    view::{Popup, View, ViewId},
    MouseState, Viewable, UID,
};

use glfw::{Action, Key, Modifiers, MouseButton, Window};

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

pub static TEST_DATA: &str = include_str!("./textbuffer/contiguous/contiguous.rs");
static INACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.021, g: 0.62, b: 0.742123, a: 1.0 };
static ACTIVE_VIEW_BACKGROUND: RGBAColor = RGBAColor { r: 0.071, g: 0.202, b: 0.3242123, a: 1.0 };

fn all_views<'app>(panels: &'app Vec<Panel>) -> impl Iterator<Item = &View> + Clone {
    panels.iter().flat_map(|p| p.children.iter())
}

fn all_views_mut<'app>(panels: &'app mut Vec<Panel>) -> impl Iterator<Item = &'app mut View> + 'app {
    panels.iter_mut().flat_map(|p| p.children.iter_mut())
}

pub struct Application<'app> {
    /// Window Title
    _title_bar: String,
    /// Window Size
    window_size: Size,
    /// Total space of window, that can be occupied by panels (status bar for instance, is *not* counted among this space)
    panel_space_size: Size,
    /// Loaded fonts. Must be loaded up front, before application is initialized, as the reference must outlive Application<'app>
    fonts: Vec<Rc<Font>>,
    /// The shader for the font
    font_shader: TextShader,
    /// Shaders for rectangles/windows/views
    rect_shader: RectShader,
    /// Shader for drawing rectangles and polygons, that can be textured, have rounded corners using SDF in the shader code, etc
    polygon_shader: RectShader,
    /// The panels, which hold the different views, and manages their layout and size
    pub panels: Vec<Panel>,
    /// buffers we're editing and is live, yet not open in any view currently, lives in this field
    buffers: Buffers,
    /// The command popup, an input box similar to that of Clion, or VSCode, or Vim's command input line
    popup: Popup,
    /// The active element's id
    pub active_ui_element: UID,
    /// Whether or not we're in debug interface mode (showing different kinds of debug information)
    debug: bool,
    /// Pointer to the text editor view that is currently active
    pub active_view: *mut View,
    /// Pointer to the element that's currently receiving user input. This handle, handles the behavior of the application
    /// and dispatches accordingly to the right type, to determine what should be done when user inputs, key strokes or mouse movements, etc
    pub active_input: &'app mut dyn InputBehavior,
    /// We keep running the application until close_requested is true. If true, Application will see if all data and views are in an acceptably quittable state, such as,
    /// all files are saved to disk (aka pristine) or all files are cached to disk (unsaved, but stored in permanent medium in newest state) etc. If App is not in acceptably quittable state,
    /// close_requested will be set to false again, so that user can respond to Application asking the user about actions needed to quit.
    close_requested: bool,
    /// The input box, for opening files & running commands like VSCode
    input_box: InputBox,
    /// Debug view, shows frame rate, heap allocation, resident set size, shared library code size
    pub debug_view: DebugView,
    /// Current mouse state
    mouse_state: MouseState,
    /// renderer for "animations" such as when we're "moving" a window to a new place. Due to how i've designed the draw command list in PolygonRenderer and RectRenderer, I may very well be able
    /// to compress this into 3 renderers in total, instead of having a bunch of them
    rect_animation_renderer: RectRenderer,
    /// Currently *only* holds the background texture.
    /// todo: make this hold both the font atlas & and the future textures, such as button images, logos etc
    pub tex_map: TextureMap,

    pub clipboard: ClipBoard,

    key_bindings: HashMap<(glfw::Key, glfw::Action, glfw::Modifiers), InputTranslation>,
}

static mut INVALID_INPUT: InvalidInputElement = InvalidInputElement {};

impl<'app> Application<'app> {
    pub fn create(
        fonts: Vec<Rc<Font>>, font_shader: TextShader, rect_shader: RectShader, polygon_shader: RectShader, debug_info: DebugInfo,
    ) -> Application<'app> {
        let active_view_id = 0;
        let backgrounds = vec![
            (Path::new("./logo.png"), TextureType::Background(1)),
            (Path::new("./logo_transparent.png"), TextureType::Background(2)),
        ];
        let tex_map = TextureMap::new(backgrounds);
        font_shader.bind();
        let mvp = super::opengl::glinit::screen_projection_matrix(1024, 768, 0);
        font_shader.set_projection(&mvp);
        // utility renderer creation
        rect_shader.bind();
        rect_shader.set_projection(&mvp);
        polygon_shader.bind();
        polygon_shader.set_projection(&mvp);

        // let make_view_renderers = || (TextRenderer::create(font_shader.clone(), 1024 * 10), RectRenderer::create(rect_shader.clone(), 8 * 60));

        let make_view_renderers = || {
            (
                TextRenderer::create(font_shader.clone(), 1024),
                RectRenderer::create(rect_shader.clone(), 64),
                PolygonRenderer::create(polygon_shader.clone(), 64),
            )
        };

        let mut buffers = Buffers::new();

        // Create default 1st panel to hold views in
        let panel = Panel::new(0, Layout::Horizontal(0.into()), None, None, 1024, 768, Vec2i::new(0i32, 768i32));
        let mut panels = vec![panel];

        // Create the default 1st view
        let (tr, rr, pr) = make_view_renderers();
        let mut buffer = buffers.request_new_buffer();
        buffer.rebuild_metadata();
        let view = View::new(
            "Unnamed view",
            active_view_id.into(),
            tr,
            rr,
            pr,
            1024,
            768,
            ACTIVE_VIEW_BACKGROUND,
            buffer,
            fonts[0].clone(),
            fonts[1].clone(),
            tex_map.textures.get(&TextureType::Background(2)).map(|t| *t).unwrap(),
        );
        panels[0].add_view(view);

        // Create the popup UI
        let (tr, rr, pr) = make_view_renderers();
        let mut popup = View::new(
            "Popup view",
            (active_view_id + 1).into(),
            tr,
            rr,
            pr,
            524,
            518,
            ACTIVE_VIEW_BACKGROUND,
            Buffers::free_buffer(),
            fonts[0].clone(),
            fonts[1].clone(),
            tex_map.textures.get(&TextureType::Background(2)).map(|t| *t).unwrap(),
        );

        popup.set_anchor(Vec2i::new(250, 768 - 250));
        popup.update(None);
        // popup.window_renderer.set_color(RGBAColor { r: 0.3, g: 0.34, b: 0.48, a: 0.8 });
        let popup = Popup { visible: false, view: popup };

        // Creating the Debug View UI
        let (tr, rr, pr) = make_view_renderers();
        let dbg_view_bg_color = RGBAColor { r: 0.35, g: 0.7, b: 1.0, a: 0.95 };
        let mut debug_view = View::new(
            "debug_view",
            10.into(),
            tr,
            rr,
            pr,
            1024,
            768,
            dbg_view_bg_color,
            Buffers::free_buffer(),
            fonts[0].clone(),
            fonts[1].clone(),
            tex_map.textures.get(&TextureType::Background(2)).map(|t| *t).unwrap(),
        );
        debug_view.set_anchor(Vec2i::new(5, 763));
        debug_view.update(Some(tex_map.textures.get(&TextureType::Background(2)).map(|t| *t).unwrap()));
        // debug_view.window_renderer.set_color(RGBAColor { r: 0.35, g: 0.7, b: 1.0, a: 0.95 });
        let debug_view = DebugView::new(debug_view, debug_info, tex_map.textures.get(&TextureType::Background(2)).unwrap().clone());
        let ib_border_margin = 2;
        let ib_frame = Frame {
            anchor: Vec2i::new(250, 700),
            size: Size {
                width: 500,
                height: 500 + 2 * ib_border_margin, // fonts[1].row_height() + 2 * ib_border_margin
            },
        };
        let input_box = InputBox::new(ib_frame, fonts[1].clone(), &font_shader, &rect_shader);
        let rect_animation_renderer = RectRenderer::create(rect_shader.clone(), 8 * 60);

        let mut res = Application {
            _title_bar: "cxgledit".into(),
            window_size: Size::new(1024, 768),
            panel_space_size: Size::new(1024, 768),
            fonts,
            // status_bar,
            font_shader,
            rect_shader,
            polygon_shader,
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
            tex_map,
            clipboard: ClipBoard::new(),
            key_bindings: HashMap::new(),
        };
        let v = res.panels.last_mut().and_then(|p| p.children.last_mut()).unwrap() as *mut _;
        res.active_input = unsafe { &mut (*v) as &'app mut dyn InputBehavior };
        res.active_view = res.panels.last_mut().unwrap().get_view(active_view_id.into()).unwrap();
        res
    }

    pub fn decorate_active_view(&mut self) {
        let view = unsafe { self.active_view.as_mut().unwrap() };
        view.bg_color = ACTIVE_VIEW_BACKGROUND;
        view.window_renderer.set_color(ACTIVE_VIEW_BACKGROUND);
        view.update(None);
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
            let font = self.fonts[0].clone();
            let menu_font = self.fonts[1].clone();
            let Size { width, height } = view_size;
            let view_name = view_name.as_ref().map(|name| name.as_ref()).unwrap_or("unnamed view");
            let view = View::new(
                view_name,
                view_id.into(),
                TextRenderer::create(self.font_shader.clone(), 1024),
                RectRenderer::create(self.rect_shader.clone(), 64),
                PolygonRenderer::create(self.polygon_shader.clone(), 64),
                width,
                height,
                ACTIVE_VIEW_BACKGROUND,
                self.buffers.request_new_buffer(),
                font,
                menu_font,
                self.tex_map.textures.get(&TextureType::Background(2)).map(|t| *t).unwrap(),
            );
            self.active_ui_element = UID::View(*view.id);
            p.add_view(view);
            unsafe {
                (*self.active_view).bg_color = INACTIVE_VIEW_BACKGROUND;
                // (*self.active_view).window_renderer.set_color(INACTIVE_VIEW_BACKGROUND);
                (*self.active_view).update(None);
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
            view.update(None);
            view.id
        };
        {
            let mut iter = all_views(&self.panels).filter(|v| v.visible).cycle();
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
        }
        self.decorate_active_view();
        self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn InputBehavior };
        let v = unsafe { self.active_view.as_mut().unwrap() };
        if !v.visible {
            v.visible = true;
        }
    }

    pub fn get_active_view(&mut self) -> &mut View {
        if self.popup.visible {
            return unsafe { &mut *(&self.popup.view as *const _ as *mut _) };
        } else {
            unsafe { &mut *self.active_view }
        }
    }

    pub fn get_view_unchecked(&mut self, view_id: ViewId) -> &mut View {
        self.panels
            .iter_mut()
            .flat_map(|p| p.children.iter_mut())
            .find(|v| v.id == view_id)
            .unwrap()
    }

    pub fn get_active_view_ptr(&mut self) -> *mut View {
        if self.popup.visible {
            &mut self.popup.view as *mut _
        } else {
            self.active_view
        }
    }

    pub fn set_active_view(&mut self, view: &View) {
        self.active_view = view as *const _ as *mut _;
    }

    fn sync_shader_uniform_projection(&mut self) {
        let (width, height) = self.window_size.values();
        let mvp = super::opengl::glinit::screen_projection_matrix(*width as _, *height as _, 0);
        self.font_shader.set_projection(&mvp);
        self.rect_shader.set_projection(&mvp);
        self.polygon_shader.set_projection(&mvp);
    }

    fn handle_resize_event(&mut self, width: i32, height: i32) {
        let ps_x = self.panel_space_size.width;
        let ps_y = self.panel_space_size.height;
        let ax = width as f64 / ps_x as f64;
        let ay = height as f64 / ps_y as f64;

        let new_panel_space_size = Size::new(width, height);
        let size_change_factor = new_panel_space_size / self.panel_space_size;
        assert_eq!(ax, size_change_factor.x as _);
        assert_eq!(ay, size_change_factor.y as _);

        for p in self.panels.iter_mut() {
            let panel_anchor = Vec2d::new(p.anchor.x as _, p.anchor.y as _);
            let Vec2 { x, y } = panel_anchor * size_change_factor;
            let new_size = Size::vector_multiply(p.size, size_change_factor);
            p.set_anchor(Vec2i::new(x.round() as _, y.round() as _));
            p.resize(new_size);
            p.layout();
        }

        Application::set_dimensions(self, width, height);
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

    pub fn process_all_events(&mut self, window: &mut Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
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
                    } else {
                        self.handle_mouse_input(MouseState::Released(mbtn, pos));
                    }
                }
                glfw::WindowEvent::CursorPos(mposx, mposy) => {
                    let new_pos = self.translate_screen_to_application_space(Vec2d::new(mposx, mposy));
                    match self.mouse_state {
                        MouseState::UIElementClicked(view, btn, pos) => {
                            // If control is pressed, we want to activate the Drag action for the UI element itsef
                            let cv = self.get_view_unchecked(view);
                            if cv.title_frame.to_bb().box_hit_check(pos.to_i32()) {
                                self.mouse_state = MouseState::UIElementDrag(view, btn, new_pos);
                            } else {
                                if window.get_key(glfw::Key::LeftControl) == Action::Press || window.get_key(glfw::Key::RightControl) == Action::Press {
                                    self.mouse_state = MouseState::UIElementDrag(view, btn, new_pos);
                                } else {
                                    // Otherwise, we want to tell the UI element to handle the drag action for us; e.g. for selecting text
                                    let new_state = MouseState::UIElementDragAction(view, btn, pos, new_pos);
                                    self.handle_mouse_input(new_state);
                                }
                            }
                        }
                        MouseState::UIElementDrag(view, btn, _) => {
                            // Continue drag, REMEMBER, MUST translate to Application coordinate space
                            self.mouse_state = MouseState::UIElementDrag(view, btn, new_pos)
                        }
                        MouseState::UIElementDragAction(v, btn, begin, ..) => {
                            let new_state = MouseState::UIElementDragAction(v, btn, begin, new_pos);
                            self.handle_mouse_input(new_state);
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
                    if let Some(clicked_view) = clicked_view {
                        let id = clicked_view.id;

                        let de_activate_old = id != active_id;
                        clicked_view.mouse_clicked(pos);
                        self.active_view = &mut (*clicked_view) as *mut _;
                        self.active_input = cast_ptr_to_input(self.active_view); // unsafe { self.active_view.as_mut().unwrap() as &'app mut dyn Input };
                        self.decorate_active_view();
                        // check if the clicked view, was the already active view

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
                                v.update(None);
                            }
                        }
                        self.mouse_state = MouseState::UIElementClicked(id, MouseButton::Button1, p);
                    }
                }
            }
            MouseState::UIElementClicked(_view_id, _btn, _pos) => {}
            MouseState::UIElementDrag(_maybe_view, _btn, _pos) => {}
            MouseState::UIElementDragAction(_view, _btn, begin, current) => {
                let pos = begin.to_i32();
                let view_handling_action = self
                    .panels
                    .iter_mut()
                    .flat_map(|p| p.children.iter_mut())
                    .find(|v| v.bounding_box().box_hit_check(pos));
                if let Some(handling_view) = view_handling_action {
                    handling_view.mouse_dragged(begin.to_i32(), current.to_i32());
                }
                self.mouse_state = new_state;
            }
            MouseState::Released(_btn, pos) => {
                match self.mouse_state {
                    MouseState::UIElementDrag(dragged_view_id, _, _) => {
                        let view_dropped_on = self
                            .panels
                            .iter_mut()
                            .flat_map(|p| p.children.iter_mut())
                            .find(|v| v.bounding_box().box_hit_check(pos.to_i32()))
                            .map(|v| v.id);
                        if let Some(view_dropped_on) = view_dropped_on {
                            if dragged_view_id != view_dropped_on {
                                let p_a = self
                                    .panels
                                    .iter_mut()
                                    .position(|p| p.children.iter().any(|f| f.id == dragged_view_id));
                                let mut panel_a = self.panels.swap_remove(p_a.unwrap());
                                let va = panel_a.children.iter().position(|v| v.id == dragged_view_id);

                                let coexist = panel_a.children.iter().any(|v| v.id == view_dropped_on);
                                if coexist {
                                    let vb = panel_a.children.iter().position(|v| v.id == view_dropped_on);
                                    panel_a.children.swap(va.unwrap(), vb.unwrap());
                                    panel_a.layout();
                                    for v in panel_a.children.iter_mut() {
                                        if v.id == dragged_view_id {
                                            v.bg_color = ACTIVE_VIEW_BACKGROUND;
                                            v.window_renderer.set_color(ACTIVE_VIEW_BACKGROUND);
                                            v.update(None);
                                            self.active_view = v as *mut _;
                                            self.active_input = cast_ptr_to_input(self.active_view);
                                        } else {
                                            v.bg_color = INACTIVE_VIEW_BACKGROUND;
                                            v.window_renderer.set_color(INACTIVE_VIEW_BACKGROUND);
                                            v.update(None);
                                        }
                                    }
                                    self.panels.insert(p_a.unwrap(), panel_a);
                                } else {
                                    let p_b = self
                                        .panels
                                        .iter_mut()
                                        .position(|p| p.children.iter().any(|f| f.id == view_dropped_on));
                                    let mut panel_b = self.panels.swap_remove(p_b.unwrap());

                                    let vb = panel_b.children.iter().position(|v| v.id == dragged_view_id);
                                    std::mem::swap(panel_a.children.get_mut(va.unwrap()).unwrap(), panel_b.children.get_mut(vb.unwrap()).unwrap());
                                    self.panels.insert(p_a.unwrap(), panel_a);
                                    self.panels.insert(p_b.unwrap(), panel_b);
                                }
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

    pub fn toggle_input_box(&mut self, mode: Mode) {
        if self.input_box.visible {
            self.active_input = cast_ptr_to_input(self.active_view);
            self.input_box.clear();
            self.input_box.visible = false;
            self.input_box.mode = mode;
        } else {
            self.input_box.mode = mode;
            // self.active_input = &mut self.input_box as &'app mut dyn Input;
            self.active_input = unsafe { &mut *(&mut self.input_box as *mut _) as &'app mut dyn InputBehavior };
            self.input_box.visible = true;
        }
    }

    #[rustfmt::skip]
    pub fn handle_key_event(&mut self, _window: &mut Window, key: glfw::Key, action: glfw::Action, modifier: glfw::Modifiers) {
        // todo: this is where we will hook the config library into. It will read from a config -> parse that into a map, which we map the
        //  input against, and it will have to return an InputTranslation, which instead match on in this function, instead of matching
        //  directly on key input.
        let _op = translate_key_input(key, action, modifier);

         {
            match _op {
                InputTranslation::Cancel                                    => {}
                InputTranslation::Movement(movement)                        => {}
                InputTranslation::TextSelect(movement)                      => {}
                InputTranslation::Delete(movement)                          => {}
                InputTranslation::ChangeValueOfAssignment                   => {}
                InputTranslation::StaticInsertStr(_)                        => {}
                InputTranslation::Cut                                       => {}
                InputTranslation::Copy                                      => {}
                InputTranslation::Paste                                     => {}
                InputTranslation::Undo                                      => {}
                InputTranslation::Redo                                      => {}
                InputTranslation::OpenFile                                  => {}
                InputTranslation::SaveFile                                  => {}
                InputTranslation::Search                                    => {}
                InputTranslation::Goto                                      => {}
                InputTranslation::CycleFocus                                => {}
                InputTranslation::HideFocused                               => {}
                InputTranslation::ShowAll                                   => {}
                InputTranslation::ShowDebugInterface                        => {},
                InputTranslation::CloseActiveView                           => {},
                InputTranslation::Quit                                      => {},
                InputTranslation::OpenNewView                               => {}
                InputTranslation::LineOperation(line_op)                    => {},
                InputTranslation::Debug                                     => {}
            }
        }

        match key {
            Key::Escape | Key::CapsLock if key_press(action) => {
                if self.input_box.visible {
                    self.toggle_input_box(Mode::Command(CommandTag::Goto));
                } else {
                    self.active_input.handle_key(key, action, modifier);
                }
            }
            Key::F if key_press(action) && modifier == Modifiers::Control => {
                if key_press(action) {
                    self.toggle_input_box(Mode::Command(CommandTag::Find));
                }
            }
            Key::KpAdd => {}
            Key::W if modifier.contains(Modifiers::Control) && action == Action::Press => {
                self.close_active_view(modifier.contains(Modifiers::Shift));
            }
            Key::H if modifier == Modifiers::Control && action == Action::Press => {
                let visible = all_views(&self.panels).filter(|v| v.visible).count();
                if visible > 1 {
                    let v_ptr = unsafe { &mut (*self.active_view) };
                    self.cycle_focus();
                    v_ptr.visible = false;
                    for p in self.panels.iter_mut() {
                        p.layout();
                    }
                }
            }
            // Paste
            Key::V if key_press(action) && modifier == Modifiers::Control => {
                if let Some(v) = _window.get_clipboard_string() {
                    // todo: room for *plenty* of optimization here. Now we do brute force insert ch by ch,
                    //  which obviously introduces function call overhead, etc, etc
                    for ch in v.chars() {
                        self.active_input.handle_char(ch);
                    }
                } else {
                    // todo: room for *plenty* of optimization here. Now we do brute force insert ch by ch,
                    //  which obviously introduces function call overhead, etc, etc
                    for cb_data in self.clipboard.give() {
                        for ch in cb_data.chars() {
                            self.active_input.handle_char(ch);
                        }
                    }
                }
            }
            Key::G if modifier == Modifiers::Control && key_press(action) => {
                self.toggle_input_box(Mode::Command(CommandTag::Goto));
            }
            Key::S if modifier == Modifiers::Control | Modifiers::Shift && action == Action::Press => {
                let p = &mut self.panels;
                all_views_mut(p).for_each(|v| v.visible = true);
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
                if modifier == (Modifiers::Control | Modifiers::Shift) {
                    self.toggle_input_box(Mode::FileList);
                }
            }
            Key::Tab if action == Action::Press => {
                if modifier == Modifiers::Control {
                    self.cycle_focus();
                } else {
                    self.active_input.handle_key(key, action, modifier);
                }
            }
            Key::Q if modifier == Modifiers::Control => {
                self.close_requested = true;
            }
            Key::F1  => {
                if action == Action::Press {
                    if modifier == Modifiers::Shift {
                        self.active_input.handle_key(key, action, modifier);
                    } else {
                        let v = self.get_active_view();
                        println!("Cursor row: {}, Topmost line: {}", *v.buffer.cursor_row(), v.topmost_line_in_buffer);
                        println!("Cursor: {:?} <===> Meta cursor: {:?}", v.buffer.get_cursor(), v.buffer.meta_cursor);
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
            // dispatches handler to current active input, which we handle a possible response from
            _ => match self.active_input.handle_key(key, action, modifier) {
                InputResponse::OpenFile(path) => {
                    let v = self.get_active_view();
                    if v.buffer.empty() {
                        v.buffer.load_file(&path);
                        v.set_need_redraw();
                        v.update(None);
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
                        v.update(None);
                        self.input_box.visible = false;
                    }
                    self.input_box.clear();
                }
                InputResponse::Goto(line) => {
                    let v = self.get_active_view();
                    v.buffer.goto_line(line as usize);
                    v.set_view_on_buffer_cursor();
                    v.set_need_redraw();
                    v.update(None);
                    self.active_input = unsafe { &mut (*self.active_view) as &'app mut dyn InputBehavior };
                    self.input_box.visible = false;
                    self.input_box.clear();
                }
                InputResponse::Find(find) => {
                    // todo: use the regex crate for searching
                    let v = self.get_active_view();
                    v.buffer.search_next(&find);
                    v.set_view_on_buffer_cursor();
                    v.set_need_redraw();
                }
                InputResponse::SaveFile(file_path) => {
                    if let Some(p) = file_path {
                        let v = self.get_active_view();
                        v.buffer.save_file(&p);
                    } else {
                        // todo: we need to turn off _all_ GLFW input handling at this point. Because if we hit Ctrl+Q while the nfd-dialog is open
                        //  we have told our application to quit running, and it will try to exit - only to be blocked by the nfd. This doesn't seem safe at all.
                        //  best thing to do, would be to turn off all polling for input and restore state once we return from nfd
                        match nfd::open_save_dialog(Some("*"), Some(".")) {
                            Ok(res) => match res {
                                nfd::Response::Okay(file_name_selected) => {
                                    let v = self.get_active_view();
                                    v.buffer.save_file(Path::new(&file_name_selected));
                                }
                                nfd::Response::OkayMultiple(multi_string) => {
                                    println!("Response: {:?}", multi_string);
                                }
                                nfd::Response::Cancel => {}
                            },
                            Err(err) => {
                                println!("Error: {}", err);
                            }
                        }
                    }
                }
                // we discard the ClipboardCopy response, if it did not hold any data, which is why we match exactly on Some(data) here
                InputResponse::ClipboardCopy(Some(data)) => {
                    println!("Application clip board copy: '{}'", data);
                    self.clipboard.take(data);
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
        self.panel_space_size = Size::new(width, height);
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
        unsafe {
            gl::Scissor(0, 0, self.width(), self.height());
        }

        if self.popup.visible {
            self.popup.view.draw();
        }
        unsafe {
            gl::Scissor(0, 0, self.width(), self.height());
        }

        self.input_box.draw();
        unsafe {
            gl::Scissor(0, 0, self.width(), self.height());
        }
        self.debug_view.draw();
        unsafe {
            gl::Scissor(0, 0, self.width(), self.height());
        }
        if let MouseState::UIElementDrag(.., pos) = self.mouse_state {
            let v = unsafe { self.active_view.as_mut().unwrap() };
            let mut bb = v.bounding_box();
            bb.center_align_around(pos.to_i32());
            self.rect_animation_renderer
                .set_rect(bb, RGBAColor { r: 0.75, g: 0.75, b: 0.75, a: 0.25 });
            self.rect_animation_renderer.draw();
        } else {
            self.rect_animation_renderer.clear_data();
        }
    }

    pub fn close_active_view(&mut self, force_close: bool) {
        // we never detroy the popup window until the application is exited. So hitting "ctrl+w" or whatever keybinding we might have,
        // is just going to cancel the popup and hide it again
        if self.popup.visible {
            self.popup.visible = false;
            self.popup.reset();
            return;
        }
        // todo: we need to ask user,  what to do with unsaved files etc.

        let view = unsafe { self.active_view.as_mut().unwrap() };

        if view.buffer.pristine() || force_close {
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
        } else {
            println!("File has been altered! You must save the file.");
        }
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
