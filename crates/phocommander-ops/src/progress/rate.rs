//! The rate of a running job.
//!
//! WHAT: the clock of a job ([`Clock`]), the rate that the clock measures
//!       ([`Rate`]) and the estimate of the remaining time ([`Rate::eta`]).
//! WHY: a job that copies ten gigabytes is a job that takes a while, and a
//! user who does not know how long it will run cannot decide whether to
//! wait or to stop. The cue of the job therefore carries the rate and the
//! estimate (FR-24, FR-22g).
//! The clock is a trait, and not the system clock, because a test that
//! measures a rate has to know the time. A test with the system clock would
//! either sleep or hope — and a test that hopes is not a test.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::progress::rate::{Clock, Rate, Seconds};
//!
//! let mut clock = Seconds::new();
//! let mut rate = Rate::new(&clock);
//! clock.advance(2.0);
//! assert_eq!(rate.bytes_per_second(1_000, &clock), Some(500));
//! ```

/// The clock of a job.
pub trait Clock: Send + Sync {
    /// The seconds that passed since the start of the job.
    fn seconds(&self) -> f64;
}

/// The clock of the system.
#[derive(Debug, Clone)]
pub struct SystemClock {
    /// The moment of the start.
    start: std::time::Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemClock {
    /// A clock that starts now.
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Clock for Box<dyn Clock> {
    fn seconds(&self) -> f64 {
        (**self).seconds()
    }
}

impl Clock for SystemClock {
    fn seconds(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

/// A clock that the test moves by hand.
#[derive(Debug, Clone, Default)]
pub struct Seconds {
    /// The seconds that passed since the start.
    seconds: f64,
}

impl Seconds {
    /// A clock at the start.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the clock forward.
    pub fn advance(&mut self, seconds: f64) {
        self.seconds += seconds;
    }
}

impl Clock for Seconds {
    fn seconds(&self) -> f64 {
        self.seconds
    }
}

/// The rate of a running job.
#[derive(Debug, Clone, Copy)]
pub struct Rate {
    /// The seconds that passed when the measurement began.
    at_start: f64,
    /// The bytes that were done when the measurement began.
    bytes_at_start: u64,
}

impl Rate {
    /// Begins the measurement.
    pub fn new<C: Clock + ?Sized>(clock: &C) -> Self {
        Self {
            at_start: clock.seconds(),
            bytes_at_start: 0,
        }
    }

    /// The bytes per second, if the job has run long enough for a number.
    ///
    /// WHY: the first fraction of a second says nothing. A job that copied
    ///       ten bytes in a millisecond would report a rate of ten kilobytes
    ///       per second, and the number would frighten the user for no
    ///       reason. The rate therefore needs a second of work before it is
    ///       reported.
    pub fn bytes_per_second<C: Clock + ?Sized>(&self, bytes_done: u64, clock: &C) -> Option<u64> {
        let seconds = clock.seconds() - self.at_start;
        if seconds < 1.0 {
            return None;
        }
        let bytes = bytes_done.saturating_sub(self.bytes_at_start);
        Some((bytes as f64 / seconds) as u64)
    }

    /// The seconds that the rest of the job will take, if that can be said.
    ///
    /// WHY: without a total there is nothing to estimate, and without a
    ///       rate there is nothing to estimate with. Both are missing when
    ///       the job has just begun, and the answer is then `None` — the
    ///       cue shows nothing instead of a number that would be wrong.
    pub fn eta<C: Clock + ?Sized>(
        &self,
        bytes_done: u64,
        bytes_total: u64,
        clock: &C,
    ) -> Option<u64> {
        let rate = self.bytes_per_second(bytes_done, clock)?;
        if rate == 0 {
            return None;
        }
        let rest = bytes_total.saturating_sub(bytes_done);
        Some(rest / rate)
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_rate_of_a_job_that_ran_a_second_is_the_bytes_of_that_second() {
        // arrange
        let mut clock = Seconds::new();
        let rate = Rate::new(&clock);

        // act
        clock.advance(2.0);
        let bytes = rate.bytes_per_second(1_000, &clock);

        // assert
        assert_eq!(bytes, Some(500));
    }

    #[test]
    fn a_job_that_has_not_run_a_second_has_no_rate() {
        // arrange
        // WHY: ten bytes in a millisecond would be a rate of ten kilobytes
        //       per second, and the number would be wrong.
        let mut clock = Seconds::new();
        let rate = Rate::new(&clock);

        // act
        clock.advance(0.5);
        let bytes = rate.bytes_per_second(1_000, &clock);

        // assert
        assert_eq!(bytes, None);
    }

    #[test]
    fn the_estimate_is_the_rest_of_the_bytes_at_the_rate() {
        // arrange
        let mut clock = Seconds::new();
        let rate = Rate::new(&clock);
        clock.advance(1.0);

        // act
        let eta = rate.eta(1_000, 11_000, &clock);

        // assert
        // WHY: 10 000 bytes are left and 1 000 bytes go by the second.
        assert_eq!(eta, Some(10));
    }

    #[test]
    fn a_job_without_a_total_has_no_estimate() {
        // arrange
        let mut clock = Seconds::new();
        let rate = Rate::new(&clock);
        clock.advance(1.0);

        // act
        let ohne_gesamt = rate.eta(1_000, 0, &clock);
        let ohne_zeit = {
            let neu = Seconds::new();
            Rate::new(&neu).eta(1_000, 11_000, &neu)
        };

        // assert
        assert_eq!(ohne_gesamt, Some(0));
        assert_eq!(ohne_zeit, None);
    }

    #[test]
    fn a_job_that_copies_nothing_has_no_rate_to_estimate_with() {
        // arrange
        let mut clock = Seconds::new();
        let rate = Rate::new(&clock);
        clock.advance(1.0);

        // act, assert
        assert_eq!(rate.bytes_per_second(0, &clock), Some(0));
        assert_eq!(rate.eta(0, 0, &clock), None);
    }
}
