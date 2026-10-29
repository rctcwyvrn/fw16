//! c2h: the card's source streams the pattern through the ring to the host,
//! which checks every beat.

use std::time::{Duration, Instant};

use fw16_fpga::pattern::{BEAT_BYTES, Checker, Mode};
use fw16_fpga::regs::*;
use fw16_fpga::{Result, Transport, counters};

use crate::DMA_TIMEOUT;
use crate::args::Path;
use crate::report::Report;

pub fn run(t: &dyn Transport, pattern: Mode, duration: Duration, chunk: usize) -> Result<Report> {
    // Quiesce, then clear source and ring so the stream starts at beat 0.
    t.write32(FW16_FPGA_SRC_CTL, 0)?;
    t.write32(FW16_FPGA_RING_CTL, 0)?;
    t.write32(FW16_FPGA_SRC_CLEAR, 1)?;
    t.write32(FW16_FPGA_RING_CLEAR, 1)?;
    t.write32(FW16_FPGA_SRC_RATE, FW16_FPGA_SRC_RATE_RESET)?; // full rate
    t.write32(
        FW16_FPGA_SRC_CTL,
        FW16_FPGA_SRC_CTL_EN_MASK | pattern.bit() << FW16_FPGA_SRC_CTL_MODE_SHIFT,
    )?;

    let mut buf = t.alloc(chunk)?;
    let mut checker = Checker::new(pattern);
    let mut bytes = 0u64;
    let start = Instant::now();
    let result = loop {
        if start.elapsed() >= duration {
            break Ok(());
        }
        if let Err(e) = t.dma_read(&mut buf, chunk, DMA_TIMEOUT) {
            break Err(e);
        }
        checker.check(buf.words());
        bytes += chunk as u64;
    };
    let elapsed = start.elapsed();
    t.write32(FW16_FPGA_SRC_CTL, 0)?;
    result?;

    let mut notes = Vec::new();
    let sent = counters::read_u64(t, FW16_FPGA_SRC_BEATS_LO, FW16_FPGA_SRC_BEATS_HI)?;
    let received = bytes / BEAT_BYTES as u64;
    if sent < received {
        notes.push(format!("SRC_BEATS {sent} < {received} beats received"));
    }
    Ok(Report {
        path: Path::C2h,
        pattern,
        bytes,
        elapsed,
        errors: u64::from(checker.errors()) + u64::from(sent < received),
        notes,
    })
}
