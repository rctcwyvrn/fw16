//! `vfio:<BDF>` — the fw16-fpga card through [`VfioPci`]: BAR0 is the
//! shell's AXI-Lite registers, BAR1 the XDMA register space
//! (`gateware/vivado/xdma_0.tcl`), BAR2 the project slot's AXI4 bypass
//! port (decision 0013; `SLOT_ID` at 0x0, `SLOT_VERSION` at 0x4),
//! interrupts are MSI-X vectors, DMA is the XDMA engine ([`crate::xdma`])
//! on IOMMU-mapped buffers, and a logind delay inhibitor guards suspend
//! (`card_dma.rs` says how).
//!
//! BAR2 is optional at open: bitstreams built before the bypass interface
//! have none, and only [`Card::slot_bar`] asks for it. The BAR numbers are
//! VFIO region indices, so they hold only while BAR0 and BAR1 stay 32-bit
//! (a 64-bit BAR takes two indices and shifts the rest).

use std::sync::Arc;
use std::time::Duration;

use super::bar::Bar;
use super::bdf::Bdf;
use super::card_dma::CardDma;
use super::pci::VfioPci;
use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};
use crate::regs::{FW16_FPGA_ID, FW16_FPGA_ID_RESET};
use crate::sleep_inhibit::SleepInhibitor;
use crate::transport::Transport;
use crate::xdma::Data;

/// The bypass BAR (decision 0013).
const SLOT_BAR: usize = 2;

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
            &format!("fw16-fpga {bdf} DMA"),
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

    /// The project slot's registers (BAR2, the XDMA AXI4 bypass port), or
    /// `Error::Device` if this bitstream has no bypass BAR.
    pub fn slot_bar(&self) -> Result<&Bar> {
        self.pci.bar(SLOT_BAR)
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
    let id = pci.bar(0)?.read32(FW16_FPGA_ID)?;
    if id != FW16_FPGA_ID_RESET {
        return Err(Error::Device(format!(
            "{}: ID reads {id:#010x}, not {FW16_FPGA_ID_RESET:#010x} — not a fw16-fpga shell",
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
