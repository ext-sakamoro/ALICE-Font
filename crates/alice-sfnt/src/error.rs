//! The one error type of the crate: what went wrong and where.

use core::fmt;

/// Why a font, a table or a glyph could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A read or a declared count runs past the end of the data it belongs to.
    Truncated,
    /// A magic number, a version or a format that this crate does not read.
    Unsupported,
    /// A value that is out of its range (an index past a count, offsets out of order, a bad size).
    Malformed,
    /// A table the operation needs is not in the font.
    MissingTable,
    /// A glyph id that is not below the font's glyph count.
    BadGlyph,
    /// A nesting, stack or work limit was reached (composite depth, subroutine depth, operand stack).
    Limit,
}

/// An error with the place it was found (a short static description such as `"cmap format 4"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    /// What went wrong.
    pub kind: ErrorKind,
    /// Where: the table, field or operator being read.
    pub context: &'static str,
}

impl Error {
    pub(crate) const fn new(kind: ErrorKind, context: &'static str) -> Self {
        Error { kind, context }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            ErrorKind::Truncated => "runs past the end of its data",
            ErrorKind::Unsupported => "is not supported",
            ErrorKind::Malformed => "is malformed",
            ErrorKind::MissingTable => "is missing",
            ErrorKind::BadGlyph => "is not a glyph of this font",
            ErrorKind::Limit => "reached a limit",
        };
        write!(f, "{} {}", self.context, what)
    }
}

/// The crate's result type.
pub type Result<T> = core::result::Result<T, Error>;

pub(crate) const fn truncated(context: &'static str) -> Error {
    Error::new(ErrorKind::Truncated, context)
}

pub(crate) const fn malformed(context: &'static str) -> Error {
    Error::new(ErrorKind::Malformed, context)
}

pub(crate) const fn unsupported(context: &'static str) -> Error {
    Error::new(ErrorKind::Unsupported, context)
}

pub(crate) const fn limit(context: &'static str) -> Error {
    Error::new(ErrorKind::Limit, context)
}
