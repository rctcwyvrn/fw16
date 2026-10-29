//! What one run measured, and how it prints.

use std::fmt;
use std::time::Duration;

use fw16_fpga::pattern::Mode;

use crate::args::Path;

#[derive(Debug)]
pub struct Report {
    pub path: Path,
    pub pattern: Mode,
    /// Payload bytes moved (in loop mode each crosses the link twice).
    pub bytes: u64,
    pub elapsed: Duration,
    /// Pattern errors from every checker involved, plus beat-count mismatches.
    pub errors: u64,
    pub notes: Vec<String>,
}

impl Report {
    pub fn gb_per_s(&self) -> f64 {
        self.bytes as f64 / self.elapsed.as_secs_f64().max(1e-9) / 1e9
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = match self.path {
            Path::C2h => "c2h",
            Path::H2c => "h2c",
            Path::Loop => "loop",
        };
        write!(
            f,
            "{path:<4}  {:?}  {} bytes in {:.3} s  {:.3} GB/s  errors {}",
            self.pattern,
            self.bytes,
            self.elapsed.as_secs_f64(),
            self.gb_per_s(),
            self.errors
        )?;
        for n in &self.notes {
            write!(f, "\n      note: {n}")?;
        }
        Ok(())
    }
}

/// Forced-interrupt round trips: IRQ_FORCE write → `wait_irq` return.
#[derive(Debug)]
pub struct IrqLatency {
    pub samples: Vec<Duration>,
}

impl fmt::Display for IrqLatency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.samples.clone();
        s.sort();
        let us = |d: Duration| d.as_secs_f64() * 1e6;
        match (s.first(), s.last()) {
            (Some(&min), Some(&max)) => write!(
                f,
                "irq   {} forced  latency min {:.1} us  median {:.1} us  max {:.1} us",
                s.len(),
                us(min),
                us(s[s.len() / 2]),
                us(max)
            ),
            _ => write!(f, "irq   no samples"),
        }
    }
}
