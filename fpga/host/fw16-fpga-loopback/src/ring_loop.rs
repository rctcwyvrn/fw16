//! loop and slot: `RING_CTL.SRC_SEL` routes H2C into the ring (loop: `H2c`,
//! the v0.2 LOOPBACK) or through the project slot into the ring (slot:
//! `Slot`, which needs an echo project loaded and out of `SLOT_CTL.RESET`)
//! and back out over C2H, while the card's sink checks the H2C side and the
//! host checks what returns.
//!
//! Single-threaded write-then-read, so a chunk must fit in the ring: a
//! bigger write stalls (the ring backpressures H2C) with nothing draining
//! it. The chunk is clamped to RING_DEPTH. Concurrent H2C/C2H threads (the
//! `&self` transport allows them) are the real-card follow-up, once the
//! XDMA engine exists.

use std::time::{Duration, Instant};

use fw16_fpga::pattern::{BEAT_BYTES, Checker, Generator, Mode};
use fw16_fpga::regs::*;
use fw16_fpga::{Result, RingSrc, Transport};

use crate::DMA_TIMEOUT;
use crate::args::Path;
use crate::report::Report;
use crate::sink_tally;

/// `path` is `Path::Loop` or `Path::Slot`.
pub fn run(
    t: &dyn Transport,
    path: Path,
    pattern: Mode,
    duration: Duration,
    chunk: usize,
) -> Result<Report> {
    let src = match path {
        Path::Slot => RingSrc::Slot,
        _ => RingSrc::H2c,
    };
    t.write32(FW16_FPGA_SRC_CTL, 0)?;
    t.write32(FW16_FPGA_SNK_CTL, 0)?;
    if src == RingSrc::Slot {
        t.write32(FW16_FPGA_SLOT_CTL, 0)?;
    }
    t.write32(FW16_FPGA_RING_CTL, src.ring_ctl())?;
    t.write32(FW16_FPGA_RING_CLEAR, 1)?;
    t.write32(FW16_FPGA_SNK_CLEAR, 1)?;
    t.write32(
        FW16_FPGA_SNK_CTL,
        FW16_FPGA_SNK_CTL_EN_MASK | pattern.bit() << FW16_FPGA_SNK_CTL_MODE_SHIFT,
    )?;

    let mut notes = vec!["each byte crosses the link twice".to_string()];
    let ring_bytes = t.read32(FW16_FPGA_RING_DEPTH)? as usize * BEAT_BYTES;
    let chunk = if chunk > ring_bytes {
        notes.push(format!("chunk clamped to the ring, {ring_bytes} bytes"));
        ring_bytes
    } else {
        chunk
    };

    let mut out = t.alloc(chunk)?;
    let mut back = t.alloc(chunk)?;
    let mut generator = Generator::new(pattern);
    let mut checker = Checker::new(pattern);
    let mut bytes = 0u64;
    let start = Instant::now();
    let result = loop {
        if start.elapsed() >= duration {
            break Ok(());
        }
        generator.fill(out.words_mut());
        if let Err(e) = t.dma_write(&out, chunk, DMA_TIMEOUT) {
            break Err(e);
        }
        if let Err(e) = t.dma_read(&mut back, chunk, DMA_TIMEOUT) {
            break Err(e);
        }
        checker.check(back.words());
        bytes += chunk as u64;
    };
    let elapsed = start.elapsed();
    result?;

    let mut errors = sink_tally::read(t, bytes, &mut notes)?;
    if checker.errors() != 0 {
        notes.push(format!(
            "host checker: {} bad beats returned",
            checker.errors()
        ));
        errors += u64::from(checker.errors());
    }
    t.write32(FW16_FPGA_SNK_CTL, 0)?;
    t.write32(FW16_FPGA_RING_CTL, 0)?;
    Ok(Report {
        path,
        pattern,
        bytes,
        elapsed,
        errors,
        notes,
    })
}
