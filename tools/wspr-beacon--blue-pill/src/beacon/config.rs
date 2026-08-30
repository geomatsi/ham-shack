//! Beacon configuration: every knob in one place, as a tree of plain values.

use si5351::{ClockOutput, DriveStrength, Frequency, PLL};

/// Bands this beacon is built for, by USB dial frequency, ascending.
///
/// These are the standard WSPR dial frequencies; worth a sanity check against
/// your own reference before the first transmission on a band.
pub const BANDS: [Band; 8] = [
    Band::new(3_568_600, "3.5MHz / 80m"),
    Band::new(7_038_600, "7MHz / 40m"),
    Band::new(10_138_700, "10MHz / 30m"), // WARC
    Band::new(14_095_600, "14MHz / 20m"),
    Band::new(18_104_600, "18MHz / 17m"), // WARC
    Band::new(21_094_600, "21MHz / 15m"),
    Band::new(24_924_600, "24MHz / 12m"), // WARC
    Band::new(28_124_600, "28MHz / 10m"),
];

/// The build-time configuration of this beacon.
pub const CFG: Config = Config {
    ham: Ham {
        callsign: "R1BRL",
        bands: BANDS,
        band: 3, // 20m
        pwr: 27,
        tx_period_min: 10,
    },
    sw: Sw {
        disp: Disp { poll_ms: 250 },
        wdg: Watchdog {
            period_ms: 10000,
            feed_ms: 2000,
        },
    },
    hw: Hw {
        mcu: Mcu {
            crystal_mhz: 8,
            sysclk_mhz: 32,
            pclk_mhz: 16,
            i2c_khz: 400,
        },
        gps: Gps {
            baudrate: 9600,
            ublox_len: 2048,
        },
        rf: Rf {
            wspr_clk: ClockOutput::Clk0,
            calib_clk: ClockOutput::Clk1,
            nominal: Frequency::from_hz(10_000_000),
            pll: PLL::A,
            pll_parked: Frequency::from_hz(62 * 14_097_100),
            drive: DriveStrength::_8mA,
        },
    },
};

/// Whether every band divides legally out of the parked PLL.
///
/// AN619 2.1.1: a fractional MultiSynth divider is only legal from 8 to 2048.
/// The +1.4..1.6 kHz audio offset shifts the divider by under 10 ppm, so the
/// dial settles the whole sub-band.
const fn bands_fit(pll: Frequency, bands: &[Band]) -> bool {
    let mut i = 0;
    while i < bands.len() {
        let div = pll.as_microhz() / bands[i].dial.as_microhz();
        if div < 8 || div > 2048 {
            return false;
        }
        i += 1;
    }
    true
}

// The PLL is never retuned for a transmission, so it has to serve every band.
const _: () = assert!(
    bands_fit(CFG.hw.rf.pll_parked, &CFG.ham.bands),
    "hw.rf.pll_parked does not serve every band in ham.bands"
);

// WSPR frames start on even UTC minutes; dividing 60 keeps the cadence
// unbroken across the hour.
const _: () = assert!(
    CFG.ham.tx_period_min.is_multiple_of(2) && 60u8.is_multiple_of(CFG.ham.tx_period_min),
    "ham.tx_period_min must be even and divide 60"
);

// A frame is 110.6 s, so ten minutes is ~18% duty - about WSJT-X's default Tx
// fraction of 20%.
const _: () = assert!(
    CFG.ham.tx_period_min >= 10,
    "ham.tx_period_min below 10 is an impolite duty cycle"
);

// IWDG ceiling: /256 prescaler, 12-bit reload, 40 kHz nominal LSI (RM0008
// Table 96). stm32f1xx-hal panics inside `start()` above this, and since the
// period is a constant that folds to an unconditional panic at boot - leaving
// a stripped binary that halts with the watchdog never started.
const _: () = assert!(
    CFG.sw.wdg.period_ms <= 26_214,
    "sw.wdg.period_ms exceeds the IWDG maximum period"
);

// The HAL sizes the reload for 40 kHz, but LSI is 30-60 kHz (RM0008 7.2.5),
// so the real timeout can be as short as 2/3 of period_ms. Feed twice per that
// at the fast corner, or a fast part reboot-loops and a slow one never does.
const _: () = assert!(
    (CFG.sw.wdg.feed_ms as u32) * 2 <= CFG.sw.wdg.period_ms * 2 / 3,
    "sw.wdg.feed_ms leaves no margin at the fast LSI corner"
);

const _: () = assert!(
    CFG.ham.band < CFG.ham.bands.len(),
    "ham.band is out of range of ham.bands"
);

/// Root of the configuration tree.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// Station identity: who and what is transmitting.
    pub ham: Ham,
    /// Misc software configuration
    pub sw: Sw,
    /// The board this firmware runs on.
    pub hw: Hw,
}

/// Station identity.
#[derive(Debug, Clone, Copy)]
pub struct Ham {
    pub callsign: &'static str,
    /// Bands this beacon is built for, ascending.
    pub bands: [Band; BANDS.len()],
    /// Index into `bands` of the band in use. Reordering `BANDS` moves it -
    /// keep the comment beside it honest. Becomes a per-slot choice once the
    /// beacon rotates.
    pub band: usize,
    pub pwr: u8,
    /// Minutes between transmissions. Even, so frames land on the even UTC
    /// minutes receivers listen on; the lower bound is duty cycle, not the
    /// protocol.
    pub tx_period_min: u8,
}

/// One WSPR band, named by the USB dial frequency its sub-band is referenced
/// to. Transmissions sit in the 200 Hz window `dial + 1400 .. dial + 1600`.
#[derive(Debug, Clone, Copy)]
pub struct Band {
    pub dial: Frequency,
    /// How the band is named on a display.
    pub name: &'static str,
}

impl Band {
    /// A band from its USB dial frequency in hertz.
    pub const fn new(dial_hz: u32, name: &'static str) -> Self {
        Band {
            dial: Frequency::from_hz(dial_hz),
            name,
        }
    }
}

/// Board-level configuration.
#[derive(Debug, Clone, Copy)]
pub struct Hw {
    pub mcu: Mcu,
    pub gps: Gps,
    pub rf: Rf,
}

/// MCU and clock tree.
#[derive(Debug, Clone, Copy)]
pub struct Mcu {
    /// Crystal freq in MHz
    pub crystal_mhz: u32,
    /// SYSCLK freq in MHz
    pub sysclk_mhz: u32,
    /// PCLK freq in MHz
    pub pclk_mhz: u32,
    /// i2c freq in kHz
    pub i2c_khz: u32,
}

/// GPS receiver.
#[derive(Debug, Clone, Copy)]
pub struct Gps {
    /// GPS module UART bauderate
    pub baudrate: u32,
    /// Size of one half of the double-buffered NMEA DMA buffer,
    /// two of these are allocated statically.
    pub ublox_len: usize,
}

/// Si5351 clock generator and the RF plan around it.
#[derive(Debug, Clone, Copy)]
pub struct Rf {
    /// Output driving the antenna during a transmission.
    pub wspr_clk: ClockOutput,
    /// Output fed back to the MCU timer input for crystal calibration.
    pub calib_clk: ClockOutput,
    /// Frequency `calib_clk` is programmed to while calibrating. The PPS-gated
    /// tick count is compared against this to get the crystal error in ppb, so
    /// it is also the expected count for one gate.
    pub nominal: Frequency,
    /// Common PLL for tx and calib
    pub pll: PLL,
    /// PLL is parked here at init, before any output is enabled, and is never
    /// retuned: transmissions move only the output MultiSynth. So it has to
    /// sit in the 600-900 MHz VCO range and divide legally into every band in
    /// `ham.bands` and into `nominal` - see `bands_fit`. Landing on an exact
    /// multiple of a band buys nothing; the 20-bit fractional divider holds
    /// every band to well under a millihertz either way.
    pub pll_parked: Frequency,
    /// Output drive strength
    pub drive: DriveStrength,
}

/// Software configuration.
#[derive(Debug, Clone, Copy)]
pub struct Sw {
    pub disp: Disp,
    pub wdg: Watchdog,
}

/// Display misc configuration
#[derive(Debug, Clone, Copy)]
pub struct Disp {
    /// Display poll period
    pub poll_ms: u64,
}

/// Watchdog misc configuration
#[derive(Debug, Clone, Copy)]
pub struct Watchdog {
    /// Watchdog period
    pub period_ms: u32,
    /// Watchdog feed period
    pub feed_ms: u64,
}
