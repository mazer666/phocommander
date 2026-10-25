//! The messages about a setting of the configuration file that had no effect.
//!
//! WHAT: the severity of a message ([`NoticeSeverity`]) and the message
//!       itself ([`ConfigNotice`]).
//! WHY: the specification (section 9.8) asks for two things that only a
//!       message can do: a rule that cannot be switched off has to say so
//!       when the file tries to switch it off, and a profile that overrides a
//!       switch has to say so as well. A user who carries a configuration
//!       file over from another machine must not silently get a weaker
//!       application than the one the file asks for — and must not be left
//!       wondering why a switch has no effect either. The messages are values
//!       and not writes to a log, because the crate that reads the file does
//!       not own the log; the composition root writes them where the log
//!       lives.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::{ConfigNotice, NoticeSeverity};
//!
//! let notice = ConfigNotice::warning(
//!     "security.background_index",
//!     "the application never keeps a background index; the value was ignored",
//! );
//! assert_eq!(notice.severity, NoticeSeverity::Warning);
//! assert!(notice.message.contains("background index"));
//! ```

/// How serious a message about the configuration file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoticeSeverity {
    /// The application does something the file did not ask for, and the user
    /// may want to know why.
    ///
    /// WHY: a profile that overrides a switch is a decision of the
    /// application, not a failure. It is worth a line in the log, but it is
    /// not a warning.
    Information,

    /// The file asked for something the application will not do.
    ///
    /// WHY: a guarantee that cannot be switched off is a promise of the
    /// product. A file that tries to break it is reported, so that the user
    /// knows that the application is still the one that was promised.
    Warning,
}

impl NoticeSeverity {
    /// The name of the severity, as the log writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::NoticeSeverity;
    ///
    /// assert_eq!(NoticeSeverity::Warning.as_str(), "warning");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            NoticeSeverity::Information => "information",
            NoticeSeverity::Warning => "warning",
        }
    }
}

/// A message about a setting of the configuration file that had no effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigNotice {
    /// The name of the field in the configuration file.
    ///
    /// WHY: the user edits the file, so the message has to name the place in
    /// the file that carries the value — not an internal name.
    pub field: String,

    /// What happened and what the application does instead.
    pub message: String,

    /// How serious the message is.
    pub severity: NoticeSeverity,
}

impl ConfigNotice {
    /// Builds a message with the severity [`NoticeSeverity::Information`].
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::ConfigNotice;
    ///
    /// let notice = ConfigNotice::new("security.remember_session", "the profile decides");
    /// assert_eq!(notice.field, "security.remember_session");
    /// ```
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            severity: NoticeSeverity::Information,
        }
    }

    /// Builds a message with the severity [`NoticeSeverity::Warning`].
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::ConfigNotice;
    ///
    /// let notice = ConfigNotice::warning("security.telemetry", "there is no telemetry");
    /// assert!(notice.message.contains("telemetry"));
    /// ```
    pub fn warning(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: NoticeSeverity::Warning,
            ..Self::new(field, message)
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn a_new_notice_is_an_information() {
        // act
        let notice = ConfigNotice::new("security.remember_session", "the profile decides");

        // assert
        assert_eq!(notice.severity, NoticeSeverity::Information);
        assert_eq!(notice.field, "security.remember_session");
        assert_eq!(notice.message, "the profile decides");
    }

    #[test]
    fn a_warning_carries_the_severity_of_a_refused_setting() {
        // act
        let notice = ConfigNotice::warning("security.telemetry", "there is no telemetry");

        // assert
        assert_eq!(notice.severity, NoticeSeverity::Warning);
        assert_eq!(notice.severity.as_str(), "warning");
    }

    #[test]
    fn the_severities_have_a_name_for_the_log() {
        // act, assert
        assert_eq!(NoticeSeverity::Information.as_str(), "information");
        assert_eq!(NoticeSeverity::Warning.as_str(), "warning");
    }
}
