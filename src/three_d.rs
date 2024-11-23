pub trait LocalToGlobal {
    fn to_global(&self, position: &glm::Vec3, direction: &glm::Vec3) -> (glm::Vec3, glm::Vec3);
}
