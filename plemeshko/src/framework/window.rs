use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub fn initialize() -> (EventLoop<()>, Window) {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(crate::params::STARTUP_WINDOW_WIDTH, crate::params::STARTUP_WINDOW_HEIGHT);
        WindowBuilder::new()
            .with_title(crate::params::WINDOW_TITLE)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    (event_loop, window)
}
