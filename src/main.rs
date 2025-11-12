use tower_defense_lib::engine::window::App;
use winit::event_loop::EventLoop;

fn main() {
    // Log in case of an unexpected crash
    env_logger::init();

    // Creating event loop and initiating the App
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();

    // Running the App
    event_loop.run_app(&mut app).unwrap();
}
