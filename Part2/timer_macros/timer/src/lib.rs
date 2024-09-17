pub mod counter;

use counter::{read_os_timer, ts_ratio};

use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::fmt::Display;


#[derive(Debug)]
pub struct TimeAnchor {
    hit_count: u64,
    elapsed_acc: u64,
    elapsed_children_acc: u64,
    elapsed: TimeElapsed,
    elapsed_children: TimeElapsed,
    processed_bytes: u64,
}

impl TimeAnchor {
    pub fn new() -> Self {
        TimeAnchor {
            hit_count: 1,
            elapsed_acc: 0,
            elapsed_children_acc: 0,
            elapsed: TimeElapsed::new(),
            elapsed_children: TimeElapsed { start: 0 },
            processed_bytes: 0,
        }
    }

    pub fn start(&mut self) -> () {
        self.elapsed.start();
        self.hit_count += 1
    }

    pub fn stop(&mut self) -> () {
        self.elapsed_acc += self.elapsed.stop();
    }

    pub fn pause(&mut self) -> () {
        self.stop();
        self.elapsed_children.start();
    }

    pub fn continue_run(&mut self) -> () {
        self.elapsed_children_acc += self.elapsed_children.stop();
        self.elapsed.start()
    }

    pub fn start_children(&mut self) -> () {
        self.elapsed_children.start()
    }

    pub fn stop_children(&mut self) -> () {
        self.elapsed_children_acc += self.elapsed_children.stop();
    }

    pub fn elapsed_w_children(&self) -> u64 {
        self.elapsed_acc + self.elapsed_children_acc
    }
}

#[derive(Debug)]
pub struct TimeElapsed {
    start: u64,
}

impl TimeElapsed {
    pub fn new() -> Self {
        TimeElapsed {
            start: read_os_timer(),
        }
    }

    pub fn stop(&mut self) -> u64 {
        let stop = read_os_timer();
        let elapsed = stop - self.start;
        self.start = 0;

        elapsed
    }

    pub fn start(&mut self) -> () {
        self.start = read_os_timer();
    }
}

pub struct Timer {
    anchors: HashMap<String, TimeAnchor>,
    paused: VecDeque<String>,
    running: Option<String>,
    start_main: u64,
    stop_main: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            anchors: HashMap::new(),
            paused: VecDeque::new(),
            running: None,
            start_main: 0,
            stop_main: 0,
        }
    }

    pub fn start_main(&mut self) -> () {
        self.start_main = read_os_timer();
    }

    pub fn stop_main(&mut self) -> () {
        self.stop_main = read_os_timer();
    }

    pub fn main_duration_ms(&self, freq: f64) -> f64 {
        let elapsed_main = self.stop_main - self.start_main;
        elapsed_main as f64 / freq as f64 * 1000.0
    }

    pub fn start(&mut self, ident: &str) -> () {
        let ident = ident.to_string();

        self.pause_running();
        self.running = Some(ident.clone());

        match self.anchors.get_mut(&ident) {
            Some(anchor) => anchor.start(),
            None => {
                self.anchors.insert(ident, TimeAnchor::new());
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

        self.anchor_mut(&ident.to_string()).stop();
        self.running = None;
        self.continue_running_paused();
    }

    fn anchor_mut(&mut self, ident: &str) -> &mut TimeAnchor {
        self.anchors
            .get_mut(ident)
            .expect(&format!("Profile for {:} does not exists", ident))
    }

    fn anchor(&self, ident: &str) -> &TimeAnchor {
        self.anchors
            .get(ident)
            .expect(&format!("Profile for {:} does not exists", ident))
    }

    fn elapsed_total_ts(&self, ident: &str) -> u64 {
        self.anchor(ident).elapsed_acc
    }

    fn elapsed_total_ts_w_children(&self, ident: &str) -> u64 {
        self.anchor(ident).elapsed_w_children()
    }

    fn pause(&mut self, ident: &str) -> () {
        if ident == "main" {
            return;
        }

        self.anchor_mut(&ident.to_string()).pause();
        self.paused.push_front(ident.to_string());
    }

    pub fn add_bytes_processed(&mut self, ident: &str, byte_count: usize) -> () {
        self.anchor_mut(ident).processed_bytes += byte_count as u64;
    }

    fn hit_count(&self, ident: &str) -> u64 {
        self.anchor(ident).hit_count
    }

    fn elapsed_total_main(&self) -> u64 {
        self.stop_main - self.start_main
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
            Some(ident) => {
                self.anchor_mut(&ident).continue_run();
                self.running = Some(ident.to_string());
            }
            None => {}
        }
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let freq = 2.1e9;
        writeln!(f, "\nCPU Freq: {:}", freq)?;
        writeln!(f, "\nTotal time: {:?}ms", self.main_duration_ms(freq))?;
        for ident in self.anchors.keys() {
            if ident != "main" {
                write!(
                    f,
                    "{}[{:}]: {} ({:.2}%",
                    ident,
                    self.hit_count(ident),
                    self.elapsed_total_ts(ident),
                    ts_ratio(self.elapsed_total_ts(ident), self.elapsed_total_main()),
                )?;
            }

            if self.anchor(ident).elapsed_children_acc != 0 {
                write!(
                    f,
                    ", {:.2} w/children",
                    ts_ratio(
                        self.elapsed_total_ts_w_children(ident),
                        self.elapsed_total_main()
                    )
                )?;
            };

            if self.anchor(ident).processed_bytes > 0 {
                let megabyte: f64 = 1024.0 * 1024.0;
                let gigabyte: f64 = 1024.0 * megabyte;

                let seconds = 1000.0 * self.elapsed_total_ts_w_children(ident) as f64 / freq as f64;
                let bytes_per_sec = self.anchor(ident).processed_bytes as f64 / seconds;
                let megabytes = self.anchor(ident).processed_bytes as f64 / megabyte;
                let gigebytes_per_sec = bytes_per_sec / gigabyte;

                write!(f, ", {:.3}mb at {:.2}gb/s", megabytes, gigebytes_per_sec)?;
            }

            writeln!(f, ")")?;
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

        assert!(timer.elapsed_total_ts("foo") > 0)
    }

    #[test]
    fn test_timer_complex() {
        let mut timer = Timer::new();

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        let elapsed_1 = timer.elapsed_total_ts("foo");

        timer.start("foo");
        sleep(Duration::new(0, 1000));
        timer.stop("foo");

        assert!(elapsed_1 < timer.elapsed_total_ts("foo"));
    }

    #[test]
    fn test_timer_pause() {
        let mut timer = Timer::new();

        assert_eq!(timer.running, None);

        timer.start("foo");
        assert_eq!(timer.running, Some("foo".to_string()));

        timer.start("foofoo");
        let elapsed_foo_1 = timer.elapsed_total_ts("foo");
        assert_eq!(timer.running, Some("foofoo".to_string()));

        timer.stop("foofoo");
        assert_eq!(timer.running, Some("foo".to_string()));
        timer.stop("foo");

        assert!(elapsed_foo_1 < timer.elapsed_total_ts("foo"))
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
