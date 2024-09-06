pub mod counter;

use counter::{guess_cpu_freq, read_os_timer};

use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use queue::Queue;
use std::collections::HashMap;
use std::time::Duration;

pub struct TimeElapsed {
    start: u64,
    stop: Option<u64>,
}

impl TimeElapsed {
    pub fn new() -> Self {
        TimeElapsed {
            start: read_os_timer(),
            stop: None,
        }
    }

    pub fn stop(&mut self) -> () {
        self.stop = Some(read_os_timer());
    }

    fn elapsed(&self, cpu_freq: f64) -> Duration {
        let ts = self.stop.unwrap() - self.start;
        let nanosec = (ts as f64 / cpu_freq) * 1e9;

        Duration::new(0, nanosec as u32)
    }
}

pub struct Timer {
    profiles: HashMap<String, Vec<TimeElapsed>>,
    paused: Queue<String>,
    running: Option<String>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            profiles: HashMap::new(),
            paused: Queue::new(),
            running: None,
        }
    }
    pub fn start(&mut self, ident: &str) -> () {
        let ident = ident.to_string();

        match self.running.take() {
            Some(id) => self.pause(&id),
            None => {}
        };

        self.running = Some(ident.clone());

        match self.profiles.get_mut(&ident) {
            Some(profile) => profile.push(TimeElapsed::new()),
            None => {
                self.profiles.insert(ident, vec![TimeElapsed::new()]);
            }
        }
    }

    pub fn stop(&mut self, ident: &str) -> () {
        self.profile_mut(&ident.to_string()).stop();
        self.running = None;

        match self.paused.dequeue() {
            Some(ident) => self.start(&ident),
            None => {}
        }

        todo!("Check that ident we want to stop is currently running");
    }

    fn profile_mut(&mut self, ident: &str) -> &mut TimeElapsed {
        self.profiles
            .get_mut(ident)
            .expect(&format!("Profile for {:} does not exists", ident))
            .last_mut()
            .expect("Should always start before stopping")
    }

    fn elapsed_total(&mut self, ident: &str) -> Duration {
        let cpu_freq = guess_cpu_freq(Some(100));
        let profiles = self
            .profiles
            .get(ident)
            .expect(&format!("Profile for {:} does not exists", ident));

        profiles
            .iter()
            .fold(Duration::new(0, 0), |a, b| a + b.elapsed(cpu_freq as f64))
    }

    pub fn pause(&mut self, ident: &str) -> () {
        self.profile_mut(ident).stop();
        self.paused.queue(ident.to_string()).unwrap();
    }
}

pub static mut TIMER: Lazy<Timer> = Lazy::new(Timer::new);

#[cfg(test)]
mod test {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timer_simple() {
        let mut timer = Timer::new();

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        assert!(timer.elapsed_total("foo") > Duration::new(0, 0))
    }

    #[test]
    fn test_timer_complex() {
        let mut timer = Timer::new();

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        let elapsed_1 = timer.elapsed_total("foo");

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        assert!(elapsed_1 < timer.elapsed_total("foo"));
    }

    #[test]
    fn test_timer_pause() {
        let mut timer = Timer::new();

        assert_eq!(timer.running, None);

        timer.start("foo");
        assert_eq!(timer.running, Some("foo".to_string()));

        timer.start("foofoo");
        let elapsed_foo_1 = timer.elapsed_total("foo");
        assert_eq!(timer.running, Some("foofoo".to_string()));

        timer.stop("foofoo");
        assert_eq!(timer.running, Some("foo".to_string()));
        timer.stop("foo");

        assert!(elapsed_foo_1 < timer.elapsed_total("foo"))
    }
}
