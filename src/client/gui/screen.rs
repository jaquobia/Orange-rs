pub trait Screen {
    fn new() -> Self where Self: Sized;
}

pub struct MainMenu {

}

impl Screen for MainMenu {
    fn new() -> Self {
        Self {
            
        } 
    }
}
