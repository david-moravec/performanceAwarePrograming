pub mod counter;

use counter::read_os_timer;

use once_cell::sync::Lazy;
use std::collections::HashMap;

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

    fn elapsed(&self) -> u64 {
        self.stop.unwrap() - self.start
    }
}

pub struct Timer {
    profiles: HashMap<String, Vec<TimeElapsed>>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            profiles: HashMap::new(),
        }
    }
    pub fn start(&mut self, ident: String) -> () {
        match self.profiles.get_mut(&ident) {
            Some(profile) => profile.push(TimeElapsed::new()),
            None => {
                self.profiles.insert(ident, vec![TimeElapsed::new()]);
            }
        }
    }

    pub fn stop(&mut self, ident: &String) -> () {
        self.profiles
            .get_mut(ident)
            .expect("Should always start before stopping")
            .last_mut()
            .expect("Should always start before stopping")
            .stop()
    }
}

pub static mut TIMER: Lazy<Timer> = Lazy::new(Timer::new);
