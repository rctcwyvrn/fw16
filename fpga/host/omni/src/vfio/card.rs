//! `vfio:<BDF>` — the omnimeter card through [`VfioPci`]: BAR0 is the
//! shell's AXI-Lite registers, BAR1 the XDMA register space
//! (`gateware/vivado/xdma_0.tcl`), interrupts are MSI-X vectors, DMA is the
//! XDMA engine ([`crate::xdma`]) on IOMMU-mapped buffers, and a logind
//! delay inhibitor guards suspend (`card_dma.rs` says how).

use std::sync::Arc;
use std::time::Duration;

use super::bar::Bar;
use super::bdf::Bdf;
use super::card_dma::CardDma;
use super::pci::VfioPci;
use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};
use crate::regs::{OMNI_ID, OMNI_ID_RESET};
use crate::sleep_inhibit::SleepInhibitor;
use crate::transport::Transport;
use crate::xdma::Data;

pub struct Card {
    pci: VfioPci,
    dma: Arc<CardDma>,
    /// `None` when there is no logind (a headless box without a system
    /// bus): the card still works, it just will not survive a suspend.
    sleep: Option<SleepInhibitor>,
}

impl Card {
    pub fn open(bdf: Bdf) -> Result<Card> {
        let mut pci = VfioPci::open(bdf)?;
        check_id(&pci)?;
        pci.enable_irqs()?;
        let dma = Arc::new(CardDma::new(pci.bar(1)?)?);
        let d = dma.clone();
        let sleep = SleepInhibitor::start(
            &format!("omnimeter {bdf} DMA"),
            Box::new(move |e| d.on_sleep(e)),
        )
        .ok();
        Ok(Card { pci, dma, sleep })
    }

    pub fn pci(&self) -> &VfioPci {
        &self.pci
    }

    /// Whether a logind inhibitor guards this handle against suspend.
    pub fn sleep_guarded(&self) -> bool {
        self.sleep.as_ref().is_some_and(SleepInhibitor::held)
    }

    pub fn resumes(&self) -> u64 {
        self.dma.resumes()
    }

    fn bar1(&self) -> Result<&Bar> {
        self.pci.bar(1)
    }

    /// Refuse while suspending; re-check and re-init after a resume.
    fn ready(&self) -> Result<&Bar> {
        let bar1 = self.bar1()?;
        self.dma.ready(bar1, || check_id(&self.pci))?;
        Ok(bar1)
    }
}

fn check_id(pci: &VfioPci) -> Result<()> {
    let id = pci.bar(0)?.read32(OMNI_ID)?;
    if id != OMNI_ID_RESET {
        return Err(Error::Device(format!(
            "{}: ID reads {id:#010x}, not {OMNI_ID_RESET:#010x} — not an omnimeter shell",
            pci.bdf()
        )));
    }
    Ok(())
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

    fn dma_read(&self, buf: &mut DmaBuf, len: usize, timeout: Duration) -> Result<()> {
        let bar1 = self.ready()?;
        self.dma
            .transfer(bar1, &|n| self.pci.alloc(n), Data::C2h(buf), len, timeout)
    }

    fn dma_write(&self, buf: &DmaBuf, len: usize, timeout: Duration) -> Result<()> {
        let bar1 = self.ready()?;
        self.dma
            .transfer(bar1, &|n| self.pci.alloc(n), Data::H2c(buf), len, timeout)
    }

    fn wait_irq(&self, vector: u32, timeout: Duration) -> Result<()> {
        self.ready()?;
        self.pci.wait_irq(vector, timeout).map(|_| ())
    }
}
