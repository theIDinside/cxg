pub mod shaders;
pub mod types;
/// Text renderer module. Renders text, using Views
pub mod text;
/// Rect renderer module. Renders simple rectangles, such as windows/borders and cursors 
pub mod rect;
#[macro_use]
pub mod glinit;

pub trait Renderable {
    fn render(&mut self);
}