mod application;
mod compositor;
mod utils;

use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use crate::application::App;


fn main() {
    // Initialize the event loop and window
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
