pub mod counter;

use counter::{guess_cpu_freq, read_os_timer, ts_ratio};

use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
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
        let ts = self.stop.expect("not yet stopped") - self.start;
        let nanosec = (ts as f64 / cpu_freq) * 1e9;

        Duration::new(0, nanosec as u32)
    }

    fn elapsed_ts(&self) -> u64 {
        match self.stop {
            Some(t) => t - self.start,
            None => read_os_timer() - self.start,
        }
    }
}

pub struct Timer {
    profiles: HashMap<String, Vec<TimeElapsed>>,
    paused: VecDeque<String>,
    running: Option<String>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            profiles: HashMap::new(),
            paused: VecDeque::new(),
            running: None,
        }
    }

    pub fn start(&mut self, ident: &str) -> () {
        let ident = ident.to_string();

        self.pause_running();
        self.running = Some(ident.clone());

        match self.profiles.get_mut(&ident) {
            Some(profile) => profile.push(TimeElapsed::new()),
            None => {
                self.profiles.insert(ident, vec![TimeElapsed::new()]);
            }
        }
    }

    pub fn stop(&mut self, ident: &str) -> () {
        match self.running {
            Some(ref running) => {
                if running != ident {
                    panic!("Cannot stop {:}. {:} is currently running", ident, running)
                }
            }
            None => {}
        }

        self.profile(&ident.to_string()).stop();
        self.running = None;
        self.continue_running_paused();
    }

    fn profile(&mut self, ident: &str) -> &mut TimeElapsed {
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
        if ident == "main" {
            return;
        }

        self.profile(&ident.to_string()).stop();
        self.paused.push_front(ident.to_string());
    }

    fn pause_running(&mut self) -> () {
        match self.running.take() {
            Some(id) => self.pause(&id),
            None => {}
        };

        assert!(self.running.is_none());
    }

    fn continue_running_paused(&mut self) -> () {
        assert!(self.running.is_none());

        match self.paused.pop_front() {
            Some(ident) => self.start(&ident),
            None => {}
        }
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
    unsafe { println!("{}", ::once_cell::sync::Lazy::get(&TIMER).unwrap()) }
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

    #[test]
    fn test_timer_pause_complex() {
        let mut timer = Timer::new();

        assert_eq!(timer.running, None);

        let bar = "bar";
        let foo = "foo";
        let foo_inner = "foo_inner";
        let bar_inner = "bar_inner";

        let mut queue_test = VecDeque::new();

        fn start_fn(ident: &str, queue_test: &mut VecDeque<String>, timer: &mut Timer) -> () {
            queue_test.push_front(timer.running.clone().unwrap());
            timer.start(ident);
            assert_eq!(*queue_test, timer.paused);
            assert_eq!(timer.running, Some(ident.to_string()));
        }

        fn stop_fn(ident: &str, queue_test: &mut VecDeque<String>, timer: &mut Timer) -> () {
            timer.stop(ident);
            assert_eq!(timer.running, queue_test.pop_front());
            assert_eq!(*queue_test, timer.paused);
        }

        timer.start("main");

        timer.start(foo);
        assert_eq!(timer.running, Some(foo.to_string()));

        start_fn(foo_inner, &mut queue_test, &mut timer);
        stop_fn(foo_inner, &mut queue_test, &mut timer);

        start_fn(bar, &mut queue_test, &mut timer);

        start_fn(bar_inner, &mut queue_test, &mut timer);
        stop_fn(bar_inner, &mut queue_test, &mut timer);

        stop_fn(bar, &mut queue_test, &mut timer);

        timer.stop(foo);
        timer.stop("main");
    }
}
