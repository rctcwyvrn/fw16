//! Interrupt latency: IRQ_FORCE (regmap v0.2 proposal) → `wait_irq`, n times.
//! On a map without the IRQ block nothing fires; that is reported as "no
//! interrupt", not as a failure of the run.

use std::time::{Duration, Instant};

use omni::regs::proposed::*;
use omni::{Error, Result, Transport};

use crate::report::IrqLatency;

const VECTOR: u32 = 0;
const TIMEOUT: Duration = Duration::from_millis(100);

/// `Ok(None)` if the first forced interrupt never arrives.
pub fn run(t: &mut dyn Transport, n: u32) -> Result<Option<IrqLatency>> {
    t.write32(OMNI_IRQ_EN, 0)?;
    t.write32(OMNI_IRQ_PENDING, !0)?;
    t.write32(OMNI_IRQ_EN, OMNI_IRQ_EN_FORCE_MASK)?;
    let mut samples = Vec::with_capacity(n as usize);
    let mut result = Ok(());
    for _ in 0..n {
        let t0 = Instant::now();
        t.write32(OMNI_IRQ_FORCE, 1)?;
        match t.wait_irq(VECTOR, TIMEOUT) {
            Ok(()) => samples.push(t0.elapsed()),
            Err(Error::Timeout { .. }) if samples.is_empty() => break,
            Err(e) => {
                result = Err(e);
                break;
            }
        }
        t.write32(OMNI_IRQ_PENDING, OMNI_IRQ_EN_FORCE_MASK)?;
    }
    t.write32(OMNI_IRQ_EN, 0)?;
    t.write32(OMNI_IRQ_PENDING, !0)?;
    result?;
    Ok(if samples.is_empty() {
        None
    } else {
        Some(IrqLatency { samples })
    })
}
