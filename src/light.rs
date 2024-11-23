#[derive(Debug)]
pub struct SunLight {
    pub direction: glm::Vec3,
    pub color: glm::Vec3,
}

impl SunLight {
    pub fn new() -> Self {
        Self {
            direction: glm::vec3(0.0, 0.0, -1.0),
            color: glm::vec3(1.0, 1.0, 1.0),
        }
    }
}
