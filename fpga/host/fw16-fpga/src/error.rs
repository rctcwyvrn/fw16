//! The one error type every transport returns.

use std::{fmt, io};

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
    /// A VFIO ioctl failed (container, group, device, DMA map, interrupts).
    Vfio(vfio_ioctls::VfioError),
    /// A system call outside the VFIO ioctls failed; `what` says which step.
    Os { what: String, err: io::Error },
    /// Talking to logind over the system bus failed.
    Dbus(zbus::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// `Error::Os` for the step `what`, from `errno`.
    pub(crate) fn last_os(what: impl Into<String>) -> Error {
        Error::Os {
            what: what.into(),
            err: io::Error::last_os_error(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Timeout { done } => write!(f, "timed out after {done} bytes"),
            Error::BadUri(uri) => write!(f, "bad transport URI {uri:?} (want mock: or vfio:<BDF>)"),
            Error::Unsupported(what) => write!(f, "not supported yet: {what}"),
            Error::InvalidArgument(why) => write!(f, "invalid argument: {why}"),
            Error::Device(why) => write!(f, "device error: {why}"),
            Error::Vfio(e) => write!(f, "vfio: {e}"),
            Error::Os { what, err } => write!(f, "{what}: {err}"),
            Error::Dbus(e) => write!(f, "logind: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Vfio(e) => Some(e),
            Error::Os { err, .. } => Some(err),
            Error::Dbus(e) => Some(e),
            _ => None,
        }
    }
}

impl From<vfio_ioctls::VfioError> for Error {
    fn from(e: vfio_ioctls::VfioError) -> Error {
        Error::Vfio(e)
    }
}

impl From<zbus::Error> for Error {
    fn from(e: zbus::Error) -> Error {
        Error::Dbus(e)
    }
}
