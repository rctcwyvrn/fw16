//! c2h: the card's source streams the pattern through the ring to the host,
//! which checks every beat.

use std::time::{Duration, Instant};

use omni::pattern::{BEAT_BYTES, Checker, Mode};
use omni::regs::*;
use omni::{Result, Transport, counters};

use crate::DMA_TIMEOUT;
use crate::args::Path;
use crate::report::Report;

pub fn run(t: &dyn Transport, pattern: Mode, duration: Duration, chunk: usize) -> Result<Report> {
    // Quiesce, then clear source and ring so the stream starts at beat 0.
    t.write32(OMNI_SRC_CTL, 0)?;
    t.write32(OMNI_RING_CTL, 0)?;
    t.write32(OMNI_SRC_CLEAR, 1)?;
    t.write32(OMNI_RING_CLEAR, 1)?;
    t.write32(OMNI_SRC_RATE, OMNI_SRC_RATE_RESET)?; // full rate
    t.write32(
        OMNI_SRC_CTL,
        OMNI_SRC_CTL_EN_MASK | pattern.bit() << OMNI_SRC_CTL_MODE_SHIFT,
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
    t.write32(OMNI_SRC_CTL, 0)?;
    result?;

    let mut notes = Vec::new();
    let sent = counters::read_u64(t, OMNI_SRC_BEATS_LO, OMNI_SRC_BEATS_HI)?;
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
