//! The undo area of the work surface.
//!
//! WHAT: what the undo of the product can do ([`UndoScope`]) and whether the
//!       surface can take a copy back ([`UndoAvailability`]).
//! WHY: the undo of the specification is deliberately incomplete (FR-26): the
//!       undo of a copy means "delete the copy", not "restore the state that
//!       was there before". A file that the copy overwrote is gone, and the
//!       undo cannot bring it back. The limitation is therefore shown in the
//!       interface and not only in the documentation — the surface carries
//!       the notice of the undo area, and the notice names the limitation
//!       every time an undo is on offer.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::undo::{UndoAvailability, UndoScope};
//!
//! let availability = UndoAvailability::available(UndoScope::RemovesCopiesOnly);
//! assert!(availability.is_available());
//! assert_eq!(availability.notice_key(), Some("panel-undo-notice"));
//! ```
//! BOUNDARY: this module holds the state of the undo area and the text of
//!       its notice. It performs no file operation and knows no file system.

use crate::i18n::Catalogue;

/// The key of the notice of the undo area in the catalogue.
///
/// WHY: the notice is a visible text and goes through the catalogue
///       (FR-05). The key lives next to the notice, so that the notice and
///       the key cannot drift apart.
pub const UNDO_NOTICE_KEY: &str = "panel-undo-notice";

/// What the undo of the product can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoScope {
    /// The undo removes the copies and restores nothing.
    ///
    /// WHY: a copy does not keep a copy of the file it overwrote, so the
    ///       undo cannot restore it. The scope names the limitation, so that
    ///       the interface can show it.
    RemovesCopiesOnly,
}

impl UndoScope {
    /// The name of the scope.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::undo::UndoScope;
    ///
    /// assert_eq!(UndoScope::RemovesCopiesOnly.as_str(), "removes-copies-only");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            UndoScope::RemovesCopiesOnly => "removes-copies-only",
        }
    }

    /// Whether the undo restores the state that was there before.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::undo::UndoScope;
    ///
    /// assert!(!UndoScope::RemovesCopiesOnly.restores_previous_state());
    /// ```
    pub const fn restores_previous_state(&self) -> bool {
        false
    }
}

/// Whether the surface can take a copy back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UndoAvailability {
    /// There is nothing to take back.
    #[default]
    Unavailable,

    /// There is something to take back, and the scope says what that means.
    Available {
        /// What the undo can do.
        scope: UndoScope,
    },
}

impl UndoAvailability {
    /// Builds the availability of an undo with the given scope.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::undo::{UndoAvailability, UndoScope};
    ///
    /// let availability = UndoAvailability::available(UndoScope::RemovesCopiesOnly);
    /// assert!(availability.is_available());
    /// ```
    pub const fn available(scope: UndoScope) -> Self {
        UndoAvailability::Available { scope }
    }

    /// Whether an undo is on offer.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::undo::UndoAvailability;
    ///
    /// assert!(!UndoAvailability::Unavailable.is_available());
    /// ```
    pub const fn is_available(&self) -> bool {
        matches!(self, UndoAvailability::Available { .. })
    }

    /// What the undo can do, if it is on offer.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::undo::{UndoAvailability, UndoScope};
    ///
    /// let availability = UndoAvailability::available(UndoScope::RemovesCopiesOnly);
    /// assert_eq!(availability.scope(), Some(UndoScope::RemovesCopiesOnly));
    /// ```
    pub const fn scope(&self) -> Option<UndoScope> {
        match self {
            UndoAvailability::Unavailable => None,
            UndoAvailability::Available { scope } => Some(*scope),
        }
    }

    /// The key of the notice that the undo area shows.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::undo::UndoAvailability;
    ///
    /// assert_eq!(UndoAvailability::Unavailable.notice_key(), None);
    /// assert_eq!(
    ///     UndoAvailability::default().notice_key(),
    ///     None
    /// );
    /// ```
    pub const fn notice_key(&self) -> Option<&'static str> {
        match self {
            UndoAvailability::Unavailable => None,
            UndoAvailability::Available { .. } => Some(UNDO_NOTICE_KEY),
        }
    }

    /// The text of the notice that the undo area shows.
    ///
    /// WHY: the notice is never empty. A catalogue that does not know the
    ///       key answers with the key itself, and the user sees that there is
    ///       a translation missing (FR-122).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::i18n::{Argument, Catalogue};
    /// use phocommander_ui::undo::{UndoAvailability, UndoScope};
    ///
    /// # struct Empty;
    /// # impl Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let catalogue = Empty;
    /// let availability = UndoAvailability::available(UndoScope::RemovesCopiesOnly);
    /// assert_eq!(availability.notice(&catalogue).as_deref(), Some("panel-undo-notice"));
    /// ```
    pub fn notice(&self, catalogue: &dyn Catalogue) -> Option<String> {
        let key = self.notice_key()?;
        match catalogue.lookup(key) {
            Some(text) if !text.is_empty() => Some(text),
            _ => Some(key.to_string()),
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::i18n::Argument;

    /// A catalogue that knows the keys of the undo area.
    struct CatalogueWithUndo;

    impl Catalogue for CatalogueWithUndo {
        fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
            match key {
                "panel-undo-notice" => Some(
                    "Undo removes the copies only; the previous state is not restored.".to_string(),
                ),
                _ => None,
            }
        }
    }

    #[test]
    fn an_undo_that_is_on_offer_shows_its_limitation() {
        // arrange
        let catalogue = CatalogueWithUndo;
        let availability = UndoAvailability::available(UndoScope::RemovesCopiesOnly);

        // act
        let notice = availability.notice(&catalogue);

        // assert
        assert_eq!(
            notice.as_deref(),
            Some("Undo removes the copies only; the previous state is not restored.")
        );
    }

    #[test]
    fn without_an_undo_there_is_no_notice() {
        // arrange
        let catalogue = CatalogueWithUndo;

        // act, assert
        assert!(UndoAvailability::Unavailable.notice(&catalogue).is_none());
        assert!(!UndoAvailability::Unavailable.is_available());
    }

    #[test]
    fn the_notice_is_never_empty() {
        // WHY: a catalogue that does not know the key answers with the key
        // itself — the notice stays visible (FR-122).
        // arrange
        struct Empty;
        impl Catalogue for Empty {
            fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
                None
            }
        }
        let catalogue = Empty;
        let availability = UndoAvailability::available(UndoScope::RemovesCopiesOnly);

        // act
        let notice = availability.notice(&catalogue);

        // assert
        assert_eq!(notice.as_deref(), Some("panel-undo-notice"));
    }

    #[test]
    fn the_scope_says_that_nothing_is_restored() {
        // act, assert
        assert!(!UndoScope::RemovesCopiesOnly.restores_previous_state());
        assert_eq!(UndoScope::RemovesCopiesOnly.as_str(), "removes-copies-only");
    }
}
