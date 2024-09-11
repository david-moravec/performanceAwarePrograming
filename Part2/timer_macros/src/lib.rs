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
}
