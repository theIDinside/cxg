
/// Default shader sources, compiled into the binary
pub mod source {
    pub const CURSOR_VERTEX_SHADER: &str = include_str!("../assets/cursor.vs.glsl");
    pub const CURSOR_FRAGMENT_SHADER: &str = include_str!("../assets/cursor.fs.glsl");
    pub const TEXT_VERTEX_SHADER: &str = include_str!("../assets/text.vs.glsl");
    pub const TEXT_FRAGMENT_SHADER: &str = include_str!("../assets/text.fs.glsl");
}