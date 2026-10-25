//! The parts of the dialog of the multiple rename that have the focus.
//!
//! WHAT: the part of the dialog that has the focus ([`RenameFocus`]).
//! WHY: the order of the parts is the order of the tab key, and it is fixed
//!       (section 6.4 of the specification). The order is therefore part of
//!       the state and not an accident of the drawing. Every part of the
//!       dialog is reachable with the keyboard — the mouse is optional.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::rename::RenameFocus;
//!
//! assert_eq!(RenameFocus::all().len(), 11);
//! assert_eq!(RenameFocus::all()[0], RenameFocus::Format);
//! ```

/// The part of the dialog that has the focus.
///
/// WHY: the order of the parts is the order of the tab key, and it is fixed
///       (section 6.4 of the specification). The order is therefore part of
///       the state and not an accident of the drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameFocus {
    /// The field with the format of the new name.
    Format,

    /// The field with the text that is searched.
    Search,

    /// The field with the text that takes its place.
    Replacement,

    /// The switch that turns the search into a regular expression.
    Regex,

    /// The field with the first number of the counter.
    CounterStart,

    /// The field with the step of the counter.
    CounterStep,

    /// The field with the width of the counter.
    CounterWidth,

    /// The field with the name of the series.
    SeriesName,

    /// The field with the name of a new template.
    TemplateName,

    /// The list of the stored templates.
    Templates,

    /// The list of the preview.
    Preview,
}

impl RenameFocus {
    /// Every part of the dialog, in the order of the tab key.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameFocus;
    ///
    /// assert_eq!(RenameFocus::all().len(), 11);
    /// assert_eq!(RenameFocus::all()[0], RenameFocus::Format);
    /// ```
    pub const fn all() -> [RenameFocus; 11] {
        [
            RenameFocus::Format,
            RenameFocus::Search,
            RenameFocus::Replacement,
            RenameFocus::Regex,
            RenameFocus::CounterStart,
            RenameFocus::CounterStep,
            RenameFocus::CounterWidth,
            RenameFocus::SeriesName,
            RenameFocus::TemplateName,
            RenameFocus::Templates,
            RenameFocus::Preview,
        ]
    }

    /// Whether the part is a text field.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameFocus;
    ///
    /// assert!(RenameFocus::Format.is_field());
    /// assert!(!RenameFocus::Preview.is_field());
    /// ```
    pub const fn is_field(&self) -> bool {
        matches!(
            self,
            RenameFocus::Format
                | RenameFocus::Search
                | RenameFocus::Replacement
                | RenameFocus::CounterStart
                | RenameFocus::CounterStep
                | RenameFocus::CounterWidth
                | RenameFocus::SeriesName
                | RenameFocus::TemplateName
        )
    }
}
