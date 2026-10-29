//! Interrupts as eventfds: vfio-pci writes vector `i`'s eventfd each time
//! the device sends message `i`; `wait` blocks on it with a timeout.
//!
//! The card uses MSI-X; QEMU's `edu` (the VM test) has only MSI. Both arrive
//! the same way here, so the VM test proves this path for both except the
//! MSI-X table programming, which vfio-pci does in the kernel.

use std::os::fd::AsRawFd;
use std::time::Duration;

use vmm_sys_util::eventfd::{EFD_NONBLOCK, EventFd};

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqKind {
    Msi,
    Msix,
}

pub struct Irqs {
    pub kind: IrqKind,
    fds: Vec<EventFd>,
}

impl Irqs {
    pub fn new(kind: IrqKind, vectors: usize) -> Result<Irqs> {
        let fds = (0..vectors)
            .map(|_| {
                EventFd::new(EFD_NONBLOCK).map_err(|err| Error::Os {
                    what: "eventfd".into(),
                    err,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Irqs { kind, fds })
    }

    pub fn fds(&self) -> Vec<&EventFd> {
        self.fds.iter().collect()
    }

    /// Block until `vector` has fired at least once since the last `wait`
    /// (or `clear`) on it, and return how many times it fired.
    pub fn wait(&self, vector: u32, timeout: Duration) -> Result<u64> {
        let fd = self.fd(vector)?;
        if let Some(n) = take(fd)? {
            return Ok(n);
        }
        let mut pfd = libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // Round up so a sub-millisecond timeout still waits, not polls.
        let ms = timeout.as_nanos().div_ceil(1_000_000).min(i32::MAX as u128) as i32;
        // SAFETY: one valid pollfd.
        match unsafe { libc::poll(&mut pfd, 1, ms) } {
            -1 => Err(Error::last_os("poll on interrupt eventfd")),
            0 => Err(Error::Timeout { done: 0 }),
            _ => take(fd)?.ok_or(Error::Timeout { done: 0 }),
        }
    }

    /// Forget any interrupts already counted on `vector`.
    pub fn clear(&self, vector: u32) -> Result<()> {
        take(self.fd(vector)?).map(|_| ())
    }

    fn fd(&self, vector: u32) -> Result<&EventFd> {
        self.fds.get(vector as usize).ok_or_else(|| {
            Error::InvalidArgument(format!(
                "interrupt vector {vector}, device has {}",
                self.fds.len()
            ))
        })
    }
}

/// Read and reset the eventfd counter; `None` if it was zero.
fn take(fd: &EventFd) -> Result<Option<u64>> {
    match fd.read() {
        Ok(n) => Ok(Some(n)),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(err) => Err(Error::Os {
            what: "read interrupt eventfd".into(),
            err,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn counts_and_resets() {
        let irqs = Irqs::new(IrqKind::Msi, 2).unwrap();
        irqs.fds()[1].write(1).unwrap();
        irqs.fds()[1].write(1).unwrap();
        assert_eq!(irqs.wait(1, Duration::ZERO).unwrap(), 2);
        assert!(matches!(
            irqs.wait(1, Duration::ZERO),
            Err(Error::Timeout { done: 0 })
        ));
    }

    #[test]
    fn wakes_on_a_write_from_another_thread() {
        let irqs = Irqs::new(IrqKind::Msix, 1).unwrap();
        let t0 = Instant::now();
        std::thread::scope(|s| {
            s.spawn(|| {
                std::thread::sleep(Duration::from_millis(20));
                irqs.fds()[0].write(1).unwrap();
            });
            assert_eq!(irqs.wait(0, Duration::from_secs(5)).unwrap(), 1);
        });
        assert!(t0.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn times_out_and_rejects_unknown_vectors() {
        let irqs = Irqs::new(IrqKind::Msi, 1).unwrap();
        let t0 = Instant::now();
        assert!(matches!(
            irqs.wait(0, Duration::from_millis(30)),
            Err(Error::Timeout { .. })
        ));
        assert!(t0.elapsed() >= Duration::from_millis(30));
        assert!(matches!(
            irqs.wait(1, Duration::ZERO),
            Err(Error::InvalidArgument(_))
        ));
    }
}
