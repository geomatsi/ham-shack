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

/* DWT cycle-counter profiling, compiled out unless `dwt-profile` */

/// Cycles to microseconds
///
/// `sysclk_mhz` is by definition cycles per microsecond, so the division is
/// the whole conversion. Kept a `const fn` rather than a macro so it is
/// type-checked once instead of at every expansion.
#[inline(always)]
pub const fn dwt_us(cycles: u32) -> u32 {
    cycles / crate::beacon::config::CFG.hw.mcu.sysclk_mhz
}

/// Absolute timestamp: `"<label>: DWT <n> ms"`.
///
/// A coarse timeline marker, not a measurement - `cycle_count` is 32 bits and
/// wraps every ~134 s at 32 MHz, so the value is modulo that.
#[macro_export]
macro_rules! dwt_stamp {
    ($label:literal) => {{
        #[cfg(feature = "dwt-profile")]
        ::rtt_target::rprintln!(
            concat!($label, ": DWT {} ms"),
            $crate::beacon::logging::dwt_us(::cortex_m::peripheral::DWT::cycle_count()) / 1_000
        );
    }};
}

/// Opens a span, binding `$name` in the enclosing scope. Without the feature
/// it expands to nothing, so `$name` does not exist - only `dwt_since!` may
/// read it.
#[macro_export]
macro_rules! dwt_start {
    ($name:ident) => {
        #[cfg(feature = "dwt-profile")]
        let $name = ::cortex_m::peripheral::DWT::cycle_count();
    };
}

/// Closes a span opened by `dwt_start!`: `"<label>: <n> us"`. `wrapping_sub`
/// makes it correct across a counter wrap for any span under ~134 s.
#[macro_export]
macro_rules! dwt_since {
    ($label:literal, $start:ident) => {{
        #[cfg(feature = "dwt-profile")]
        ::rtt_target::rprintln!(
            concat!($label, ": {} us"),
            $crate::beacon::logging::dwt_us(
                ::cortex_m::peripheral::DWT::cycle_count().wrapping_sub($start)
            )
        );
    }};
}
