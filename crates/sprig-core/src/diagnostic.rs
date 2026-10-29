//! Problems in a plan, reported the same way by every tool.
//!
//! A diagnostic's code is its identity: `spec/diagnostics.md` lists every code
//! with its meaning, and a code once published is never reused, so a search for
//! `S0103` finds the same problem in every version. Messages may improve; codes
//! may not move.
//!
//! # Example
//!
//! ```
//! use sprig_core::{Code, Date, Severity, Span, Workspace};
//!
//! let mut ws = Workspace::new();
//! ws.insert("plan.sprig", "Plan\n\n- Go  after:^nowhere\n");
//! let res = ws.resolve(Date::new(2026, 10, 1).unwrap());
//! let d = &res.diagnostics[0];
//! assert_eq!(d.code, Code::UnknownLocalAnchor);
//! assert_eq!(d.code.as_str(), "S0201");
//! assert_eq!(d.code.rule(), "§7.2");
//! assert!(!d.code.summary().is_empty());
//! assert_eq!(d.severity(), Severity::Error);
//! assert_eq!(d.code.severity().as_str(), "error");
//! assert_eq!((d.line, d.col), (3, 7));
//! assert_eq!(d.sort_key(), ("plan.sprig", 3, 7, Code::UnknownLocalAnchor));
//! assert_eq!(Span::new(2, 5).end, 5);
//! ```

use std::fmt;

/// A byte range in a file's source text, end exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    /// The first byte.
    pub start: usize,
    /// The byte after the last.
    pub end: usize,
}

impl Span {
    /// The bytes from `start` up to but not including `end`.
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// How much a problem matters: an error means the file does not read as written.
pub enum Severity {
    /// The file does not mean what it appears to; `sprig check` exits 1.
    Error,
    /// The file reads, but probably not as meant; `sprig check --strict` exits 1.
    Warning,
}

impl Severity {
    /// `error` or `warning`, as the tree format and `check --plain` write it.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

macro_rules! codes {
    ($($variant:ident = $code:literal, $severity:ident, $rule:literal, $summary:literal;)*) => {
        /// Every problem the core can report.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        pub enum Code {
            $(#[doc = $summary] $variant,)*
        }

        impl Code {
            /// Every code, in catalogue order. A test holds this list and
            /// `spec/diagnostics.md` to each other.
            pub const ALL: &'static [Code] = &[$(Code::$variant,)*];

            /// The code as `spec/diagnostics.md` lists it, such as `S0201`.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Code::$variant => $code,)*
                }
            }

            /// Whether this code is an error or a warning; always the same for a code.
            pub fn severity(self) -> Severity {
                match self {
                    $(Code::$variant => Severity::$severity,)*
                }
            }

            /// The spec rule the problem breaks.
            pub fn rule(self) -> &'static str {
                match self {
                    $(Code::$variant => $rule,)*
                }
            }

            /// One line, as the catalogue states it.
            pub fn summary(self) -> &'static str {
                match self {
                    $(Code::$variant => $summary,)*
                }
            }
        }
    };
}

codes! {
    UnreadableDate = "S0001", Warning, "§9.5", "A date field that is not a YYYY-MM-DD date";
    UnreadableEstimate = "S0002", Warning, "§10.1", "An estimate that is not a number and a unit";
    SeveralAnchors = "S0003", Warning, "§4.5", "An item with more than one anchor";
    DuplicateAnchor = "S0004", Warning, "§4.5", "An anchor already used in the same file";
    GraftWithoutLink = "S0005", Error, "§8.1", "A graft line with no link";
    UnreadableRecurrence = "S0006", Warning, "§10.3", "An every: value that is not a recurrence";
    UnknownVersion = "S0007", Warning, "§4.6", "A title naming a version of Sprig the tool does not read";
    GraftFileMissing = "S0101", Error, "§8.5", "A graft naming a file that does not exist";
    GraftAnchorMissing = "S0102", Error, "§8.5", "A graft naming an anchor its file does not write";
    GraftLoop = "S0103", Error, "§8.4", "A graft that would loop";
    GraftOutsideRoot = "S0104", Error, "§8.2", "A graft whose path leaves the workspace";
    GraftRepeated = "S0105", Error, "§8.8", "A graft whose content is already in the tree";
    UnknownLocalAnchor = "S0201", Error, "§7.2", "A reference to an anchor the file does not write";
    UnknownFile = "S0202", Error, "§7.2", "A reference to a file that does not exist";
    UnknownAnchor = "S0203", Error, "§7.2", "A reference to an anchor its file does not write";
    ReferenceOutsideRoot = "S0204", Error, "§8.2", "A reference whose path leaves the workspace";
    NotAReference = "S0205", Error, "§7.2", "An after: value that is not a reference";
    EmptyReference = "S0206", Warning, "§5.7", "A reference to something with nothing to do";
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One problem at one place.
///
/// `line` and `col` are 1-based, and `col` counts characters, the way compilers
/// and editors print a position. `span` is the same place in bytes, for tools
/// that edit.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    /// What kind of problem, with a stable code.
    pub code: Code,
    /// The workspace path of the file it is in.
    pub file: String,
    /// The bytes it is about.
    pub span: Span,
    /// The 1-based line of the span's start.
    pub line: usize,
    /// The 1-based column, in characters, of the span's start.
    pub col: usize,
    /// What is wrong, for a person; its wording may change between releases.
    pub message: String,
    /// What to do about it, when there is something obvious.
    pub help: Option<String>,
}

impl Diagnostic {
    /// Whether it is an error or a warning, from its code.
    pub fn severity(&self) -> Severity {
        self.code.severity()
    }

    /// Sorts by place, then by code, so output is the same however it was found.
    pub fn sort_key(&self) -> (&str, usize, usize, Code) {
        (&self.file, self.line, self.col, self.code)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}[{}]: {}",
            self.file,
            self.line,
            self.col,
            self.severity().as_str(),
            self.code,
            self.message
        )
    }
}

/// Turns byte offsets into lines and columns for one file.
pub(crate) struct LineIndex<'a> {
    src: &'a str,
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub(crate) fn new(src: &'a str) -> LineIndex<'a> {
        let mut starts = vec![0];
        starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex { src, starts }
    }

    /// 1-based line and character column of a byte offset.
    pub(crate) fn position(&self, offset: usize) -> (usize, usize) {
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let start = self.starts[line];
        let end = offset.min(self.src.len());
        // A BOM is not a column anybody can see.
        let text = self.src.get(start..end).unwrap_or("");
        let col = text.trim_start_matches('\u{feff}').chars().count() + 1;
        (line + 1, col)
    }

    pub(crate) fn diagnostic(
        &self,
        file: &str,
        code: Code,
        span: Span,
        message: impl Into<String>,
    ) -> Diagnostic {
        let (line, col) = self.position(span.start);
        Diagnostic {
            code,
            file: file.to_string(),
            span,
            line,
            col,
            message: message.into(),
            help: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diagnostic_prints_the_way_compilers_do() {
        let src = "Plan\n\n- Sign  after:^permt\n";
        let index = LineIndex::new(src);
        let at = src.find("after").unwrap();
        let d = index.diagnostic(
            "lease.sprig",
            Code::UnknownLocalAnchor,
            Span::new(at, at + 12),
            "no item ^permt in lease.sprig",
        );
        assert_eq!(
            d.to_string(),
            "lease.sprig:3:9: error[S0201]: no item ^permt in lease.sprig"
        );
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        let src = "Plan\n- Café  after:^x\n";
        let index = LineIndex::new(src);
        let at = src.find("after").unwrap();
        assert_eq!(index.position(at), (2, 9));
    }

    #[test]
    fn codes_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for c in Code::ALL {
            assert!(seen.insert(c.as_str()), "{c} is listed twice");
        }
    }
}
