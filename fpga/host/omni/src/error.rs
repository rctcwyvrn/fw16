//! The one error type every transport returns.

use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// The operation did not finish in time; `done` bytes were transferred
    /// before it gave up (0 for `wait_irq`).
    Timeout { done: usize },
    /// `open` could not parse the URI.
    BadUri(String),
    /// The backend exists in the plan but is not built yet.
    Unsupported(&'static str),
    /// A caller error: misaligned offset, length not a whole number of beats,
    /// length past the end of the buffer, unknown interrupt vector.
    InvalidArgument(String),
    /// The device answered, but not sensibly (e.g. a 64-bit counter whose
    /// high word never settles).
    Device(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Timeout { done } => write!(f, "timed out after {done} bytes"),
            Error::BadUri(uri) => write!(f, "bad transport URI {uri:?} (want mock: or vfio:<BDF>)"),
            Error::Unsupported(what) => write!(f, "not supported yet: {what}"),
            Error::InvalidArgument(why) => write!(f, "invalid argument: {why}"),
            Error::Device(why) => write!(f, "device error: {why}"),
        }
    }
}

impl std::error::Error for Error {}
