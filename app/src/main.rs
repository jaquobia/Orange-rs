fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    orange_rs::handle_args(&args);
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Orange-rs")
        .with_window_icon(orange_rs::get_app_icon("icon.png"))
        .build(&event_loop).unwrap();
    orange_rs::main_loop(event_loop, window);
}
