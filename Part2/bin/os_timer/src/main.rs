use util::counter::*;

fn main() {
    let freq = os_freq();

    println!("    OS Freq: {}", freq);

    let os_start = read_os_timer();
    let mut os_elapsed = 0;
    let mut os_now = 0;

    while os_elapsed < freq {
        os_now = read_os_timer();
        os_elapsed = os_now - os_start;
    }

    println!("   OS Timer: {} -> {} = {}", os_start, os_now, os_elapsed);
    println!(" OS Seconds: {}", os_elapsed / freq);
}
