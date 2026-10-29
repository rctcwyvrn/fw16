//! `open(uri)`: pick a backend by URI scheme.

use crate::error::{Error, Result};
use crate::mock::Mock;
use crate::transport::Transport;
use crate::vfio::{Bdf, Card};

/// `mock:` — the in-process shell model. `vfio:<BDF>` — the card bound to
/// vfio-pci (`0000:01:00.0` or `01:00.0`).
pub fn open(uri: &str) -> Result<Box<dyn Transport>> {
    match uri.split_once(':') {
        Some(("mock", "")) => Ok(Box::new(Mock::new())),
        Some(("vfio", bdf)) => match Bdf::parse(bdf) {
            Some(bdf) => Ok(Box::new(Card::open(bdf)?)),
            None => Err(Error::BadUri(uri.to_string())),
        },
        _ => Err(Error::BadUri(uri.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemes() {
        assert!(open("mock:").is_ok());
        // Domain ffff never exists, so this reaches the sysfs check and
        // fails there, whatever machine runs the test.
        assert!(matches!(open("vfio:ffff:01:00.0"), Err(Error::Device(_))));
        assert!(matches!(open("vfio:"), Err(Error::BadUri(_))));
        assert!(matches!(open("vfio:01:00"), Err(Error::BadUri(_))));
        assert!(matches!(open("mock"), Err(Error::BadUri(_))));
        assert!(matches!(open("sim:"), Err(Error::BadUri(_))));
    }
}
