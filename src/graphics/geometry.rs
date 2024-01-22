use glam::Vec2;

#[derive(Debug)]
pub struct Rect2D {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect2D {
    pub const ZERO: Self = Rect2D {
        position: Vec2::ZERO,
        size: Vec2::ZERO,
    };
}
