//! `open(uri)`: pick a backend by URI scheme.

use crate::error::{Error, Result};
use crate::mock::Mock;
use crate::transport::Transport;

/// `mock:` — the in-process shell model. `vfio:<BDF>` — the card bound to
/// vfio-pci (Path 4 step 4, not built yet).
pub fn open(uri: &str) -> Result<Box<dyn Transport>> {
    match uri.split_once(':') {
        Some(("mock", "")) => Ok(Box::new(Mock::new())),
        Some(("vfio", bdf)) if !bdf.is_empty() => {
            Err(Error::Unsupported("the vfio backend (Path 4 step 4)"))
        }
        _ => Err(Error::BadUri(uri.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemes() {
        assert!(open("mock:").is_ok());
        assert!(matches!(
            open("vfio:0000:01:00.0"),
            Err(Error::Unsupported(_))
        ));
        assert!(matches!(open("vfio:"), Err(Error::BadUri(_))));
        assert!(matches!(open("mock"), Err(Error::BadUri(_))));
        assert!(matches!(open("sim:"), Err(Error::BadUri(_))));
    }
}
