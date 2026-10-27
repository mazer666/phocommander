//! The two dialects of the placeholders.
//!
//! WHAT: the dialect of a text ([`Dialect`]) and the conversion from one
//! dialect into the other ([`Dialect::convert`]).
//! WHY: the specification decides in E13 that the rename engine knows both
//! dialects: square brackets in the dialog, curly braces in the named
//! templates. The dialog stays simple, the templates get the expressive
//! power, and the separation is documented.
//!
//! The named templates are a file that the user reads and edits. The
//! dialect of the file is therefore part of its interface, and the
//! conversion happens at the border — when a template is stored and when
//! it is applied to the dialog.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::rename::Dialect;
//!
//! let known = ["name"];
//! let text = "Serienname - [name]";
//! assert_eq!(
//!     Dialect::convert(text, &known, Dialect::Square, Dialect::Curly),
//!     "Serienname - {name}"
//! );
//! ```

#[cfg(test)]
mod tests;

/// The dialect of the placeholders in a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dialect {
    /// The square brackets — the dialect of the dialog.
    Square,

    /// The curly braces — the dialect of the named templates.
    Curly,
}

impl Dialect {
    /// The opening character of the dialect.
    pub const fn open(&self) -> char {
        match self {
            Self::Square => '[',
            Self::Curly => '{',
        }
    }

    /// The closing character of the dialect.
    pub const fn close(&self) -> char {
        match self {
            Self::Square => ']',
            Self::Curly => '}',
        }
    }

    /// Converts a text from one dialect into the other.
    ///
    /// WHY: the conversion only touches the brackets that frame a name the
    /// engine knows. A bracket in a file name — `[1]` in `episode [1].mkv`
    /// — belongs to the name itself and stays as it is. A bracket around a
    /// name that the engine does not know stays as well, because the user
    /// may have written it for another tool.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::Dialect;
    ///
    /// let known = ["name"];
    /// assert_eq!(
    ///     Dialect::convert("[name] [1] [unknown]", &known, Dialect::Square, Dialect::Curly),
    ///     "{name} [1] [unknown]"
    /// );
    /// ```
    pub fn convert(text: &str, known: &[&str], from: Dialect, to: Dialect) -> String {
        if from == to {
            return text.to_string();
        }

        let mut out = String::with_capacity(text.len());
        let mut rest = text;

        while let Some(start) = rest.find(from.open()) {
            out.push_str(&rest[..start]);
            let tail = &rest[start + from.open().len_utf8()..];
            let Some(end) = tail.find(from.close()) else {
                // WHY: a bracket without a partner is a bracket of the name
                //       itself and not the start of a placeholder.
                out.push(from.open());
                rest = tail;
                continue;
            };

            let name = &tail[..end];
            if known.contains(&name) {
                out.push(to.open());
                out.push_str(name);
                out.push(to.close());
            } else {
                // WHY: the engine does not know the name. The bracket
                //       belongs to the text of the user.
                out.push(from.open());
                out.push_str(name);
                out.push(from.close());
            }
            rest = &tail[end + from.close().len_utf8()..];
        }

        out.push_str(rest);
        out
    }
}
