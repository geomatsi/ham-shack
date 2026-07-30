//! LCD1602 (HD44780) backend, 4-bit mode.
//!
//! Wiring, all on GPIOA:
//!
//! | signal | pin |
//! |--------|-----|
//! | RS     | PA0 |
//! | EN     | PA1 |
//! | D4     | PA2 |
//! | D5     | PA3 |
//! | D6     | PA4 |
//! | D7     | PA5 |
//!
//! R/W is tied low on the panel — the driver never reads back — and D0..D3 are
//! left unconnected in 4-bit mode. PA0/PA1 overlap with the SSD1306 backend's
//! bit-banged I2C, which is fine because the two backends are mutually
//! exclusive.
//!
//! `hd44780-driver` 0.4 is built on embedded-hal 0.2, and the HAL implements the
//! 0.2 pin and delay traits as well, so no compat shim is needed here (unlike
//! `Eh1BitBangI2c`, which exists only because ssd1306 0.10 wants eh 1.0).
//!
//! TIM2 is the time source: the driver needs blocking waits at both µs and ms
//! scale, and `Delay<TIM2, 1_000_000>` covers the whole range. The SSD1306
//! backend spends the same timer on its bit-bang I2C clock instead — only one
//! backend is built at a time, so they never contend.
//!
//! Panel geometry and line layout are private to this module.

use super::{DisplayError, DisplayParts, StatusDisplay};
use crate::beacon::status::Status;

use hd44780_driver::bus::FourBitBus;
use hd44780_driver::{Cursor, CursorBlink, Display as PanelPower, DisplayMode, HD44780};
use stm32f1xx_hal::{
    gpio::{self, Output, PushPull},
    pac,
    prelude::*,
    rcc::Rcc,
    timer,
};

/// Character columns per row.
const COLS: usize = 16;
/// DDRAM address of the first cell of each row. The 1602 controller keeps the
/// second row at 0x40, not immediately after the first.
const ROW_ADDR: [u8; 2] = [0x00, 0x40];

type Pin<const N: u8> = gpio::Pin<'A', N, Output<PushPull>>;

type Panel = HD44780<FourBitBus<Pin<0>, Pin<1>, Pin<2>, Pin<3>, Pin<4>, Pin<5>>>;

/// Microsecond-resolution blocking delay off TIM2. The driver asks for both
/// `DelayUs<u16>` and `DelayMs<u8>`; a 1 MHz tick serves both.
type Delay = timer::DelayUs<pac::TIM2>;

pub struct Display {
    panel: Panel,
    delay: Delay,
    /// Last status put on the panel, for the per-row skip in `show`.
    shown: Option<Status>,
}

pub fn create(p: DisplayParts, rcc: &mut Rcc) -> Option<Display> {
    let DisplayParts {
        mut gpioa, tim2, ..
    } = p;

    let mut delay = tim2.delay_us(rcc);

    let rs = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
    let en = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);
    let d4 = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);
    let d5 = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    let d6 = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let d7 = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);

    // A missing or wedged panel must not take the beacon down at boot. The
    // driver runs the 4-bit init sequence here, including the power-on wait.
    let mut panel = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).ok()?;

    // The driver defaults to a visible blinking cursor, which is noise on a
    // status readout.
    panel
        .set_display_mode(
            DisplayMode {
                display: PanelPower::On,
                cursor_visibility: Cursor::Invisible,
                cursor_blink: CursorBlink::Off,
            },
            &mut delay,
        )
        .ok()?;
    panel.clear(&mut delay).ok()?;

    Some(Display {
        panel,
        delay,
        shown: None,
    })
}

impl Display {
    /// Write one full row, truncated and space-padded to `COLS`.
    ///
    /// Padding rather than `clear()` on every update: the panel has no
    /// framebuffer, so short text would otherwise leave stale characters
    /// behind, and clearing the whole panel first makes the readout flicker.
    fn row(&mut self, row: usize, text: &str) -> Result<(), DisplayError> {
        self.panel
            .set_cursor_pos(ROW_ADDR[row], &mut self.delay)
            .map_err(|_| DisplayError::Bus)?;

        let mut written = 0;

        for c in text.chars().take(COLS) {
            // The controller's character ROM is byte-oriented, so anything
            // outside ASCII would be mangled on the way out.
            let c = if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                '?'
            };

            self.panel
                .write_char(c, &mut self.delay)
                .map_err(|_| DisplayError::Bus)?;
            written += 1;
        }

        for _ in written..COLS {
            self.panel
                .write_char(' ', &mut self.delay)
                .map_err(|_| DisplayError::Bus)?;
        }

        Ok(())
    }
}

impl StatusDisplay for Display {
    /// Rewrite only the rows whose contents changed.
    ///
    /// The driver holds E high for 2 ms per nibble, so a character costs ~4.1 ms
    /// and a full 2x16 refresh blocks for ~139 ms. Skipping untouched rows keeps
    /// the common single-field update to roughly half of that.
    fn show(&mut self, status: &Status) -> Result<(), DisplayError> {
        let shown = self.shown;

        if shown.map(|shown| shown.state) != Some(status.state) {
            self.row(0, status.state.as_str())?;
        }

        if shown.map(|shown| shown.qth) != Some(status.qth) {
            self.row(1, status.qth_str().unwrap_or(""))?;
        }

        // Only after both rows are on the panel: a partial failure leaves the
        // cache stale so the next call rewrites everything.
        self.shown = Some(*status);

        Ok(())
    }
}
