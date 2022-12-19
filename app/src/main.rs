fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    orange_rs::handle_args(&args);
    orange_rs::main_loop();
}
