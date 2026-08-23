//! Beacon configuration: every knob in one place, as a tree of plain values.

use si5351::{ClockOutput, DriveStrength, Frequency, PLL};

/// The build-time configuration of this beacon.
pub const CFG: Config = Config {
    ham: Ham {
        callsign: "R1BRL",
        wspr_dial: Frequency::from_hz(14_095_600),
        pwr: 37,
    },
    sw: Sw {
        disp: Disp { poll_ms: 250 },
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

/// Total MultiSynth divider the parked PLL implies for the dial frequency.
const PLL_DIAL_DIV: u64 = CFG.hw.rf.pll_parked.as_microhz() / CFG.ham.wspr_dial.as_microhz();

// The PLL is never retuned for a transmission, so the two must stay in step.
// AN619 2.1.1: a fractional MultiSynth divider is only legal from 8 to 2048.
const _: () = assert!(
    PLL_DIAL_DIV >= 8 && PLL_DIAL_DIV <= 2048,
    "hw.rf.pll_parked is not a usable multiple of ham.wspr_dial"
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
    pub wspr_dial: Frequency,
    pub pwr: u8,
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
    /// PLL is parked here at init, before any output is enabled. It is
    /// 62x the centre of the 20m WSPR passband (dial + 1500 Hz audio offset),
    /// which puts it inside the 600-900 MHz VCO range and lets both outputs be
    /// derived from this one PLL with integer-ish multisynth dividers.
    pub pll_parked: Frequency,
    /// Output drive strength
    pub drive: DriveStrength,
}

/// Software configuration.
#[derive(Debug, Clone, Copy)]
pub struct Sw {
    pub disp: Disp,
}

/// Display misc configuration
#[derive(Debug, Clone, Copy)]
pub struct Disp {
    /// Display poll period
    pub poll_ms: u64,
}
