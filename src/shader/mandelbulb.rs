use glm::{asin, atan, cos, length, log, max, pow, sin, vec3};

pub fn mandelbulb(p: &glm::Vec3, power: f32, phase: f32) -> f32 {
    let mut z = *p;
    let mut r: f32 = 0.0;
    let mut theta: f32;
    let mut phi: f32;
    let mut dr = 1.0;
    for _ in 0..32 {
        // change i < # for iterations.
        r = length(z);
        if r > 2.0 {
            continue;
        }
        theta = atan(z.y / z.x);
        phi = asin(z.z / r) + phase;
        dr = pow(r, power - 1.0) * dr * power + 1.0;
        r = pow(r, power);
        theta = theta * power;
        phi = phi * power;
        z = vec3(cos(theta) * cos(phi) * r, sin(theta) * cos(phi), sin(phi));
    }
    return max(0.001, 0.25 * log(r) * r / dr);
}
