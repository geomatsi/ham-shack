//! Fallback backend used when `display` is enabled without naming one.
//!
//! Claims nothing, draws nothing. Keeps `--features display` building and gives
//! host-side tests something to instantiate.

use super::{DisplayError, DisplayParts, StatusDisplay};
use crate::beacon::status::DisplayInfo;

use stm32f1xx_hal::rcc::Rcc;

pub struct Display;

pub fn create(_p: DisplayParts, _rcc: &mut Rcc) -> Option<Display> {
    Some(Display)
}

impl StatusDisplay for Display {
    fn show(&mut self, _info: &DisplayInfo) -> Result<(), DisplayError> {
        Ok(())
    }
}
