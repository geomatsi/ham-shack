use crate::beacon::states::State;

/// UTC time of day from the GPS, hours and minutes.
/// Seconds are deliberately omitted to avoid frequent redraws.
#[derive(Clone, Copy, PartialEq)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
}

impl Time {
    /// Hand-rolled rather than `write!` not to pull `core::fmt` machinery
    pub fn hhmm<'a>(&self, buf: &'a mut [u8; 5]) -> &'a str {
        buf[0] = b'0' + (self.hours / 10) % 10;
        buf[1] = b'0' + self.hours % 10;
        buf[2] = b':';
        buf[3] = b'0' + (self.minutes / 10) % 10;
        buf[4] = b'0' + self.minutes % 10;

        // Every byte above is ASCII by construction, fallback is to keep this infallible without `unsafe`.
        core::str::from_utf8(buf).unwrap_or("--:--")
    }
}

/// Everything the beacon knows about itself:
/// - the state machine's current state
/// - the information worth showing
#[derive(Clone, Copy, PartialEq)]
pub struct Status {
    pub state: State,
    pub qth: Option<[u8; 4]>,
    pub time: Option<Time>,
}

impl Status {
    /// QTH square as text, once a fix has produced one.
    pub fn qth_str(&self) -> Option<&str> {
        self.qth
            .as_ref()
            .and_then(|qth| core::str::from_utf8(qth).ok())
    }
}
