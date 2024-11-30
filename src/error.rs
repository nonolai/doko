use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::io;

/// Doko's internal, all-encompassing error type.
pub enum Error {
    IO(io::Error),
    Utf8(OsString),
    Empty,
}

/// Result type for all fallible operations in this crate.
pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => err.fmt(f),
            Error::Utf8(name) => write!(
                f,
                "unsupported non-UTF-8 file name: {}",
                name.to_string_lossy(),
            ),
            Error::Empty => f.write_str("no source files found"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
