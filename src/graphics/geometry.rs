use glam::Vec2;

#[derive(Debug)]
pub struct Rect2D {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect2D {
    pub const ZERO: Self = Rect2D {
        pos: Vec2::ZERO,
        size: Vec2::ZERO,
    };
}
