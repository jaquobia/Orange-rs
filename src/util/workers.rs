use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::{JoinHandle, self};

pub struct WorkerThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl WorkerThread {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
    pub fn spawn<F: FnMut() -> () + Send + 'static>(&mut self, mut f: F) {
        let stop = self.stop.clone(); 
        self.handle = Some(thread::spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::Acquire) {
                f();
            }
        }));
    }
    pub fn stop(&mut self) {
        self.handle.take().and_then(|handle| {
            self.stop.store(true, std::sync::atomic::Ordering::Release);
            match handle.join() {
                Ok(()) => { Some(()) },
                Err(_) => { None },
            }
        });
    }
}

impl Drop for WorkerThread {
    fn drop(&mut self) {
        self.stop();
    }
}
