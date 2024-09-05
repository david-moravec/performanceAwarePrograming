use core::arch::x86_64::_rdtsc;
use std::mem;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

pub fn os_freq() -> u64 {
    unsafe {
        let mut freq = mem::zeroed();
        QueryPerformanceFrequency(&mut freq);
        *freq.QuadPart() as u64
    }
}

pub fn read_os_timer() -> u64 {
    unsafe {
        let mut counter = mem::zeroed();
        QueryPerformanceCounter(&mut counter);
        *counter.QuadPart() as u64
    }
}

pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub fn guess_cpu_freq(wait_for_ms: Option<u64>) -> u64 {
    let wait_for_ms = wait_for_ms.unwrap_or(1000);

    let cpu_start = read_cpu_timer();
    let os_freq = os_freq();
    let os_start = read_os_timer();
    let mut os_elapsed = 0;
    let mut os_now;
    let os_wait_time = (os_freq as f64 * (wait_for_ms as f64 / 1000 as f64)) as u64;

    while os_elapsed < os_wait_time {
        os_now = read_os_timer();
        os_elapsed = os_now - os_start;
    }

    let cpu_elapsed = read_cpu_timer() - cpu_start;

    os_freq / os_elapsed * cpu_elapsed
}

pub fn ts_ratio(t1: u64, t2: u64) -> f64 {
    t1 as f64 / t2 as f64 * 100.0
}

pub fn format_ts_output(label: &str, ts_elapsed: u64, ts_total: u64) -> String {
    let ratio = ts_ratio(ts_elapsed, ts_total);

    format!("  {}: {} ({:.2}%)", label, ts_elapsed, ratio)
}
