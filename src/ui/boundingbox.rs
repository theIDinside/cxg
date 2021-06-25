use crate::datastructure::generic::Vec2i;


pub struct BoundingBox {
    /// Bottom left corner
    pub min: Vec2i,     
    /// Top right corner
    pub max: Vec2i
}

impl BoundingBox {
    /// Create a new bounding box. No assertions or checks is done to verify that min <= max. User must take responsibility for that
    pub fn new(min: Vec2i, max: Vec2i) -> BoundingBox { 
        debug_assert!(min.x <= max.x && min.y <= max.y, "Assertion failed for {:?} <= {:?}", min, max);
        BoundingBox { min, max } 
    }

    /// Checks if parameter pos, is inside the bounding box coordinate space
    pub fn box_hit_check(&self, pos: Vec2i) -> bool {
        pos.x >= self.min.x && pos.y >= self.min.y &&
        pos.x <= self.max.x && pos.y <= self.max.y
    }
}