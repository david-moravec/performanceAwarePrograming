pub mod counter;

use counter::{guess_cpu_freq, read_os_timer, ts_ratio};

use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use queue::Queue;
use std::collections::HashMap;
use std::fmt::Display;
use std::time::Duration;

#[derive(Debug)]
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

    fn elapsed_duration(&self, cpu_freq: f64) -> Duration {
        let ts = self.stop.unwrap() - self.start;
        let nanosec = (ts as f64 / cpu_freq) * 1e9;

        Duration::new(0, nanosec as u32)
    }

    fn elapsed_ts(&self) -> u64 {
        self.stop.unwrap() - self.start
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
        self.last_elapsed(&ident.to_string()).stop();
        self.running = None;

        match self.paused.dequeue() {
            Some(ident) => self.start(&ident),
            None => {}
        }

        // TODO: ("Check that ident we want to stop is currently running");
    }

    fn last_elapsed(&mut self, ident: &str) -> &mut TimeElapsed {
        self.profiles
            .get_mut(ident)
            .expect(&format!("Profile for {:} does not exists", ident))
            .last_mut()
            .expect("Should always start before stopping")
    }

    fn elapsed_total_duration(&self, ident: &str) -> Duration {
        let cpu_freq = guess_cpu_freq(Some(100));
        let profiles = self
            .profiles
            .get(ident)
            .expect(&format!("Profile for {:} does not exists", ident));

        profiles.iter().fold(Duration::new(0, 0), |a, b| {
            a + b.elapsed_duration(cpu_freq as f64)
        })
    }

    fn elapsed_total_ts(&self, ident: &str) -> u64 {
        let profile = self
            .profiles
            .get(ident)
            .expect(&format!("Profile for {:} does not exists", ident));

        profile.iter().fold(0, |a, b| a + b.elapsed_ts())
    }

    fn pause(&mut self, ident: &str) -> () {
        self.last_elapsed(ident).stop();
        self.paused.queue(ident.to_string()).unwrap();
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.profiles.get("main") {
            Some(_) => {
                writeln!(f, "\nTotal time: {:?}", self.elapsed_total_duration("main"))?;
                for ident in self.profiles.keys() {
                    if ident != "main" {
                        writeln!(
                            f,
                            "{}: {} ({:.2}%)",
                            ident,
                            self.elapsed_total_ts(ident),
                            ts_ratio(self.elapsed_total_ts(ident), self.elapsed_total_ts("main"))
                        )?;
                    }
                }
            }
            None => (),
        }


        Ok(())
    }
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:}", self)
    }
}

pub static mut TIMER: Lazy<Timer> = Lazy::new(Timer::new);

pub fn print_timer() -> () {
    unsafe {
        println!("{}", ::once_cell::sync::Lazy::get(&TIMER).unwrap())
    }
}

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

        assert!(timer.elapsed_total_duration("foo") > Duration::new(0, 0))
    }

    #[test]
    fn test_timer_complex() {
        let mut timer = Timer::new();

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        let elapsed_1 = timer.elapsed_total_duration("foo");

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        assert!(elapsed_1 < timer.elapsed_total_duration("foo"));
    }

    #[test]
    fn test_timer_pause() {
        let mut timer = Timer::new();

        assert_eq!(timer.running, None);

        timer.start("foo");
        assert_eq!(timer.running, Some("foo".to_string()));

        timer.start("foofoo");
        let elapsed_foo_1 = timer.elapsed_total_duration("foo");
        assert_eq!(timer.running, Some("foofoo".to_string()));

        timer.stop("foofoo");
        assert_eq!(timer.running, Some("foo".to_string()));
        timer.stop("foo");

        assert!(elapsed_foo_1 < timer.elapsed_total_duration("foo"))
    }
}
