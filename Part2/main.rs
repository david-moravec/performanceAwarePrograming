use timer_macros::time_it;

#[time_it]
fn main() {
    one()
}

#[time_it]
pub fn one() -> () {
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }

    eprintln!("{}", sum);
}
