//! Interrupt latency: IRQ_FORCE → `wait_irq`, n times. On a map without
//! the IRQ block nothing fires; that is reported as "no interrupt", not as a
//! failure of the run.
//!
//! The wait is on the vector itself (not `fw16_fpga::irq::wait_pending`, which
//! returns as soon as FORCE reads pending, before any message), and after
//! each message `irq::drain` clears IRQ_PENDING until it reads 0, so the
//! next FORCE is a fresh rising edge of `usr_irq_req`.

use std::time::{Duration, Instant};

use fw16_fpga::regs::*;
use fw16_fpga::{Error, Result, Transport, irq};

use crate::report::IrqLatency;

const VECTOR: u32 = 0;
const TIMEOUT: Duration = Duration::from_millis(100);

/// `Ok(None)` if the first forced interrupt never arrives.
pub fn run(t: &dyn Transport, n: u32) -> Result<Option<IrqLatency>> {
    t.write32(FW16_FPGA_IRQ_EN, 0)?;
    t.write32(FW16_FPGA_IRQ_PENDING, irq::SHELL_SOURCES)?;
    t.write32(FW16_FPGA_IRQ_EN, FW16_FPGA_IRQ_EN_FORCE_MASK)?;
    let mut samples = Vec::with_capacity(n as usize);
    let mut result = Ok(());
    for _ in 0..n {
        let t0 = Instant::now();
        t.write32(FW16_FPGA_IRQ_FORCE, 1)?;
        match t.wait_irq(VECTOR, TIMEOUT) {
            Ok(()) => samples.push(t0.elapsed()),
            Err(Error::Timeout { .. }) if samples.is_empty() => break,
            Err(e) => {
                result = Err(e);
                break;
            }
        }
        if let Err(e) = irq::drain(t) {
            result = Err(e);
            break;
        }
    }
    t.write32(FW16_FPGA_IRQ_EN, 0)?;
    t.write32(FW16_FPGA_IRQ_PENDING, irq::SHELL_SOURCES)?;
    result?;
    Ok(if samples.is_empty() {
        None
    } else {
        Some(IrqLatency { samples })
    })
}
