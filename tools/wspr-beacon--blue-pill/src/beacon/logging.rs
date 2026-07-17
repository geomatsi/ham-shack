#[macro_export]
macro_rules! wspr_log {
    ($($arg:tt)*) => {
        {
            #[cfg(feature = "rtt-log")]
            rprintln!($($arg)*);
            #[cfg(not(feature = "rtt-log"))]
            let _ = core::format_args!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! wspr_lognln{
    ($($arg:tt)*) => {
        {
            #[cfg(feature = "rtt-log")]
            rprint!($($arg)*);
            #[cfg(not(feature = "rtt-log"))]
            let _ = core::format_args!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! wspr_debug{
    ($($arg:tt)*) => {
        {
            #[cfg(feature = "rtt-log-debug")]
            rprintln!($($arg)*);
            #[cfg(not(feature = "rtt-log-debug"))]
            let _ = core::format_args!($($arg)*);
        }
    };
}
