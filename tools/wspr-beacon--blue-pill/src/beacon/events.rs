use core::cmp::Ordering;

#[derive(Clone, Copy, Default)]
pub enum Event {
    /// Empty
    #[default]
    NIL,
    /// LED data
    LED,
    /// No GPS fix
    NOGPS,
    /// GPS data
    GPS((u8, u8), (u8, u8), (u8, u8, f32)),
    /// PPS data
    PPS,
}

impl Event {
    fn prio(self) -> u8 {
        match self {
            Event::NIL => 0u8,
            Event::LED => 10u8,
            Event::NOGPS => 20u8,
            Event::GPS(_, _, _) => 20u8,
            Event::PPS => 50u8,
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
        match self {
            Event::NIL => match other {
                Event::NIL => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::LED => match other {
                Event::LED => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::NOGPS => match other {
                Event::NOGPS => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::GPS(_, _, _) => match other {
                Event::GPS(_, _, _) => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
            Event::PPS => match other {
                Event::PPS => Ordering::Equal,
                _ => self.prio().cmp(&other.prio()),
            },
        }
    }
}
