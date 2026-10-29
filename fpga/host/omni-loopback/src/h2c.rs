//! h2c: the host streams the pattern to the card, whose sink checks it.

use std::time::{Duration, Instant};

use omni::pattern::{Generator, Mode};
use omni::regs::*;
use omni::{Result, Transport};

use crate::DMA_TIMEOUT;
use crate::args::Path;
use crate::report::Report;
use crate::sink_tally;

pub fn run(t: &dyn Transport, pattern: Mode, duration: Duration, chunk: usize) -> Result<Report> {
    t.write32(OMNI_SNK_CTL, 0)?;
    t.write32(OMNI_RING_CTL, 0)?; // H2C to the sink only
    t.write32(OMNI_SNK_CLEAR, 1)?;
    // The sink counts only while enabled, so enable before the first beat.
    t.write32(
        OMNI_SNK_CTL,
        OMNI_SNK_CTL_EN_MASK | pattern.bit() << OMNI_SNK_CTL_MODE_SHIFT,
    )?;

    let mut buf = t.alloc(chunk)?;
    let mut generator = Generator::new(pattern);
    let mut bytes = 0u64;
    let start = Instant::now();
    let result = loop {
        if start.elapsed() >= duration {
            break Ok(());
        }
        generator.fill(buf.words_mut());
        if let Err(e) = t.dma_write(&buf, chunk, DMA_TIMEOUT) {
            break Err(e);
        }
        bytes += chunk as u64;
    };
    let elapsed = start.elapsed();
    result?;

    let mut notes = Vec::new();
    let errors = sink_tally::read(t, bytes, &mut notes)?;
    t.write32(OMNI_SNK_CTL, 0)?;
    Ok(Report {
        path: Path::H2c,
        pattern,
        bytes,
        elapsed,
        errors,
        notes,
    })
}
