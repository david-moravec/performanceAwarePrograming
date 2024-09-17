pub use timer::{print_timer, TIMER};
pub use timer_internal::time_it;

#[macro_export]
macro_rules! time_block {
    ($ident: expr, $block: block) => {
        unsafe {
            ::timer_macros::TIMER.start($ident);
        }
        $block;
        unsafe {
            ::timer_macros::TIMER.stop($ident);
        }
    };
    ($ident: expr, $bytes_processed: expr, $block: block) => {
        unsafe {
            ::timer_macros::TIMER.start($ident);
            ::timer_macros::TIMER.add_bytes_processed($ident, $bytes_processed);
        }
        $block;
        unsafe {
            ::timer_macros::TIMER.stop($ident);
        }
    };
}
