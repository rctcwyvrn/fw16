//! QEMU's `edu` teaching device (docs/specs/edu.rst in QEMU), as far as the
//! check needs it: identity, liveness, interrupt raise/ack, and its DMA
//! engine, which copies between host memory (an IOVA) and a 4 KiB buffer
//! inside the device at 0x40000.

use std::time::{Duration, Instant};

use fw16_fpga::vfio::{Bar, VfioPci};
use fw16_fpga::{Error, Result};

pub const VENDOR: u16 = 0x1234;
pub const DEVICE: u16 = 0x11e8;

const ID: u32 = 0x00;
const LIVENESS: u32 = 0x04;
const IRQ_STATUS: u32 = 0x24;
const IRQ_RAISE: u32 = 0x60;
const IRQ_ACK: u32 = 0x64;
const DMA_SRC: u32 = 0x80;
const DMA_DST: u32 = 0x88;
const DMA_COUNT: u32 = 0x90;
const DMA_CMD: u32 = 0x98;

const DMA_CMD_START: u32 = 1 << 0;
const DMA_CMD_TO_RAM: u32 = 1 << 1;
const DMA_CMD_IRQ: u32 = 1 << 2;
/// The interrupt-status bit a finished DMA sets.
pub const IRQ_DMA_DONE: u32 = 0x100;

/// Device-side address of edu's DMA buffer, and its size.
pub const DEV_BUF: u64 = 0x40000;
pub const DEV_BUF_LEN: usize = 4096;

pub struct Edu<'a> {
    pub pci: &'a VfioPci,
    bar: &'a Bar,
}

impl<'a> Edu<'a> {
    pub fn new(pci: &'a VfioPci) -> Result<Edu<'a>> {
        Ok(Edu {
            pci,
            bar: pci.bar(0)?,
        })
    }

    /// `0xRRrr00ed`: major, minor, then the constant 0xed.
    pub fn id(&self) -> Result<u32> {
        self.bar.read32(ID)
    }

    /// edu answers the bitwise inverse of what was written.
    pub fn liveness(&self, v: u32) -> Result<u32> {
        self.bar.write32(LIVENESS, v)?;
        self.bar.read32(LIVENESS)
    }

    pub fn raise(&self, bits: u32) -> Result<()> {
        self.bar.write32(IRQ_RAISE, bits)
    }

    pub fn status(&self) -> Result<u32> {
        self.bar.read32(IRQ_STATUS)
    }

    pub fn ack(&self, bits: u32) -> Result<()> {
        self.bar.write32(IRQ_ACK, bits)
    }

    /// Run one DMA and wait for its completion interrupt on MSI vector 0.
    /// Addresses fit in 32 bits here (the IOVA window is low), and edu
    /// takes a 32-bit write as the whole 64-bit address.
    pub fn dma(&self, src: u64, dst: u64, len: usize, to_ram: bool) -> Result<Duration> {
        let addr = |a: u64| {
            u32::try_from(a).map_err(|_| Error::InvalidArgument(format!("{a:#x} > 32 bits")))
        };
        self.bar.write32(DMA_SRC, addr(src)?)?;
        self.bar.write32(DMA_DST, addr(dst)?)?;
        self.bar.write32(DMA_COUNT, len as u32)?;
        let dir = if to_ram { DMA_CMD_TO_RAM } else { 0 };
        let t0 = Instant::now();
        self.bar
            .write32(DMA_CMD, DMA_CMD_START | DMA_CMD_IRQ | dir)?;
        // edu finishes on a 100 ms timer; TCG can stretch that.
        self.pci.wait_irq(0, Duration::from_secs(10))?;
        let took = t0.elapsed();
        if self.status()? & IRQ_DMA_DONE == 0 {
            return Err(Error::Device(
                "DMA interrupt without the DMA-done status bit".into(),
            ));
        }
        self.ack(IRQ_DMA_DONE)?;
        if self.bar.read32(DMA_CMD)? & DMA_CMD_START != 0 {
            return Err(Error::Device(
                "DMA still running after its interrupt".into(),
            ));
        }
        Ok(took)
    }
}
