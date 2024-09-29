use core::fmt;
use std::time::{Duration, Instant};

enum TestStatus {
    UnInitialized,
    Testing,
    Completed,
    Error,
}

struct TestResults {
    count: u64,
    total_time: Duration,
    min: Duration,
    max: Duration,
}

impl TestResults {
    pub fn new() -> Self {
        TestResults {
            count: 0,
            total_time: Duration::new(0, 0),
            min: Duration::new(0, 0),
            max: Duration::new(0, 0),
        }
    }

    pub fn write(&mut self, test_result: Duration) -> bool {
        self.count += 1;
        self.total_time += test_result;

        if self.min > test_result {
            self.min = test_result;
            println!("New min: {:?}", self.min);
            return true;
        } else if self.max < test_result {
            self.max = test_result;
            return false;
        } else {
            return false;
        }
    }
}

struct RepTest {
    timer: Instant,
    try_for: Duration,
    results: TestResults,
    target_bytes: usize,
    total_bytes_processed: usize,
    status: TestStatus,
    name: String,
}

impl RepTest {
    pub fn new(target_bytes: usize, try_for: Duration, name: &str) -> Self {
        RepTest {
            timer: Instant::now(),
            results: TestResults::new(),
            try_for,
            target_bytes,
            total_bytes_processed: 0,
            status: TestStatus::UnInitialized,
            name: name.to_string(),
        }
    }

    pub fn start(&mut self, target_bytes: usize) -> () {
        if self.target_bytes != target_bytes {
            self.status = TestStatus::Error;
            return;
        }

        self.timer = Instant::now();
        self.status = TestStatus::Testing;
        self.total_bytes_processed += target_bytes
    }

    pub fn stop(&mut self) -> () {
        let min_overwritten = self.results.write(self.timer.elapsed());

        // restart timer
        if min_overwritten {
            self.timer = Instant::now()
        }
    }

    pub fn is_testing(&mut self) -> bool {
        match self.status {
            TestStatus::Testing => {
                if self.try_for < self.timer.elapsed() {
                    self.status = TestStatus::Completed;
                    return false;
                } else {
                    eprintln!(
                        "Time till the end {:?}",
                        self.try_for - self.timer.elapsed()
                    );
                    return true;
                }
            }
            TestStatus::Error => {
                eprintln!("Error during testing Occured");
                return false;
            }
            TestStatus::UnInitialized => return true,
            TestStatus::Completed => {
                println!("{:}", self);
                return false;
            }
        }
    }
}

impl fmt::Display for RepTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "___ {:} ___", self.name);
        writeln!(f, "Min: {:?}", self.results.min);
        writeln!(f, "Max: {:?}", self.results.min);
        Ok(())
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut tester = RepTest::new(0, Duration::new(10, 0), "test");
        let mut result: usize = 0;

        while tester.is_testing() {
            tester.start(0);
            result = add(2, 2);
            tester.stop()
        }

        assert_eq!(result, 4);
    }
}
