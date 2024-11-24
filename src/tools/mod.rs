use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub struct FpsCounter {
    tx: mpsc::Sender<()>,
    rx: mpsc::Receiver<()>,
    start_time: Instant,
    frame_count: u32,
    is_measuring: bool,
}

impl FpsCounter {
    const DEFAULT_DURATION: Duration = Duration::from_secs(5);

    pub fn new() -> FpsCounter {
        let (tx, rx) = mpsc::channel();
        FpsCounter {
            tx,
            rx,
            start_time: Instant::now(),
            frame_count: 0,
            is_measuring: false,
        }
    }

    pub fn begin_measuring(&mut self) {
        if self.is_measuring {
            return;
        }

        self.queue_next_measurement();

        self.is_measuring = true;
    }

    pub fn stop_measuring(&mut self) {
        if !self.is_measuring {
            return;
        }

        (self.tx, self.rx) = mpsc::channel();

        self.is_measuring = false;
    }

    pub fn incr_frame_count(&mut self) {
        if !self.is_measuring {
            return;
        }

        self.frame_count += 1;

        match self.rx.try_recv() {
            Ok(()) => {
                let acutal_time_passed = Instant::now().duration_since(self.start_time);
                println!(
                    "Roughly {} secs have passed. {} fps",
                    FpsCounter::DEFAULT_DURATION.as_secs(),
                    self.frame_count as f64 / acutal_time_passed.as_millis() as f64 * 1000.0
                );
                self.frame_count = 0;
                
                self.queue_next_measurement();
            }
            Err(_) => (),
        }
    }

    fn queue_next_measurement(&mut self) {
        let thread_tx = self.tx.clone();

        self.start_time = Instant::now();
        thread::spawn(move || {
            thread::sleep(FpsCounter::DEFAULT_DURATION);
            match thread_tx.send(()) {
                Ok(_) => (),
                Err(_) => (),
            };
        });
    }
}
