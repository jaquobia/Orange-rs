use instant::Duration;

pub mod mesh;
pub mod tessellator;
pub mod textures;
pub mod verticies;
pub mod world_renders;

pub struct ElapsedTime {
    time_now: instant::Instant,
    time_last: instant::Instant,
    dur: instant::Duration,
}

impl ElapsedTime {
    pub fn new() -> Self {
        let time_now = instant::Instant::now();
        let time_last = time_now;
        Self {
            time_now,
            time_last,
            dur: Duration::from_secs(0),
        }
    }

    pub fn tick(&mut self) {
        self.time_last = self.time_now;
        self.time_now = instant::Instant::now();
        self.dur = self.time_now - self.time_last;
    }

    pub fn elapsed_time(&self) -> f64 {
        self.dur.as_secs_f64()
    }
}
