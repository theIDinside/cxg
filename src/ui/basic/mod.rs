/// Coordinate types, such as Vec2i
pub mod coordinate;

/// Bounding box, represents a min and a max, from bottom left to top right of a UI element. Can be built from an Vec2i & Size
pub mod boundingbox;

/// A frame is a struct containing the anchor point of a UI element (it's most top left position) and it's size in pixels
pub mod frame;
