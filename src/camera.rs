use core::f32;

use crate::three_d::LocalToGlobal;

#[derive(Debug)]
pub struct Camera {
    pub position: glm::Vec3,
    pub forward: glm::Vec3,
    pub fov: f32,
    prev_position: glm::Vec3,
    next_position: glm::Vec3,
    prev_forward: glm::Vec3,
    next_forward: glm::Vec3,
    t: u128,
    t_start: u128,
    t_end: u128,
    aspect: f32,
    update_flag: bool,
}

fn easing(t: u128, a: u128, b: u128) -> f32 {
    let t = ((t - a) as f32) / ((b - a) as f32);
    if t < 0.0 {
        return 0.0;
    }
    if t > 1.0 {
        return 1.0;
    }
    return t * t * (3.0 - 2.0 * t);
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            forward: glm::vec3(0.0, 0.0, -1.0),
            prev_position: glm::vec3(0.0, 0.0, 0.0),
            prev_forward: glm::vec3(0.0, 0.0, -1.0),
            next_position: glm::vec3(0.0, 0.0, 0.0),
            next_forward: glm::vec3(0.0, 0.0, -1.0),
            t: 0,
            t_start: 0,
            t_end: 0,
            aspect: 1.0,
            fov: std::f32::consts::PI / 2.0,
            update_flag: true,
        }
    }

    pub fn update_time(&mut self, t: u128) {
        self.t = t;
        let fac = easing(t, self.t_start, self.t_end);
        if fac <= 0.0 {
            self.position = self.prev_position;
            self.forward = self.prev_forward;
            return;
        }
        if fac >= 1.0 {
            self.position = self.next_position;
            self.forward = self.next_forward;
            return;
        }
        self.position = glm::mix_s(self.prev_position, self.next_position, fac);
        self.forward = glm::mix_s(self.prev_forward, self.next_forward, fac);
        self.update_flag = false;
    }

    pub fn set_aspect(&mut self, w: f32, h: f32) {
        if w == 0.0 || h == 0.0 {
            self.aspect = 1.0;
        } else {
            self.aspect = w / h;
        }
    }

    pub fn set_position_and_forward(&mut self, next_position: glm::Vec3, next_forward: glm::Vec3) {
        self.next_position = next_position;
        self.next_forward = next_forward;
        self.t_start = 0;
        self.t_end = 1;
        self.t = 2;
    }

    pub fn animate_between(
        &mut self,
        next_position: glm::Vec3,
        next_forward: glm::Vec3,
        duration: u128,
    ) {
        self.update_time(self.t);
        self.prev_position = self.position;
        self.next_position = next_position;
        self.prev_forward = self.forward;
        self.next_forward = next_forward;
        self.t_start = self.t;
        self.t_end = self.t + duration;
    }

    pub fn get_corners(&self) -> [glm::Vec3; 4] {
        let mut dest: [glm::Vec3; 4] = unsafe { std::mem::zeroed() };
        let foc = glm::length(self.forward);
        let dx = foc * glm::tan(0.5 * self.fov);
        let dy = dx / self.aspect;
        let right = glm::normalize(glm::cross(self.forward, glm::vec3(0.0, 1.0, 0.0))) * dx;
        let up = glm::normalize(glm::cross(right, self.forward)) * dy;

        let mut i = 0;
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                dest[i] = glm::normalize(self.forward + right * x + up * y);
                i += 1;
            }
        }

        dest
    }

    pub fn get_stop_distance(&self) -> f32 {
        0.00001 * glm::length(self.forward)
    }

    pub fn translate_local(&mut self, dx: f32, dy: f32, dz: f32) {
        let (d, _) = self.to_global(&glm::vec3(dx, dy, dz), &glm::vec3(0.0, 0.0, 0.0));
        self.set_position_and_forward(d, self.forward);
    }

    pub fn orbit_controls(&mut self, dx: f32, dy: f32) {
        let delta_azimuth = dx / -200.0;
        let delta_pitch = dy / -200.0;

        let ident = glm::Mat4::new(
            glm::vec4(1.0, 0.0, 0.0, 0.0),
            glm::vec4(0.0, 1.0, 0.0, 0.0),
            glm::vec4(0.0, 0.0, 1.0, 0.0),
            glm::vec4(0.0, 0.0, 0.0, 1.0),
        );

        let pitch_axis = glm::cross(self.forward, glm::vec3(0.0, 1.0, 0.0));

        let pitch_matrix = glm::ext::rotate(&ident, delta_pitch, pitch_axis);
        let azimuth_matrix = glm::ext::rotate(&ident, delta_azimuth, glm::vec3(0.0, 1.0, 0.0));

        let n = azimuth_matrix
            * pitch_matrix
            * glm::vec4(self.forward.x, self.forward.y, self.forward.z, 1.0);
        let next_forward = glm::vec3(n.x, n.y, n.z);
        let next_position = self.position + self.forward - next_forward;

        self.set_position_and_forward(next_position, next_forward);
        self.update_flag = true;
    }

    pub fn zoom(&mut self, scroll_amount: f32, distance: f32) {
        let fac = glm::exp(0.3 * scroll_amount);
        let center_dist = glm::min(distance, glm::length(self.forward));
        let center_point = self.position + glm::normalize(self.forward) * center_dist;
        let next_forward = if fac > 1.0 {
            self.forward * fac
        } else {
            glm::normalize(self.forward) * fac * center_dist
        };
        let next_position = center_point - next_forward;

        self.set_position_and_forward(next_position, next_forward);
        self.update_flag = true;
    }

    pub fn should_update(&self) -> bool {
        self.update_flag || self.t < self.t_end
    }
}

impl LocalToGlobal for Camera {
    fn to_global(&self, position: &glm::Vec3, direction: &glm::Vec3) -> (glm::Vec3, glm::Vec3) {
        let foc = glm::length(self.forward);
        let dx = foc * glm::tan(0.5 * self.fov);
        let dy = dx;
        let right = glm::normalize(glm::cross(self.forward, glm::vec3(0.0, 1.0, 0.0))) * dx;
        let up = glm::normalize(glm::cross(right, self.forward)) * dy;
        let position =
            self.position + right * position.x + up * position.y - self.forward * position.z;
        let direction = right * direction.x + up * direction.y - self.forward * direction.z;

        return (position, direction);
    }
}
