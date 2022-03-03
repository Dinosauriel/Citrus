
use glam::Vec3;
use std::f32::consts;

pub const UP: Vec3 = Vec3::Y;

pub struct Camera {
	pub position: glam::Vec3,
	pub direction: glam::Vec3,

	pub yaw: f32,
	pub pitch: f32,
	pub field_of_view: f32,
}

impl Camera {
	pub fn set_pitch_and_yaw(mut self, new_pitch: &f32, new_yaw: &f32) -> Camera {
		self.pitch = new_pitch.min(consts::PI / 2.).max(- consts::PI / 2.);
		self.yaw = *new_yaw;

		self.direction = Vec3::new(
			self.yaw.cos() * self.pitch.cos(),
			self.pitch.sin(),
			self.yaw.sin() * self.pitch.cos()
		).normalize();
		return self;
	}
}

impl Default for Camera {
	fn default() -> Camera {
		Camera {
			position: Vec3::ZERO,
			direction: Vec3::Z,
	
			yaw: -consts::PI / 2.,
			pitch: 0.0,
			field_of_view: -consts::PI / 4.,
		}
	}
}