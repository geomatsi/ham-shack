//! Headless backend: types `Status` into the RTT log.
//!
//! Claims no pins, so it works on any board and is handy for bring-up when no
//! panel is attached. It also demonstrates that the `StatusDisplay` contract
//! carries no notion of geometry: this backend renders one line of text per
//! update and picks apart `Status` exactly as it pleases.
//!
//! Requires the `rtt-log` feature, which `display-rtt` pulls in.

use super::{DisplayError, DisplayParts, StatusDisplay};
use crate::beacon::states::State;
use crate::beacon::status::Status;
use crate::wspr_log;

use rtt_target::rprintln;
use stm32f1xx_hal::rcc::Rcc;

pub struct Display;

pub fn create(_p: DisplayParts, _rcc: &mut Rcc) -> Option<Display> {
    Some(Display)
}

impl StatusDisplay for Display {
    fn show(&mut self, status: &Status) -> Result<(), DisplayError> {
        let qth = status.qth_str().unwrap_or("----");

        match status.state {
            State::Error(code) => wspr_log!("DISP: {} [{}]", status.state.as_str(), code.as_str()),
            _ => wspr_log!("DISP: {} qth {}", status.state.as_str(), qth),
        }

        Ok(())
    }
}
