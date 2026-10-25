//! The filter of a directory panel.
//!
//! WHAT: the pattern language over the names of the entries
//!       ([`FilterPattern`]), the kind of the entries that the filter lets
//!       through ([`FilterKind`]), the depth the filter reaches
//!       ([`FilterDepth`]), the combination of the three ([`Filter`]), the
//!       state of the input line ([`FilterField`]) and the index of the
//!       entries that pass the filter ([`FilteredIndex`]).
//! WHY: the filter of a panel is evaluated while the user types, on every
//!       key stroke (FR-74a). The evaluation is therefore a pure function —
//!       the same input yields the same result, always — and it lives
//!       outside of the drawing code. The index of the matches is built once
//!       per change of the filter or of the directory, and only the visible
//!       slice of it reaches the widgets, so that a directory with a
//!       hundred thousand entries does not cost a hundred thousand rows per
//!       frame (FR-106, NFR-01).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::filter::{Filter, FilterKind, FilterPattern};
//!
//! let pattern = FilterPattern::parse("*.txt");
//! assert!(pattern.matches("notes.txt"));
//! assert!(!pattern.matches("notes.md"));
//!
//! let filter = Filter::new(pattern, FilterKind::Files, Default::default());
//! assert!(filter.is_active());
//! ```

pub mod field;
pub mod index;

use crate::listing::{Entry, EntryKind};

pub use field::FilterField;
pub use index::FilteredIndex;

/// The kind of the entries a filter lets through.
///
/// WHY: the two kinds that are not "all" partition the list — every entry is
/// either a directory or not a directory — so that no entry disappears
/// silently when the user switches between them (FR-74c).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FilterKind {
    /// Every entry: the files and the directories.
    #[default]
    All,

    /// Only the entries that are not directories.
    Files,

    /// Only the directories.
    Directories,
}

impl FilterKind {
    /// Every kind, in a stable order.
    pub const fn all() -> [FilterKind; 3] {
        [FilterKind::All, FilterKind::Files, FilterKind::Directories]
    }

    /// The kind that follows this one in the cycle.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::filter::FilterKind;
    ///
    /// assert_eq!(FilterKind::All.next(), FilterKind::Files);
    /// assert_eq!(FilterKind::Directories.next(), FilterKind::All);
    /// ```
    pub const fn next(&self) -> FilterKind {
        match self {
            FilterKind::All => FilterKind::Files,
            FilterKind::Files => FilterKind::Directories,
            FilterKind::Directories => FilterKind::All,
        }
    }

    /// Whether an entry of the given kind passes the filter.
    ///
    /// WHY: a symbolic link and a device node are not directories, and they
    /// are therefore part of the files. The two kinds partition the list.
    pub const fn accepts(&self, kind: EntryKind) -> bool {
        match self {
            FilterKind::All => true,
            FilterKind::Files => !kind.is_directory(),
            FilterKind::Directories => kind.is_directory(),
        }
    }
}

/// The depth the filter reaches.
///
/// WHY: the filter of a panel works on the directory that is shown
/// (FR-74d). The depth is part of the filter and not of the pattern,
/// because it says how far the filter looks and not what it looks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FilterDepth {
    /// Only the directory that is shown.
    #[default]
    Current,

    /// The directory that is shown and the directories below it.
    Recursive {
        /// The number of levels below the directory that is shown.
        levels: u16,
    },
}

impl FilterDepth {
    /// Whether an entry at the given distance from the shown directory
    /// passes the filter.
    ///
    /// WHY: the distance is the number of directories between the shown
    /// directory and the directory of the entry. An entry of the shown
    /// directory has the distance zero.
    pub const fn accepts(&self, depth: u16) -> bool {
        match self {
            FilterDepth::Current => depth == 0,
            FilterDepth::Recursive { levels } => depth <= *levels,
        }
    }

    /// The depth that follows this one in the cycle.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::filter::FilterDepth;
    ///
    /// assert_eq!(FilterDepth::Current.next(), FilterDepth::Recursive { levels: 1 });
    /// ```
    pub const fn next(&self) -> FilterDepth {
        match self {
            FilterDepth::Current => FilterDepth::Recursive { levels: 1 },
            FilterDepth::Recursive { levels: 1 } => FilterDepth::Recursive { levels: 2 },
            FilterDepth::Recursive { .. } => FilterDepth::Current,
        }
    }
}

/// One part of a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    /// A literal character that has to be in the name.
    Literal(char),

    /// `*` — any number of characters, including none.
    Any,

    /// `?` — exactly one character.
    One,
}

/// The shape of a pattern, decided once when the pattern is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Shape {
    /// No wildcard at all: a plain text that is searched anywhere in the
    /// name.
    #[default]
    Text,

    /// Exactly one `*` and no `?`: a prefix and a suffix.
    PrefixSuffix,

    /// The general case: several wildcards or a `?`.
    General,
}

/// A pattern over the names of the entries.
///
/// WHY: the pattern is read once and then evaluated many times — once per
/// entry of the directory, on every key stroke. The text is therefore
/// compiled into a shape when it is read, so that the evaluation does not
/// parse the text again and again.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::filter::FilterPattern;
///
/// let pattern = FilterPattern::parse("report");
/// assert!(pattern.matches("the-report-2026.txt"));
/// assert!(!pattern.matches("notes.txt"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FilterPattern {
    text: String,
    literal: String,
    folded: Vec<char>,
    prefix: Vec<char>,
    suffix: Vec<char>,
    parts: Vec<Part>,
    shape: Shape,
    case_insensitive: bool,
}

impl FilterPattern {
    /// Reads a pattern from the text of the filter field.
    ///
    /// WHY: the grammar is fixed and small — `*` stands for any number of
    /// characters, `?` for exactly one, a leading `~` makes the whole
    /// pattern ignore the case, and a text without a wildcard is searched
    /// anywhere in the name (FR-74b). The text is the only thing the user
    /// types, and it is therefore the only thing that is parsed.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::filter::FilterPattern;
    ///
    /// assert!(FilterPattern::parse("~*.TXT").matches("notes.txt"));
    /// assert!(!FilterPattern::parse("*.TXT").matches("notes.txt"));
    /// ```
    pub fn parse(text: &str) -> Self {
        let mut body = text;
        let mut case_insensitive = false;
        if let Some(rest) = body.strip_prefix('~') {
            case_insensitive = true;
            body = rest;
        }

        let mut parts = Vec::new();
        let mut literal = String::new();
        for character in body.chars() {
            match character {
                '*' => parts.push(Part::Any),
                '?' => parts.push(Part::One),
                other => {
                    parts.push(Part::Literal(other));
                    literal.push(other);
                }
            }
        }

        let shape = decide_shape(&parts);
        let folded = if case_insensitive {
            literal.chars().flat_map(char::to_lowercase).collect()
        } else {
            Vec::new()
        };
        let (prefix, suffix) = match &shape {
            Shape::PrefixSuffix => {
                let position = parts
                    .iter()
                    .position(|part| matches!(part, Part::Any))
                    .unwrap_or(0);
                let prefix = characters_of(&parts[..position], case_insensitive);
                let suffix = characters_of(&parts[position + 1..], case_insensitive);
                (prefix, suffix)
            }
            _ => (Vec::new(), Vec::new()),
        };

        Self {
            text: text.to_string(),
            literal,
            folded,
            prefix,
            suffix,
            parts,
            shape,
            case_insensitive,
        }
    }

    /// The text the pattern was read from.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the pattern asks for nothing.
    ///
    /// WHY: an empty pattern is not a filter — it lets every entry through —
    /// and the panel has to be able to tell the two apart.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Whether the pattern ignores the case.
    pub fn is_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    /// Whether the name matches the pattern.
    ///
    /// WHY: the evaluation is a pure function. The same name and the same
    /// pattern always yield the same answer, and the answer does not depend
    /// on the order of the entries, on the panel or on the frame.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::filter::FilterPattern;
    ///
    /// let pattern = FilterPattern::parse("*.txt");
    /// assert!(pattern.matches("notes.txt"));
    /// assert!(!pattern.matches("notes.md"));
    /// ```
    pub fn matches(&self, name: &str) -> bool {
        match &self.shape {
            Shape::Text => {
                if self.case_insensitive {
                    contains_folded(name, &self.folded)
                } else {
                    name.contains(&self.literal)
                }
            }
            Shape::PrefixSuffix => {
                starts_with_chars(name, &self.prefix, self.case_insensitive)
                    && ends_with_chars(name, &self.suffix, self.case_insensitive)
            }
            Shape::General => glob_matches(&self.parts, name, self.case_insensitive),
        }
    }
}

/// Decides the shape of a pattern from its parts.
fn decide_shape(parts: &[Part]) -> Shape {
    if parts.iter().all(|part| matches!(part, Part::Literal(_))) {
        return Shape::Text;
    }

    let stars = parts
        .iter()
        .filter(|part| matches!(part, Part::Any))
        .count();
    let questions = parts
        .iter()
        .filter(|part| matches!(part, Part::One))
        .count();

    if stars == 1 && questions == 0 {
        return Shape::PrefixSuffix;
    }

    Shape::General
}

/// The characters of the literal parts, folded if the pattern ignores the
/// case.
fn characters_of(parts: &[Part], case_insensitive: bool) -> Vec<char> {
    parts
        .iter()
        .filter_map(|part| match part {
            Part::Literal(character) => Some(*character),
            _ => None,
        })
        .flat_map(|character| {
            if case_insensitive {
                character.to_lowercase().collect::<Vec<char>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

/// Whether two characters are the same, ignoring the case if they shall.
fn chars_equal(left: char, right: char, case_insensitive: bool) -> bool {
    if case_insensitive {
        left.to_lowercase().eq(right.to_lowercase())
    } else {
        left == right
    }
}

/// Whether the name holds the characters, ignoring the case.
///
/// WHY: the search allocates nothing. A directory with a hundred thousand
/// entries is filtered on every key stroke, and an allocation per entry
/// would cost more than the comparison itself.
fn contains_folded(name: &str, needle: &[char]) -> bool {
    if needle.is_empty() {
        return true;
    }
    let mut rest = name;
    loop {
        let mut characters = rest.chars();
        let mut matched = true;
        for expected in needle {
            match characters.next() {
                Some(actual) if actual.to_lowercase().eq(expected.to_lowercase()) => {}
                _ => {
                    matched = false;
                    break;
                }
            }
        }
        if matched {
            return true;
        }
        let Some(first) = rest.chars().next() else {
            return false;
        };
        rest = &rest[first.len_utf8()..];
    }
}

/// Whether the name begins with the characters.
///
/// WHY: the characters of the pattern are folded when the pattern ignores
/// the case, and the comparison folds the name as well — and only then.
fn starts_with_chars(name: &str, prefix: &[char], case_insensitive: bool) -> bool {
    if prefix.is_empty() {
        return true;
    }
    let mut characters = name.chars();
    for expected in prefix {
        match characters.next() {
            Some(actual) if chars_equal(actual, *expected, case_insensitive) => {}
            _ => return false,
        }
    }
    true
}

/// Whether the name ends with the characters.
///
/// WHY: the characters of the pattern are folded when the pattern ignores
/// the case, and the comparison folds the name as well — and only then.
fn ends_with_chars(name: &str, suffix: &[char], case_insensitive: bool) -> bool {
    if suffix.is_empty() {
        return true;
    }
    let mut characters = name.chars().rev();
    for expected in suffix.iter().rev() {
        match characters.next() {
            Some(actual) if chars_equal(actual, *expected, case_insensitive) => {}
            _ => return false,
        }
    }
    true
}

/// Matches a name against a pattern that holds wildcards.
///
/// WHY: the algorithm walks the name once and remembers the last `*` it
/// saw. When the rest of the pattern does not fit, the `*` takes one
/// character more and the walk continues. The work is therefore linear in
/// the common case and never exponential.
fn glob_matches(parts: &[Part], name: &str, case_insensitive: bool) -> bool {
    let mut part_index = 0usize;
    let mut name_offset = 0usize;
    let mut star_part: Option<usize> = None;
    let mut star_offset = 0usize;

    loop {
        let expected = parts.get(part_index);
        let actual = name[name_offset..].chars().next();

        match (expected, actual) {
            (None, None) => return true,
            (Some(Part::Any), _) => {
                star_part = Some(part_index);
                star_offset = name_offset;
                part_index += 1;
            }
            (Some(Part::One), Some(character)) => {
                name_offset += character.len_utf8();
                part_index += 1;
            }
            (Some(Part::Literal(wanted)), Some(character)) => {
                if chars_equal(*wanted, character, case_insensitive) {
                    name_offset += character.len_utf8();
                    part_index += 1;
                } else if !backtrack(
                    &mut part_index,
                    &mut name_offset,
                    &mut star_part,
                    &mut star_offset,
                    name,
                ) {
                    return false;
                }
            }
            _ => {
                if !backtrack(
                    &mut part_index,
                    &mut name_offset,
                    &mut star_part,
                    &mut star_offset,
                    name,
                ) {
                    return false;
                }
            }
        }
    }
}

/// Lets the last `*` take one character more and repeats the match.
///
/// WHY: the `*` is the only part of the pattern that can give back
/// characters. When the pattern does not fit, the `*` grows by one
/// character and the match continues behind it.
fn backtrack(
    part_index: &mut usize,
    name_offset: &mut usize,
    star_part: &mut Option<usize>,
    star_offset: &mut usize,
    name: &str,
) -> bool {
    let Some(star) = *star_part else {
        return false;
    };
    let Some(character) = name[*star_offset..].chars().next() else {
        return false;
    };
    *star_offset += character.len_utf8();
    *name_offset = *star_offset;
    *part_index = star + 1;
    true
}

/// The filter of a panel: a pattern, a kind and a depth.
///
/// WHY: the three belong together, because the answer to the question "does
/// this entry pass the filter" needs all three. The filter is a value and
/// not a piece of state, so that two panels with the same filter behave
/// the same way.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Filter {
    pattern: FilterPattern,
    kind: FilterKind,
    depth: FilterDepth,
}

impl Filter {
    /// A filter with a pattern, a kind and a depth.
    pub fn new(pattern: FilterPattern, kind: FilterKind, depth: FilterDepth) -> Self {
        Self {
            pattern,
            kind,
            depth,
        }
    }

    /// The filter that reads the text of the filter field.
    ///
    /// WHY: the text of the field is the pattern, and the kind and the depth
    /// keep the values they had.
    pub fn from_text(text: &str, kind: FilterKind, depth: FilterDepth) -> Self {
        Self::new(FilterPattern::parse(text), kind, depth)
    }

    /// The pattern of the filter.
    pub fn pattern(&self) -> &FilterPattern {
        &self.pattern
    }

    /// The kind of the entries the filter lets through.
    pub fn kind(&self) -> FilterKind {
        self.kind
    }

    /// The depth the filter reaches.
    pub fn depth(&self) -> FilterDepth {
        self.depth
    }

    /// The filter without a pattern.
    pub fn without_pattern(&self) -> Self {
        Self::new(FilterPattern::default(), self.kind, self.depth)
    }

    /// Whether the filter lets something through.
    ///
    /// WHY: an empty pattern is not a filter. The panel has to be able to
    /// tell "no filter" and "a filter that matches nothing" apart.
    pub fn is_active(&self) -> bool {
        !self.pattern.is_empty()
    }

    /// Whether the entry passes the filter.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::filter::{Filter, FilterKind};
    /// use phocommander_ui::listing::{Entry, EntryKind};
    ///
    /// let filter = Filter::from_text("*.txt", FilterKind::Files, Default::default());
    /// assert!(filter.matches_entry(&Entry::new("notes.txt", EntryKind::File)));
    /// assert!(!filter.matches_entry(&Entry::new("notes.md", EntryKind::File)));
    /// assert!(!filter.matches_entry(&Entry::new("notes.txt", EntryKind::Directory)));
    /// ```
    pub fn matches_entry(&self, entry: &Entry) -> bool {
        self.kind.accepts(entry.kind)
            && self.depth.accepts(entry.depth())
            && self.pattern.matches(&entry.name)
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;

#[cfg(test)]
// WHY: the property tests prove that the evaluation of the filter is a
// pure function — the same input yields the same result, always.
mod property_tests;
