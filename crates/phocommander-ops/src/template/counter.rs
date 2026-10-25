//! The counter of a rename rule.
//!
//! WHAT: the rule of the counter ([{CounterRule}]) and the order in which the
//!       files of a selection receive their number ([{CounterOrder}]).
//! WHY: the counter numbers the files of a selection (FR-32): it starts at a
//!       value, it grows by a step, and it is written with a fixed number of
//!       digits, so that the files of one run are named in the same width.
//!       The order decides which file receives which number — the order of
//!       the selection, the order of the names or the order of the time of
//!       the last change.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::template::{CounterOrder, CounterRule};
//!
//! let counter = CounterRule::new(1, 1, 3).with_order(CounterOrder::Name);
//! assert_eq!(counter.start, 1);
//! assert_eq!(counter.width, 3);
//! assert_eq!(counter.order, CounterOrder::Name);
//! ```
//!
//! BOUNDARY: this module counts. It reads no file and it forms no name.

use crate::template::RenameInput;
use crate::template::file_name_of;
use crate::template::placeholders::COUNTER_MARKER;

/// The rule of the counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterRule {
    /// The number of the first file.
    pub start: i64,

    /// The distance between two numbers.
    pub step: i64,

    /// The number of digits the number is written with.
    ///
    /// WHY: a number that is written with three digits is "001", and not "1".
    ///       The files of one selection therefore keep the same width, and a
    ///       sorted directory shows them in the order of the numbers.
    pub width: usize,

    /// The order in which the files receive the number.
    pub order: CounterOrder,
}

impl Default for CounterRule {
    /// Builds the counter that numbers the files from one.
    ///
    /// WHY: the default is the counter of the camera template — it starts at
    ///       one and it is written with three digits.
    fn default() -> Self {
        Self {
            start: 1,
            step: 1,
            width: 3,
            order: CounterOrder::Selection,
        }
    }
}

impl CounterRule {
    /// Builds a counter.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::CounterRule;
    ///
    /// let counter = CounterRule::new(10, 5, 2);
    /// assert_eq!(counter.start, 10);
    /// assert_eq!(counter.step, 5);
    /// assert_eq!(counter.width, 2);
    /// ```
    pub fn new(start: i64, step: i64, width: usize) -> Self {
        Self {
            start,
            step,
            width,
            order: CounterOrder::Selection,
        }
    }

    /// Sets the order in which the files receive the number.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{CounterOrder, CounterRule};
    ///
    /// let counter = CounterRule::new(1, 1, 3).with_order(CounterOrder::Date);
    /// assert_eq!(counter.order, CounterOrder::Date);
    /// ```
    pub fn with_order(mut self, order: CounterOrder) -> Self {
        self.order = order;
        self
    }
}

/// The order in which the files of a selection receive the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CounterOrder {
    /// The order of the selection.
    #[default]
    Selection,

    /// The order of the names of the files.
    Name,

    /// The order of the time of the last change of the files.
    Date,
}

/// The number of every file of the selection.
///
/// WHY: the numbers are read for the whole selection and not for a single
///       file, because the order decides which file receives which number.
pub(crate) fn counter_values(
    rule: Option<&CounterRule>,
    inputs: &[RenameInput],
) -> Vec<Option<i64>> {
    let Some(rule) = rule else {
        return vec![None; inputs.len()];
    };

    let order = order_of(rule.order, inputs);
    let mut values: Vec<Option<i64>> = vec![None; inputs.len()];
    for (position, index) in order.into_iter().enumerate() {
        // WHY: the arithmetic saturates instead of wrapping. A counter that
        //       runs over a very long selection must not overflow into a
        //       negative number.
        let step = rule
            .step
            .saturating_mul(i64::try_from(position).unwrap_or(i64::MAX));
        values[index] = Some(rule.start.saturating_add(step));
    }
    values
}

/// The order in which the files receive the number.
///
/// WHY: the sort keeps the order of the selection for the files that compare
///       equal, so that the result is the same in every run.
fn order_of(order: CounterOrder, inputs: &[RenameInput]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..inputs.len()).collect();
    match order {
        CounterOrder::Selection => {}
        CounterOrder::Name => {
            let names: Vec<String> = inputs.iter().map(file_name_of).collect();
            indices.sort_by(|left, right| names[*left].cmp(&names[*right]))
        }
        CounterOrder::Date => {
            indices.sort_by(|left, right| inputs[*left].modified.cmp(&inputs[*right].modified))
        }
    }
    indices
}

/// Writes the number of the counter into the name.
///
/// WHY: the counter is written after the search and the replacement, so that
///       a replacement rule cannot change a number that the user wants to
///       keep. A rule without a counter leaves the placeholder in the name as
///       it is, so that the user sees that the counter is missing.
pub(crate) fn apply(text: &str, value: Option<i64>, width: usize) -> String {
    let number = match value {
        Some(value) => format!("{value:0width$}", width = width),
        None => "[n]".to_string(),
    };
    text.replace(COUNTER_MARKER, number.as_str())
}
