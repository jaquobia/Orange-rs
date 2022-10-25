use orange_rs::{run, handle_args};
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    handle_args(&args);
    run();
}
