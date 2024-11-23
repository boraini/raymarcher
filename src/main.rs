use winit::event_loop::EventLoop;

mod app;
mod camera;
mod light;
mod macros;
mod renderer;
mod three_d;

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    app::run_app(event_loop).unwrap();
}
