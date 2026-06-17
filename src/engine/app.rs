use crate::engine::window::State;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(State::new(window).await.expect("Unable to create canvas!!!"))
                        .is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        // This is where proxy.send_event() ends up
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    // Method listenning to event to trigger behaviors (ex: button pressed)
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            // Clicking on the close button
            WindowEvent::CloseRequested => event_loop.exit(),

            // Event resizing the window
            WindowEvent::Resized(size) => state.resize(size.width, size.height),

            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }

            // Keyboard input event
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                // Exit the programm when pressing Escape key
                (KeyCode::Escape, true) => event_loop.exit(),

                // Resizing the window
                (KeyCode::KeyR, true) => {
                    // Resizing the window
                    let _ =
                        state.window.request_inner_size(winit::dpi::LogicalSize::new(1200, 1000));

                    // Request a redraw after resizing
                    state.window.request_redraw();

                    // Checking the modified width and height
                    let size = state.window.inner_size();
                    println!(
                        "Window size - width: {:?}, height: {:?}",
                        size.width, size.height
                    );
                }

                // Toggle fullscreen
                (KeyCode::KeyF, true) => {
                    if state.window.fullscreen().is_some() {
                        state.window.set_fullscreen(None);
                        println!("Exited fullscreen");
                    } else {
                        state
                            .window
                            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        println!("Entered fullscreen");
                    }
                }

                // Toggle triangle color
                (KeyCode::Space, true) => {}

                _ => {
                    // Any other key pressed
                }
            },

            // Listening for the mouse movements
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(state_ref) = &mut self.state {
                    // Normalize coordinates to 0.0-1.0 range based on window size
                    let window_size = state_ref.window.inner_size();
                    let normalized_x = position.x / window_size.width as f64;
                    let normalized_y = position.y / window_size.height as f64;

                    // Update color based on cursor position
                    state_ref.color = wgpu::Color {
                        r: normalized_x,
                        g: 0.2,
                        b: normalized_y,
                        a: 1.0,
                    };

                    println!(
                        "Updated color - R: {:.2}, B: {:.2}",
                        normalized_x, normalized_y
                    );
                }
            }

            _ => {
                // Any other event
            }
        }
    }
}
