use tower_defense_lib::engine::app::App;
use winit::event_loop::EventLoop;

fn main() {
    let _ = run().expect("Error running the game loop");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    // Application
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Log in case the programm crash
        env_logger::init();
    }

    // Web app
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}
