use core::arch::x86_64::_rdtsc;
use std::mem;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

pub fn freq() -> u64 {
    unsafe {
        let mut freq = mem::zeroed();
        QueryPerformanceFrequency(&mut freq);
        *freq.QuadPart() as u64
    }
}

pub fn counter() -> u64 {
    unsafe {
        let mut counter = mem::zeroed();
        QueryPerformanceCounter(&mut counter);
        *counter.QuadPart() as u64
    }
}

pub fn timer() -> u64 {
    unsafe { _rdtsc() }
}
