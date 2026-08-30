use crate::beacon::config::CFG;
use crate::beacon::states::State;

/// Symbols in one WSPR transmission.
pub const WSPR_SYMBOLS: usize = 162;

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
/// - the encoded message waiting to go out
/// - the information worth showing
///
/// Deliberately not `Copy` and not `PartialEq`: the message alone is 163 bytes,
/// so a whole-struct copy or compare is never what the caller wants. Readers
/// take the field they need, and the display path takes a [`DisplayInfo`].
pub struct Status {
    pub state: State,
    pub qth: Option<[u8; 4]>,
    pub time: Option<Time>,
    pub msg: Option<[u8; WSPR_SYMBOLS]>,
    pub ppb: Option<i64>,
}

impl Status {
    /// Cold start: no fix, nothing to send, nothing to show.
    pub const fn new() -> Self {
        Status {
            state: State::GpsWait,
            qth: None,
            time: None,
            msg: None,
            ppb: None,
        }
    }

    /// Back to waiting for a fix, dropping everything derived from the old one.
    ///
    /// Every path out of the transmit cycle ends here — a lost fix, a completed
    /// transmission, an error — because all three leave the same three fields
    /// stale in the same way.
    ///
    /// `time` deliberately survives: the last UTC read from the GPS stays on the panel
    pub fn reset(&mut self) {
        self.state = State::GpsWait;
        self.qth = None;
        self.msg = None;
        self.ppb = None;
    }

    /// Snapshot of just the fields a display cares about.
    /// Taken under the `status` lock and handed out by value.
    pub fn display_info(&self) -> DisplayInfo {
        DisplayInfo {
            state: self.state,
            band: CFG.ham.bands[CFG.ham.band].name,
            qth: self.qth,
            time: self.time,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

/// Small subset of [`Status`] a display backend is shown.
#[derive(Clone, Copy, PartialEq)]
pub struct DisplayInfo {
    pub state: State,
    /// Build-time today; a per-slot choice once the beacon rotates bands.
    pub band: &'static str,
    pub qth: Option<[u8; 4]>,
    pub time: Option<Time>,
}

impl DisplayInfo {
    /// QTH square as text, once a fix has produced one.
    pub fn qth_str(&self) -> Option<&str> {
        self.qth
            .as_ref()
            .and_then(|qth| core::str::from_utf8(qth).ok())
    }
}
