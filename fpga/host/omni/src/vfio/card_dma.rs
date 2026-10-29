//! The card's two XDMA channels, and keeping them safe across suspend.
//!
//! - Each channel sits behind its own lock, so one H2C and one C2H
//!   transfer run at once (on separate threads) and two of the same kind
//!   queue.
//! - Suspend (the logind hook, on its own thread): mark the card asleep,
//!   which makes every in-flight transfer's next poll abort it (the
//!   channel stops itself on the way out), then take both locks — so the
//!   hook returns, and the system sleeps, only once no channel is running.
//! - Resume: count it and wake. The card lost its XDMA setup, so the next
//!   call re-runs `xdma::init` (lazily, from a thread that has the BAR).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::bar::Bar;
use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};
use crate::sleep_inhibit::SleepEvent;
use crate::xdma::{self, Channel, Data, DeviceMem, Dir, Port};

pub struct CardDma {
    h2c: Mutex<Channel>,
    c2h: Mutex<Channel>,
    asleep: AtomicBool,
    resumes: AtomicU64,
    /// `resumes` as of the last `xdma::init`.
    inited_at: Mutex<u64>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic mid-transfer leaves the channel stopped or stoppable; the
    // next transfer reprograms everything, so a poisoned lock is usable.
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl CardDma {
    /// Bring the XDMA up (identify, stop channels, route the shell IRQ).
    pub fn new(bar1: &Bar) -> Result<CardDma> {
        xdma::init(&mut CardPort::new(bar1, &AtomicBool::new(false)))?;
        Ok(CardDma {
            h2c: Mutex::new(Channel::new(Dir::H2c)),
            c2h: Mutex::new(Channel::new(Dir::C2h)),
            asleep: AtomicBool::new(false),
            resumes: AtomicU64::new(0),
            inited_at: Mutex::new(0),
        })
    }

    pub fn resumes(&self) -> u64 {
        self.resumes.load(Ordering::SeqCst)
    }

    /// The sleep hook.
    pub fn on_sleep(&self, e: SleepEvent) {
        match e {
            SleepEvent::Suspending => {
                self.asleep.store(true, Ordering::SeqCst);
                drop(lock(&self.h2c));
                drop(lock(&self.c2h));
            }
            SleepEvent::Resumed => {
                self.resumes.fetch_add(1, Ordering::SeqCst);
                self.asleep.store(false, Ordering::SeqCst);
            }
        }
    }

    /// Before touching the card: refuse while asleep; after a resume, run
    /// `check` (the caller's identity check) and re-init the XDMA once.
    pub fn ready(&self, bar1: &Bar, check: impl FnOnce() -> Result<()>) -> Result<()> {
        if self.asleep.load(Ordering::SeqCst) {
            return Err(Error::Device("system is suspending".into()));
        }
        let mut at = lock(&self.inited_at);
        let now = self.resumes();
        if *at != now {
            check()?;
            xdma::init(&mut CardPort::new(bar1, &self.asleep))?;
            *at = now;
        }
        Ok(())
    }

    pub fn transfer(
        &self,
        bar1: &Bar,
        alloc: &dyn Fn(usize) -> Result<DmaBuf>,
        data: Data<'_>,
        len: usize,
        timeout: Duration,
    ) -> Result<()> {
        let ch = match data {
            Data::H2c(_) => &self.h2c,
            Data::C2h(_) => &self.c2h,
        };
        lock(ch).transfer(
            &mut CardPort::new(bar1, &self.asleep),
            alloc,
            data,
            len,
            timeout,
        )
    }
}

/// Polls that only yield before polling starts to sleep: the first tens of
/// microseconds, where a short transfer finishes.
const SPIN_POLLS: u32 = 200;
const POLL_SLEEP: Duration = Duration::from_micros(20);

/// BAR1, and a pause between polls of the writeback (which is in RAM, so
/// polling it costs no PCIe traffic).
struct CardPort<'a> {
    bar: &'a Bar,
    asleep: &'a AtomicBool,
    polls: u32,
}

impl<'a> CardPort<'a> {
    fn new(bar: &'a Bar, asleep: &'a AtomicBool) -> Self {
        CardPort {
            bar,
            asleep,
            polls: 0,
        }
    }
}

impl Port for CardPort<'_> {
    fn read32(&mut self, offset: u32) -> Result<u32> {
        self.bar.read32(offset)
    }

    fn write32(&mut self, offset: u32, value: u32) -> Result<()> {
        self.bar.write32(offset, value)
    }

    fn wait(&mut self, _mem: &DeviceMem<'_>, deadline: Instant) -> Result<bool> {
        if self.asleep.load(Ordering::SeqCst) {
            return Err(Error::Device(
                "transfer aborted: the system is suspending".into(),
            ));
        }
        self.polls += 1;
        if self.polls <= SPIN_POLLS {
            std::thread::yield_now();
        } else {
            let left = deadline.saturating_duration_since(Instant::now());
            std::thread::sleep(left.min(POLL_SLEEP));
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xdma::regs::*;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

    /// A BAR1 in memory whose identifiers read like the card's XDMA.
    fn fake_bar1() -> (OwnedFd, Bar) {
        // SAFETY: plain syscalls; the fd is owned from here on.
        let fd = unsafe {
            let fd = libc::memfd_create(c"omni-xdma-test".as_ptr(), 0);
            assert!(fd >= 0);
            assert_eq!(libc::ftruncate(fd, 0x10000), 0);
            OwnedFd::from_raw_fd(fd)
        };
        let bar = Bar::new(fd.as_raw_fd(), 0, 0x10000, true).unwrap();
        let id = |target: u32| ID_MAGIC << ID_MAGIC_SHIFT | target << ID_TARGET_SHIFT | ID_STREAM;
        bar.write32(H2C_CHAN, id(TARGET_H2C)).unwrap();
        bar.write32(C2H_CHAN, id(TARGET_C2H)).unwrap();
        bar.write32(IRQ_BLOCK, id(TARGET_IRQ)).unwrap();
        (fd, bar)
    }

    #[test]
    fn open_routes_the_shell_interrupt() {
        let (_fd, bar) = fake_bar1();
        CardDma::new(&bar).unwrap();
        assert_eq!(bar.read32(IRQ_BLOCK + IRQ_USER_INT_EN).unwrap(), 1);
    }

    #[test]
    fn not_an_xdma_is_refused() {
        let (_fd, bar) = fake_bar1();
        bar.write32(C2H_CHAN, 0).unwrap();
        assert!(matches!(CardDma::new(&bar), Err(Error::Device(_))));
    }

    #[test]
    fn suspend_refuses_and_resume_reinits_once() {
        let (_fd, bar) = fake_bar1();
        let dma = CardDma::new(&bar).unwrap();
        dma.on_sleep(SleepEvent::Suspending);
        assert!(dma.ready(&bar, || Ok(())).is_err());
        // The card forgets its setup across the sleep.
        bar.write32(IRQ_BLOCK + IRQ_USER_INT_EN, 0).unwrap();
        dma.on_sleep(SleepEvent::Resumed);
        let mut checks = 0;
        dma.ready(&bar, || {
            checks += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(bar.read32(IRQ_BLOCK + IRQ_USER_INT_EN).unwrap(), 1);
        dma.ready(&bar, || {
            checks += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(checks, 1, "re-init once per resume");
    }

    #[test]
    fn a_failed_identity_check_after_resume_is_retried() {
        let (_fd, bar) = fake_bar1();
        let dma = CardDma::new(&bar).unwrap();
        dma.on_sleep(SleepEvent::Suspending);
        dma.on_sleep(SleepEvent::Resumed);
        let gone = || Err(Error::Device("ID reads 0xffffffff".into()));
        assert!(dma.ready(&bar, gone).is_err());
        assert!(dma.ready(&bar, gone).is_err(), "still not re-inited");
        dma.ready(&bar, || Ok(())).unwrap();
    }
}
