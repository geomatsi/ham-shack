//! Calibration policy: what to do with each gated tick count.
//!
//! The driver owns the hardware and the PPS gate; this owns the decisions. It
//! is fed one tick count per gate and answers with the next action, so the
//! whole sequence - warmup, correct, settle, converge - is a plain state
//! machine with no I2C, no interrupts and no shared resources in it.

use si5351::{Frequency, calibrate};

/// Readings discarded after the generator is first programmed, while the
/// output settles.
const WARMUP_GATES: u8 = 2;
/// Upper bound on measurement gates, so calibration always terminates.
const MEASURE_GATES: u8 = 6;
/// Below this the dial is good enough and calibration stops early.
const TOLERANCE_PPB: i64 = 300;

/// What the driver must do before the next gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Nothing to do: keep gating.
    Continue,
    /// Program the generator to this dial frequency, then keep gating.
    SetDial(Frequency),
    /// Calibration is over: stop the generator. `Some(ppb)` is the crystal
    /// error to apply to later transmissions, `None` that nothing usable was
    /// measured.
    Stop(Option<i64>),
}

/// What the machine made of the gate it was last fed. Carried separately from
/// [`Step`] because it steers nothing - it exists so the driver can log what
/// happened without duplicating the arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// Gate not measured at all: generator off, or the count straddles a dial
    /// change and is part old frequency, part new.
    Skipped,
    /// Error against nominal, in ppb.
    Ppb(i64),
    /// Error too large to be a real crystal error - a missed or doubled
    /// reference tick. Gate rejected.
    Rejected(i64),
}

enum Phase {
    /// Generator not yet programmed; whatever is counting now means nothing.
    Start,
    /// Output settling: `left` more readings to discard.
    Warmup { left: u8 },
    /// Measuring: `left` readings still to spend. `settle` marks a gate that
    /// straddles a dial change and must be thrown away without costing one.
    Measure { left: u8, settle: bool },
    /// Finished; the driver should have stopped feeding us.
    Done,
}

pub struct Calibration {
    nominal: Frequency,
    dial: Frequency,
    phase: Phase,
    /// Plausible readings seen, so a run that measured nothing can say so
    /// rather than reporting a confident zero.
    good: u8,
    last: Reading,
}

impl Calibration {
    pub fn new(nominal: Frequency) -> Self {
        Self {
            nominal,
            dial: nominal,
            phase: Phase::Start,
            good: 0,
            last: Reading::Skipped,
        }
    }

    /// What the machine made of the most recent gate, for logging.
    pub fn last_reading(&self) -> Reading {
        self.last
    }

    /// Dial frequency currently programmed into the generator.
    pub fn dial(&self) -> Frequency {
        self.dial
    }

    /// Feed one gated tick count and get the next action.
    pub fn gate(&mut self, ticks: u32) -> Step {
        match self.phase {
            // The first event only tells us that a gate boundary went by. Use
            // it to program the dial; the count itself is meaningless, because
            // the generator was off for some or all of it.
            Phase::Start => {
                self.last = Reading::Skipped;
                self.dial = self.nominal;
                self.phase = Phase::Warmup { left: WARMUP_GATES };
                Step::SetDial(self.dial)
            }

            // Measure during warmup as well: the number steers nothing, but it
            // is the first sign of life from the generator and worth logging.
            Phase::Warmup { left } => {
                self.last = Reading::Ppb(calibrate::error_ppb(ticks, self.nominal, 1));
                self.phase = match left.saturating_sub(1) {
                    0 => Phase::Measure {
                        left: MEASURE_GATES,
                        settle: false,
                    },
                    n => Phase::Warmup { left: n },
                };
                Step::Continue
            }

            // Out of gates: take what the dial has accumulated. This arm must
            // stay ahead of the general one below, which is what makes every
            // `left - 1` there safe.
            Phase::Measure { left: 0, .. } => {
                self.last = Reading::Skipped;
                self.finish()
            }

            Phase::Measure { left, settle } => {
                // The dial moved during or just before this gate, so the count
                // is part old frequency and part new. Discard it, but do not
                // charge it against the budget.
                if settle {
                    self.last = Reading::Skipped;
                    self.phase = Phase::Measure {
                        left,
                        settle: false,
                    };
                    return Step::Continue;
                }

                let ppb = calibrate::error_ppb(ticks, self.nominal, 1);

                // A missed or doubled reference tick is not a measurement.
                if !calibrate::plausible(ppb) {
                    self.last = Reading::Rejected(ppb);
                    self.phase = Phase::Measure {
                        left: left - 1,
                        settle: false,
                    };
                    return Step::Continue;
                }

                self.last = Reading::Ppb(ppb);
                self.good += 1;

                // Close enough: stop rather than spend the rest of the budget
                // chasing noise.
                if ppb.abs() <= TOLERANCE_PPB {
                    return self.finish();
                }

                self.dial = calibrate::correct(self.dial, ppb);
                self.phase = Phase::Measure {
                    left: left - 1,
                    settle: true,
                };
                Step::SetDial(self.dial)
            }

            // The driver should have dropped us on the previous Stop.
            Phase::Done => {
                self.last = Reading::Skipped;
                Step::Stop(None)
            }
        }
    }

    fn finish(&mut self) -> Step {
        self.phase = Phase::Done;

        // Every gate was rejected: the dial was never touched, so
        // `correction_ppb` would report a confident zero for a run that
        // measured nothing at all.
        if self.good == 0 {
            return Step::Stop(None);
        }

        // What the dial ended up carrying is the whole error: each reading
        // above saw only what was left of it.
        Step::Stop(Some(calibrate::correction_ppb(self.dial, self.nominal)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOMINAL: Frequency = Frequency::from_hz(10_000_000);

    /// A part running 62 ppm fast until the dial is corrected, then perfect.
    fn ticks_for(dial: Frequency) -> u32 {
        if dial == NOMINAL {
            10_000_620
        } else {
            10_000_000
        }
    }

    #[test]
    fn converges_and_reports_the_dial_error() {
        let mut c = Calibration::new(NOMINAL);
        let mut dial = NOMINAL;
        let mut result = None;

        for _ in 0..16 {
            match c.gate(ticks_for(dial)) {
                Step::Continue => {}
                Step::SetDial(d) => dial = d,
                Step::Stop(r) => {
                    result = r;
                    break;
                }
            }
        }

        let ppb = result.expect("should have measured something");
        assert!((ppb - 62_000).abs() < 100, "unexpected {} ppb", ppb);
    }

    #[test]
    fn discards_the_gate_that_straddles_a_dial_change() {
        let mut c = Calibration::new(NOMINAL);
        assert_eq!(c.gate(0), Step::SetDial(NOMINAL));
        assert_eq!(c.gate(10_000_620), Step::Continue); // warmup 1
        assert_eq!(c.gate(10_000_620), Step::Continue); // warmup 2

        // First real measurement moves the dial...
        let Step::SetDial(dial) = c.gate(10_000_620) else {
            panic!("expected a correction")
        };
        assert!(dial < NOMINAL);

        // ...and the next count, however wild, is thrown away rather than
        // charged against the budget or acted on.
        assert_eq!(c.gate(3_000_000), Step::Continue);
        assert_eq!(c.last_reading(), Reading::Skipped);
    }

    #[test]
    fn reports_failure_when_no_gate_was_usable() {
        let mut c = Calibration::new(NOMINAL);
        let mut steps = 0;

        loop {
            match c.gate(20_000_000) {
                // doubled gate: never plausible
                Step::Stop(r) => {
                    assert_eq!(r, None);
                    break;
                }
                _ => steps += 1,
            }
            assert!(steps < 16, "should have given up");
        }
    }
}
