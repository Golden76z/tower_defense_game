use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::Window,
};

#[derive(Default)]
pub struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    // Method called when the App is active (& resumed for mobile)
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window when app becomes active
        let window = event_loop.create_window(Window::default_attributes()).unwrap();

        // Storing the window in our App
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("Redrawing game frame");

                // Request next redraw
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
