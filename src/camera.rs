
use glam::Vec3;
use std::f32::consts;

pub const UP: Vec3 = Vec3::Y;

pub struct Camera {
	pub position: glam::Vec3,
	pub direction: glam::Vec3,

	pub yaw: f32,
	pub pitch: f32,
	pub fieldOfView: f32,
}

impl Default for Camera {
	fn default() -> Camera {
		Camera {
			position: Vec3::ZERO,
			direction: Vec3::X,
	
			yaw: -consts::PI / 2.,
			pitch: 0.0,
			fieldOfView: -consts::PI / 4.,
		}
	}
}