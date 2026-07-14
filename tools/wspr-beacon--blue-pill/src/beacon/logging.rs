#[macro_export]
macro_rules! wspr_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "rtt-log")]
        rprintln!($($arg)*);
    };
}
