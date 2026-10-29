//! PCI addresses (`vfio:<BDF>`) and the sysfs checks that make an open
//! failure say what to fix, instead of a bare ENOENT from `/dev/vfio`.

use std::fmt;
use std::path::PathBuf;

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bdf {
    pub domain: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl Bdf {
    /// `DDDD:BB:DD.F` or `BB:DD.F` (domain 0), hex, as `lspci -D` prints.
    pub fn parse(s: &str) -> Option<Bdf> {
        let (rest, function) = s.rsplit_once('.')?;
        let mut parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
            parts.insert(0, "0000");
        }
        let [domain, bus, device] = parts[..] else {
            return None;
        };
        fn hex(s: &str, max_len: usize) -> Option<&str> {
            (s.len() <= max_len && !s.is_empty()).then_some(s)
        }
        let bdf = Bdf {
            domain: u16::from_str_radix(hex(domain, 4)?, 16).ok()?,
            bus: u8::from_str_radix(hex(bus, 2)?, 16).ok()?,
            device: u8::from_str_radix(hex(device, 2)?, 16).ok()?,
            function: u8::from_str_radix(hex(function, 1)?, 16).ok()?,
        };
        (bdf.device < 32 && bdf.function < 8).then_some(bdf)
    }

    pub fn sysfs(&self) -> PathBuf {
        PathBuf::from(format!("/sys/bus/pci/devices/{self}"))
    }

    /// The device exists and is bound to `vfio-pci`.
    pub fn check_bound(&self) -> Result<()> {
        let dev = self.sysfs();
        if !dev.exists() {
            return Err(Error::Device(format!("no PCI device {self}")));
        }
        let driver = dev
            .join("driver")
            .read_link()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
        match driver.as_deref() {
            Some("vfio-pci") => Ok(()),
            Some(other) => Err(Error::Device(format!(
                "{self} is bound to {other}, not vfio-pci"
            ))),
            None => Err(Error::Device(format!(
                "{self} has no driver; bind it to vfio-pci"
            ))),
        }
    }
}

impl fmt::Display for Bdf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{:x}",
            self.domain, self.bus, self.device, self.function
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_forms_and_prints_the_long_one() {
        let b = Bdf::parse("0000:01:00.0").unwrap();
        assert_eq!(b, Bdf::parse("01:00.0").unwrap());
        assert_eq!(Bdf::parse("1:2.3").unwrap().to_string(), "0000:01:02.3");
        assert_eq!(
            Bdf::parse("abcd:ef:1f.7").unwrap().to_string(),
            "abcd:ef:1f.7"
        );
    }

    #[test]
    fn rejects_garbage() {
        for s in [
            "",
            "01:00",
            "0000:01:00.8",
            "0000:01:20.0",
            "x:01:00.0",
            "0:0:01:00.0",
            "00000:01:00.0",
        ] {
            assert_eq!(Bdf::parse(s), None, "{s:?}");
        }
    }
}
