use glam::Vec3;
use std::f32::consts;
use crate::controls;
use crate::world::ray;

pub const UP: Vec3 = Vec3::Y;

pub struct Camera {
    pub ray: ray::Ray,

    pub yaw: f32,
    pub pitch: f32,
    pub field_of_view: f32,

    prev_input_state_opt: Option<controls::InputState>,
}

impl Camera {
    pub fn set_pitch_and_yaw(&mut self, new_pitch: &f32, new_yaw: &f32) {
        self.pitch = new_pitch.min(consts::PI / 2.).max(- consts::PI / 2.);
        self.yaw = *new_yaw;

        self.ray.direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        ).normalize();
    }

    pub fn update_from_input_state(&mut self, input_state: &controls::InputState) {
        // println!("direction: {}, pitch: {}, yaw: {}", self.direction, self.pitch, self.yaw);
        let speed;
        if input_state.l_shift {
            speed = 0.2;
        } else {
            speed = 0.1;
        }

        if input_state.w {
            self.ray.origin += speed * self.ray.direction;
        }
        if input_state.a {
            self.ray.origin += speed * self.ray.direction.cross(UP).normalize();
        }
        if input_state.s {
            self.ray.origin -= speed * self.ray.direction;
        }
        if input_state.d {
            self.ray.origin -= speed * self.ray.direction.cross(UP).normalize();
        }
        if input_state.l_ctrl {
            self.ray.origin -= speed * UP;
        }
        if input_state.space {
            self.ray.origin += speed * UP;
        }
        if let Some(prev_input_state) = self.prev_input_state_opt {
            if prev_input_state.cursor_did_move {
                let delta_x = prev_input_state.cursor_x - input_state.cursor_x;
                let delta_y = prev_input_state.cursor_y - input_state.cursor_y;
    
                // println!("delta_x: {}, delta_y: {}", delta_x, delta_y);
        
                let new_yaw   = self.yaw + 0.01 * (delta_x as f32);
                let new_pitch = self.pitch + 0.01 * (delta_y as f32);
        
                self.set_pitch_and_yaw(&new_pitch, &new_yaw);
            }
        }

        self.prev_input_state_opt = Some(*input_state);
    }
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            ray: ray::Ray { origin: Vec3::ZERO, direction: Vec3::Z },
    
            yaw: consts::PI / 2.,
            pitch: 0.0,
            field_of_view: -consts::PI / 4.,

            prev_input_state_opt: None,
        }
    }
}