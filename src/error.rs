/*!
This module provides error types and result types using them.

When performing an operation that might fail, a result type is returned.
It either holds a value representing success or an error.
*/

use std;
use std::error;
use std::fmt;
use std::result;

/// The result of a parsing operation, holding either the desired return value
/// (`Ok`) or a [`ParserError`][`ParserError`] (`Err`).
///
/// When a function returns `ParserResult` and fails, the `ParserError` can be
/// used to find out what went wrong:
///
/// ```
/// # #[macro_use] extern crate calc_regex;
/// # fn main() {
/// let mut reader = calc_regex::Reader::from_array(b"foo");
///
/// let re = generate!(
///     # foo := "foo";
///     // ...
/// );
///
/// match reader.parse(&re) {
///     Ok(record) => {
///         // Do something with `record`.
///     }
///     Err(err) => {
///         match err {
///             calc_regex::ParserError::Regex { regex, value } => {
///                 // Some `regex` didn't match `value`.
///             }
///             // ...
///             # _ => {}
///         }
///     }
/// }
/// # }
/// ```
///
/// See the documentation of [`ParserError`] for a list of different errors and
/// their meanings.
///
/// See [`std::result`] for more information.
///
/// [`ParserError`]: enum.ParserError.html
/// [`std::result`]: https://doc.rust-lang.org/stable/std/result/index.html
pub type ParserResult<T> = result::Result<T, ParserError>;

/// The result of an operation that accesses a sub-expression by name, holding
/// either the desired return value (`Ok`) or a [`NameError`][`NameError`]
/// (`Err`).
///
/// When a function returns `NameResult` and fails, the `NameError` can be used
/// to find out what went wrong:
///
/// ```
/// # #[macro_use] extern crate calc_regex;
/// # fn main() {
/// let mut reader = calc_regex::Reader::from_array(b"foo");
///
/// let re = generate!(
///     # foo := "foo";
///     // ...
/// );
///
/// let record = reader.parse(&re).unwrap();
///
/// let name = "foo";
/// match record.get_capture(name) {
///     Ok(capture) => {
///         // Do something with `capture`.
///     }
///     Err(err) => {
///         match err {
///             calc_regex::NameError::NoSuchName { name } => {
///                 // `name` was not found.
///             }
///             // ...
///             # _ => {}
///         }
///     }
/// }
/// # }
/// ```
///
/// See the documentation of [`NameError`] for a list of different errors and
/// their meanings.
///
/// See [`std::result`] for more information.
///
/// [`NameError`]: enum.NameError.html
/// [`std::result`]: https://doc.rust-lang.org/stable/std/result/index.html
pub type NameResult<T> = result::Result<T, NameError>;

/// An error that occurred while parsing a calc-regular expression.
#[derive(Debug)]
pub enum ParserError {
    /// A regex could not be matched during parsing.
    ///
    /// This is likely due to invalid input.
    Regex {
        /// The regex that was not matched.
        regex: String,
        /// The offending input.
        value: Vec<u8>,
    },
    /// Reached end of file before the expression could be matched.
    ///
    /// This is likely due to invalid input.
    UnexpectedEof,
    /// Encountered conflicting bounds.
    ///
    /// This can be due to invalid input or ill-defined explicit bounds.
    ConflictingBounds {
        /// The existing bound.
        old: usize,
        /// The new bound.
        new: usize,
    },
    /// The function provided to read a counter failed.
    ///
    /// This indicates that the expression given to parse a counter and the
    /// function given to read it are not compatible.
    /// Otherwise, the raw value would not have been given to the function.
    CannotReadCount {
        /// The bytes given to the provided function.
        raw_count: Vec<u8>,
    },
    /// An IO error occurred during parsing.
    ///
    /// This indicates an error with the stream itself, rather than problems
    /// matching the expression.
    IoError {
        /// The raised error.
        err: std::io::Error,
    },
    /// There are remaining characters in the input after parsing an
    /// expression.
    ///
    /// If this should not be considered an error, use a suitable parse
    /// function.
    /// Otherwise, this is likely due to invalid input.
    TrailingCharacters,
}

/// An error that occurred when trying to access a sub-expression by name.
#[derive(Debug)]
pub enum NameError {
    /// No node with the given name exists within the `CalcRegex`.
    NoSuchName {
        /// The name that couldn't be found.
        name: String,
    },
    /// A given index was out of bounds.
    OutOfBounds {
        /// The name the index was on.
        name: String,
        /// The offending index.
        index: usize,
        /// The number of elements being indexed.
        len: usize,
    },
    /// Tried to access a single capture but found repeated capture.
    MisplacedSingleAccess {
        /// The name of the capture.
        name: String,
    },
    /// Tried to access a repeat capture but found single capture.
    MisplacedRepeatAccess {
        /// The name of the capture.
        name: String,
    },
    /// The given capture name is invalid.
    InvalidCaptureName {
        /// An error message, describing the problem.
        message: &'static str,
    },
}

impl error::Error for ParserError {
    fn description(&self) -> &str {
        match *self {
            ParserError::Regex { .. } => "a regex did not match",
            ParserError::UnexpectedEof => "unexpected end of file",
            ParserError::ConflictingBounds { .. } => "conflicting bounds",
            ParserError::CannotReadCount { .. } => "could not read count",
            ParserError::IoError { .. } => "encountered an IO error",
            ParserError::TrailingCharacters =>
                "remaining characters after parsing",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ParserError::IoError { ref err } => Some(err),
            _ => None,
        }
    }
}

impl error::Error for NameError {
    fn description(&self) -> &str {
        match *self {
            NameError::NoSuchName { .. } => "given name doesn't exist",
            NameError::OutOfBounds { .. } => "given index is out of bounds",
            NameError::MisplacedSingleAccess { .. } =>
                "falsely tried to access single capture",
            NameError::MisplacedRepeatAccess { .. } =>
                "falsely tried to access repeat capture",
            NameError::InvalidCaptureName { .. } => "given name is invalid",
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::Regex { ref regex, ref value } => write!(
                f,
                "Could not match regex: \
                 Expected (a prefix of) {:?} to match {}.",
                value,
                regex
            ),
            ParserError::ConflictingBounds { ref old, ref new } => write!(
                f,
                "Encountered conflicting bounds: \
                 The expression was already bounded to {} bytes, but a later \
                 constraint expects {} bytes.",
                old,
                new
            ),
            ParserError::CannotReadCount { ref raw_count } => write!(
                f,
                "Count value could not be read: {:?}.",
                raw_count
            ),
            ParserError::UnexpectedEof => write!(
                f,
                "Unexpected end of file."
            ),
            ParserError::IoError { ref err } => write!(
                f,
                "IO error: {:?}.",
                err
            ),
            ParserError::TrailingCharacters => write!(
                f,
                "Characters left in input after parsing."
            ),
        }
    }
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NameError::NoSuchName { ref name } => write!(
                f,
                "No node named \"{}\" exists.",
                name
            ),
            NameError::OutOfBounds { ref name, index, len } => write!(
                f,
                "Tried to get element number {} of \"{}\", but only {} \
                 elements exist.",
                index,
                name,
                len
            ),
            NameError::MisplacedSingleAccess { ref name } => write!(
                f,
                "Tried to access single capture on repeat capture \"{}\".",
                name
            ),
            NameError::MisplacedRepeatAccess { ref name } => write!(
                f,
                "Tried to access repeat capture on single capture \"{}\".",
                name
            ),
            NameError::InvalidCaptureName { ref message } => write!(
                f,
                "The given capture name is invalid: {}.",
                message
            ),
        }
    }
}
