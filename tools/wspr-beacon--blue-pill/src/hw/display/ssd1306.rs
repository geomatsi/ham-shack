//! SSD1306 128x64 OLED backend.
//!
//! Wiring: bit-banged I2C on PA0 (SCL) / PA1 (SDA), TIM2 as the bus clock
//! source. `Eh1BitBangI2c` bridges bitbang-hal 0.3 (embedded-hal 0.2) up to the
//! embedded-hal 1.0 bus the `ssd1306` driver expects.
//!
//! Panel size, font and line layout are private to this module.

use super::{DisplayError, DisplayParts, StatusDisplay};
use crate::beacon::status::Status;
use crate::support::bitbang_i2c_compat::Eh1BitBangI2c;

use bitbang_hal::i2c::I2cBB;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use stm32f1xx_hal::{
    gpio::{self, OpenDrain, Output},
    pac,
    prelude::*,
    rcc::Rcc,
    timer::CounterHz,
};

/// Bit-bang I2C clock.
const I2C_HZ: u32 = 100_000;
/// FONT_6X10 line pitch, one pixel of leading.
const LINE_H: i32 = 11;

type Bus = Eh1BitBangI2c<
    I2cBB<
        gpio::gpioa::PA0<Output<OpenDrain>>,
        gpio::gpioa::PA1<Output<OpenDrain>>,
        CounterHz<pac::TIM2>,
    >,
>;

type Panel = Ssd1306<I2CInterface<Bus>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>;

pub struct Display {
    panel: Panel,
}

pub fn create(p: DisplayParts, rcc: &mut Rcc) -> Option<Display> {
    let DisplayParts {
        mut gpioa, tim2, ..
    } = p;

    let scl = gpioa.pa0.into_open_drain_output(&mut gpioa.crl);
    let sda = gpioa.pa1.into_open_drain_output(&mut gpioa.crl);

    let mut clk = tim2.counter_hz(rcc);
    clk.start(I2C_HZ.Hz()).ok()?;

    let bus = Eh1BitBangI2c::new(I2cBB::new(scl, sda, clk));
    let mut panel = Ssd1306::new(
        I2CDisplayInterface::new(bus),
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    // A missing or wedged panel must not take the beacon down at boot.
    panel.init().ok()?;

    Some(Display { panel })
}

impl Display {
    fn line(&mut self, text: &str, y: i32) -> Result<(), DisplayError> {
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        Text::with_baseline(text, Point::new(0, y), style, Baseline::Top)
            .draw(&mut self.panel)
            .map(|_| ())
            .map_err(|_| DisplayError::Bus)
    }
}

impl StatusDisplay for Display {
    /// Repaint the whole panel.
    ///
    /// `flush()` only transmits the dirty rectangle, but `clear()` marks the
    /// entire framebuffer dirty, so this always pays the full-frame cost of
    /// ~213 ms on the bit-banged bus. That is acceptable while the panel only
    /// carries two lines of text that change a few times a minute.
    ///
    /// Anything updating at symbol rate — a TX progress bar, say — must not go
    /// through `clear()`: repainting just the affected region instead keeps the
    /// dirty rectangle small, which is ~28 ms for one full-width 8-pixel page
    /// and ~4 ms for a few columns.
    fn show(&mut self, status: &Status) -> Result<(), DisplayError> {
        self.panel
            .clear(BinaryColor::Off)
            .map_err(|_| DisplayError::Bus)?;

        self.line(status.state.as_str(), 0)?;

        if let Some(qth) = status.qth_str() {
            self.line(qth, LINE_H)?;
        }

        self.panel.flush().map_err(|_| DisplayError::Bus)
    }
}
