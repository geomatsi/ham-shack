//! Status display support.
//!
//! A backend is selected at build time by feature. Every backend module exports
//! the same two items — `Display` and `create` — which this module re-exports,
//! so the application never names a concrete backend, bus or panel.
//!
//! `&DisplayInfo` is the whole interface. How a backend turns it into something
//! visible is entirely its own business: panel geometry, fonts, line order and
//! truncation all stay private to its module.
//!
//! The beacon's own `Status` deliberately does not come through here: it also
//! carries the encoded transmit message, which no panel has any business
//! seeing, and which would make every snapshot and redraw check 17x larger.

use crate::beacon::status::DisplayInfo;

use stm32f1xx_hal::{gpio, pac};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayError {
    /// Transport (I2C/SPI/GPIO) failure while updating the panel.
    Bus,
}

pub trait StatusDisplay {
    /// Render `info`.
    ///
    /// Errors are reported by the caller and otherwise ignored: a dead panel
    /// must never stop the beacon from transmitting.
    fn show(&mut self, info: &DisplayInfo) -> Result<(), DisplayError>;
}

/// Peripherals `init()` hands to whichever backend was built.
///
/// The shape is fixed so the call site in `init()` reads the same for I2C, SPI,
/// parallel and headless backends. Each backend consumes the subset it needs
/// and drops the rest.
///
/// Only PA0-PA5 are handed over — what the widest backend actually wires up
/// (LCD1602 in 4-bit mode) — plus the `crl` token that configures them. Pin
/// mode changes need `&mut Cr`, and that token cannot be shared, so it moves
/// here whole; that is also why the pins come one by one rather than as a split
/// `Parts`. PA6-PA7 are free and PA8-PA15 stay with `init()`, which needs PA15
/// to hand to `disable_jtag()` — that is what frees PB3 for the calibration
/// input. Nothing else on this board uses PA0-PA5: LED is PC13, PPS is PB1,
/// GPS is PB10/PB11. A backend wanting more pins takes PA6/PA7 by adding them
/// back here; one needing a pin outside CRL is the signal to move the whole pin
/// map into one cfg-aware board module rather than to grow this struct.
///
/// TIM1 covers whichever time source a backend needs — a bit-bang bus clock
/// (`counter_hz`) or blocking waits (`delay_us`), never both at once so far.
/// SysTick is deliberately *not* in here: it stays free for the alternative
/// `cortex-m-systick` monotonic. A backend that genuinely needs two independent
/// time sources — an I2C-backpack LCD, say, which wants a bit-bang clock *and*
/// HD44780 timing — is the point at which to add SYST or another timer.
///
/// No bus peripheral and no AFIO either. GPIOA's low half has pins to spare for
/// bit-banging any reasonable panel, so SPI1/SPI2 stay reserved as RTIC async
/// dispatchers (see the `#[rtic::app]` attribute), and nothing here needs pin
/// remapping.
/// AFIO would also be the one field that has to be borrowed rather than owned —
/// it is shared with the PPS EXTI setup and the USART3 remap — which would force
/// a lifetime parameter onto this struct. Add what a real backend actually asks
/// for, when it asks: backends destructure with `..`, so a new field costs
/// nothing to the ones that ignore it.
pub struct DisplayParts {
    pub crl: gpio::Cr<'A', false>,
    pub pa0: gpio::gpioa::PA0,
    pub pa1: gpio::gpioa::PA1,
    pub pa2: gpio::gpioa::PA2,
    pub pa3: gpio::gpioa::PA3,
    pub pa4: gpio::gpioa::PA4,
    pub pa5: gpio::gpioa::PA5,
    pub tim: pac::TIM1,
}

#[cfg(any(
    all(feature = "display-ssd1306", feature = "display-lcd1602"),
    all(feature = "display-ssd1306", feature = "display-rtt"),
    all(feature = "display-lcd1602", feature = "display-rtt"),
))]
compile_error!("at most one display backend may be selected");

// The `not(...)` guards keep exactly one backend selected even when several
// features are on, so the `compile_error!` above is the only diagnostic instead
// of a pile of duplicate-definition noise behind it.

#[cfg(feature = "display-ssd1306")]
mod ssd1306;
#[cfg(feature = "display-ssd1306")]
pub use self::ssd1306::{Display, create};

#[cfg(all(feature = "display-lcd1602", not(feature = "display-ssd1306")))]
mod lcd1602;
#[cfg(all(feature = "display-lcd1602", not(feature = "display-ssd1306")))]
pub use self::lcd1602::{Display, create};

#[cfg(all(
    feature = "display-rtt",
    not(any(feature = "display-ssd1306", feature = "display-lcd1602"))
))]
mod rtt;
#[cfg(all(
    feature = "display-rtt",
    not(any(feature = "display-ssd1306", feature = "display-lcd1602"))
))]
pub use self::rtt::{Display, create};

#[cfg(not(any(
    feature = "display-ssd1306",
    feature = "display-lcd1602",
    feature = "display-rtt"
)))]
mod null;
#[cfg(not(any(
    feature = "display-ssd1306",
    feature = "display-lcd1602",
    feature = "display-rtt"
)))]
pub use self::null::{Display, create};
