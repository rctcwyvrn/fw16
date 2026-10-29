//! One PCI BAR as 32-bit registers: mmapped from the VFIO device fd at the
//! region's offset (vfio-ioctls has no helper for this), or, when the kernel
//! will not let the whole region be mapped, `pread`/`pwrite` on that fd.
//!
//! The fallback matters on the card: if a kernel ever reports the BAR as
//! sparse-mmap (the MSI-X table carved out of the XDMA BAR), a flat mapping
//! would fault, while file I/O on the region still reaches every register.

use std::os::fd::RawFd;
use std::ptr::NonNull;

use crate::error::{Error, Result};

pub struct Bar {
    fd: RawFd,
    /// Offset of the region in the device fd.
    offset: u64,
    size: u64,
    map: Option<NonNull<u8>>,
}

// SAFETY: the mapping is device memory reached only through volatile 32-bit
// accesses, which are atomic on x86 and need no lock; `fd` outlives the Bar
// (the owning device keeps it open, see `VfioPci`).
unsafe impl Send for Bar {}
unsafe impl Sync for Bar {}

impl Bar {
    /// Map `size` bytes of `fd` at `offset` if `mmap` is true and the kernel
    /// agrees; otherwise go through file I/O. `fd` must stay open for the
    /// Bar's whole life.
    pub fn new(fd: RawFd, offset: u64, size: u64, mmap: bool) -> Result<Bar> {
        let map = if mmap { map(fd, offset, size)? } else { None };
        Ok(Bar {
            fd,
            offset,
            size,
            map,
        })
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn is_mapped(&self) -> bool {
        self.map.is_some()
    }

    pub fn read32(&self, off: u32) -> Result<u32> {
        self.check(off)?;
        match self.map {
            // SAFETY: `check` keeps the 4 bytes inside the mapping and aligned.
            Some(p) => Ok(unsafe { p.add(off as usize).cast::<u32>().read_volatile() }),
            None => {
                let mut v = [0u8; 4];
                self.file_io(off, v.as_mut_ptr(), false)?;
                Ok(u32::from_le_bytes(v))
            }
        }
    }

    pub fn write32(&self, off: u32, value: u32) -> Result<()> {
        self.check(off)?;
        match self.map {
            // SAFETY: as in `read32`.
            Some(p) => unsafe { p.add(off as usize).cast::<u32>().write_volatile(value) },
            None => {
                let mut v = value.to_le_bytes();
                self.file_io(off, v.as_mut_ptr(), true)?;
            }
        }
        Ok(())
    }

    fn check(&self, off: u32) -> Result<()> {
        if !off.is_multiple_of(4) || u64::from(off) + 4 > self.size {
            return Err(Error::InvalidArgument(format!(
                "register offset {off:#x} not aligned or past the {:#x}-byte BAR",
                self.size
            )));
        }
        Ok(())
    }

    fn file_io(&self, off: u32, buf: *mut u8, write: bool) -> Result<()> {
        let pos = (self.offset + u64::from(off)) as libc::off_t;
        // SAFETY: `buf` points at 4 live bytes; `fd` is open (see `new`).
        let n = unsafe {
            if write {
                libc::pwrite(self.fd, buf.cast(), 4, pos)
            } else {
                libc::pread(self.fd, buf.cast(), 4, pos)
            }
        };
        match n {
            4 => Ok(()),
            -1 => Err(Error::last_os(format!("BAR access at {off:#x}"))),
            n => Err(Error::Device(format!(
                "BAR access at {off:#x} moved {n} bytes"
            ))),
        }
    }
}

/// `None` when the kernel refuses the mapping with EINVAL (a region that is
/// not mappable after all); any other failure is an error.
fn map(fd: RawFd, offset: u64, size: u64) -> Result<Option<NonNull<u8>>> {
    // SAFETY: a fresh shared mapping of the device fd; nothing else aliases it.
    let p = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size as usize,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            offset as libc::off_t,
        )
    };
    if p == libc::MAP_FAILED {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINVAL) {
            return Ok(None);
        }
        return Err(Error::Os {
            what: format!("mmap BAR ({size:#x} bytes at {offset:#x})"),
            err,
        });
    }
    Ok(NonNull::new(p.cast()))
}

impl Drop for Bar {
    fn drop(&mut self) {
        if let Some(p) = self.map {
            // SAFETY: unmapping exactly what `map` mapped.
            unsafe { libc::munmap(p.as_ptr().cast(), self.size as usize) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

    fn memfd(len: u64) -> OwnedFd {
        // SAFETY: plain syscalls; the fd is owned from here on.
        unsafe {
            let fd = libc::memfd_create(c"omni-bar-test".as_ptr(), 0);
            assert!(fd >= 0);
            assert_eq!(libc::ftruncate(fd, len as libc::off_t), 0);
            OwnedFd::from_raw_fd(fd)
        }
    }

    #[test]
    fn mapped_and_file_io_paths_see_the_same_registers() {
        let fd = memfd(8192);
        let mapped = Bar::new(fd.as_raw_fd(), 4096, 4096, true).unwrap();
        let file = Bar::new(fd.as_raw_fd(), 4096, 4096, false).unwrap();
        assert!(mapped.is_mapped() && !file.is_mapped());
        mapped.write32(0x10, 0xDEAD_BEEF).unwrap();
        assert_eq!(file.read32(0x10).unwrap(), 0xDEAD_BEEF);
        file.write32(0xFFC, 0x1234_5678).unwrap();
        assert_eq!(mapped.read32(0xFFC).unwrap(), 0x1234_5678);
    }

    #[test]
    fn rejects_misaligned_and_out_of_range() {
        let fd = memfd(4096);
        let bar = Bar::new(fd.as_raw_fd(), 0, 4096, true).unwrap();
        assert!(matches!(bar.read32(2), Err(Error::InvalidArgument(_))));
        assert!(matches!(bar.read32(4096), Err(Error::InvalidArgument(_))));
        assert!(matches!(
            bar.write32(4094, 0),
            Err(Error::InvalidArgument(_))
        ));
    }
}
