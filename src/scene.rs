use crate::{camera::Camera, light::SunLight, three_d::LocalToGlobal};

pub struct Scene {
    pub camera: Camera,
    pub light: SunLight,
    pub mouse: Option<glm::Vec2>,
    pub mouse_down: bool,
}

impl Scene {
    pub fn init() -> Self {
        let mut scene = Self {
            camera: Camera::new(),
            light: SunLight::new(),
            mouse: None,
            mouse_down: false,
        };
        scene
            .camera
            .set_position_and_forward(glm::vec3(0.0, 0.0, 2.0), scene.camera.forward);

        // scene.camera.set_position_and_forward(glm::vec3(0.018368, 0.016674, 0.027951), glm::vec3(0.2, 0.0, 0.0));
        scene.light.direction = glm::normalize(glm::vec3(-1.0, -1.0, -1.0));
        scene
    }

    pub fn should_update(&self) -> bool {
        self.camera.should_update()
    }
    pub fn update_time(&mut self, t: u128) {
        self.camera.update_time(t);
        self.light.direction = glm::normalize(
            self.camera
                .to_global(&glm::vec3(0., 0., 0.), &glm::vec3(-1., -1., -1.))
                .1,
        );
    }
}
