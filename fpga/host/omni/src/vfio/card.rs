//! `vfio:<BDF>` — the omnimeter card through [`VfioPci`]: BAR0 is the
//! shell's AXI-Lite registers, BAR1 the XDMA config block (regmap v0.2
//! proposal), interrupts are MSI-X vectors, and a logind delay inhibitor
//! guards suspend.
//!
//! The XDMA descriptor engine is Path 4 step 5, so `dma_read`/`dma_write`
//! are `Unsupported` for now and the sleep hook only counts events; step 5
//! hooks quiesce and re-init into it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::bdf::Bdf;
use super::pci::VfioPci;
use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};
use crate::regs::{OMNI_ID, OMNI_ID_RESET};
use crate::sleep_inhibit::{SleepEvent, SleepInhibitor};
use crate::transport::Transport;

pub struct Card {
    pci: VfioPci,
    /// `None` when there is no logind (a headless box without a system
    /// bus): the card still works, it just will not survive a suspend.
    sleep: Option<SleepInhibitor>,
    /// Bumped on each resume; step 5 re-initialises XDMA when it moves.
    resumes: Arc<AtomicU64>,
}

impl Card {
    pub fn open(bdf: Bdf) -> Result<Card> {
        let mut pci = VfioPci::open(bdf)?;
        let id = pci.bar(0)?.read32(OMNI_ID)?;
        if id != OMNI_ID_RESET {
            return Err(Error::Device(format!(
                "{bdf}: ID reads {id:#010x}, not {OMNI_ID_RESET:#010x} — not an omnimeter shell"
            )));
        }
        pci.enable_irqs()?;
        let resumes = Arc::new(AtomicU64::new(0));
        let r = resumes.clone();
        let hook = Box::new(move |e| {
            if e == SleepEvent::Resumed {
                r.fetch_add(1, Ordering::SeqCst);
            }
        });
        let sleep = SleepInhibitor::start(&format!("omnimeter {bdf} DMA"), hook).ok();
        Ok(Card {
            pci,
            sleep,
            resumes,
        })
    }

    pub fn pci(&self) -> &VfioPci {
        &self.pci
    }

    /// Whether a logind inhibitor guards this handle against suspend.
    pub fn sleep_guarded(&self) -> bool {
        self.sleep.as_ref().is_some_and(SleepInhibitor::held)
    }

    pub fn resumes(&self) -> u64 {
        self.resumes.load(Ordering::SeqCst)
    }
}

impl Transport for Card {
    fn read32(&self, offset: u32) -> Result<u32> {
        self.pci.bar(0)?.read32(offset)
    }

    fn write32(&self, offset: u32, value: u32) -> Result<()> {
        self.pci.bar(0)?.write32(offset, value)
    }

    fn alloc(&self, len: usize) -> Result<DmaBuf> {
        self.pci.alloc(len)
    }

    fn dma_read(&self, buf: &mut DmaBuf, len: usize, _timeout: Duration) -> Result<()> {
        buf.check_len(len)?;
        Err(Error::Unsupported(
            "C2H on the card: the XDMA engine (Path 4 step 5)",
        ))
    }

    fn dma_write(&self, buf: &DmaBuf, len: usize, _timeout: Duration) -> Result<()> {
        buf.check_len(len)?;
        Err(Error::Unsupported(
            "H2C on the card: the XDMA engine (Path 4 step 5)",
        ))
    }

    fn wait_irq(&self, vector: u32, timeout: Duration) -> Result<()> {
        self.pci.wait_irq(vector, timeout).map(|_| ())
    }
}
