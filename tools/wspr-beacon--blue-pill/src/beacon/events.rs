use core::cmp::Ordering;
use core::fmt;

#[derive(Clone, Copy, Default)]
pub enum Event {
    /// Empty
    #[default]
    NIL,
    /// Multi-purpose heartbeat
    BEAT,
    /// No GPS fix
    NOGPS,
    /// GPS data
    GPS((f64, f64), (u8, u8, f32)),
    /// Tx result
    TXDONE,
    /// Calibration result
    CALIB(u32),
}

impl Event {
    fn prio(self) -> u8 {
        match self {
            Event::NIL => 0u8,
            Event::BEAT => 5u8,
            Event::NOGPS => 20u8,
            Event::GPS(_, _) => 20u8,
            Event::CALIB(_) => 30u8,
            Event::TXDONE => 30u8,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::NIL => write!(f, "Event::NIL"),
            Event::NOGPS => write!(f, "Event::NOGPS"),
            Event::GPS(_, _) => write!(f, "Event::GPS"),
            Event::CALIB(_) => write!(f, "Event::CALIB"),
            Event::TXDONE => write!(f, "Event::TXDONE"),
            Event::BEAT => write!(f, "Event::BEAT"),
        }
    }
}

impl Event {
    pub fn as_str(&self) -> &'static str {
        match self {
            Event::NIL => "Event::NIL",
            Event::NOGPS => "Event::NOGPS",
            Event::GPS(_, _) => "Event::GPS",
            Event::CALIB(_) => "Event::CALIB",
            Event::TXDONE => "Event::TXDONE",
            Event::BEAT => "Event::BEAT",
        }
    }
}

/* simple ordering of events based only on their priority */

impl Eq for Event {}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.prio() == other.prio()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.prio().cmp(&other.prio())
    }
}
