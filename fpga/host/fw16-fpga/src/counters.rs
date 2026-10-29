//! Reading the shell's 64-bit counters through two 32-bit registers.
//!
//! Map v0.1 has no latch, so a carry between the two reads tears the value.
//! Read hi, lo, hi and retry until both hi reads agree. This stays correct
//! under the v0.2 proposal (reading LO latches HI): the second hi is then
//! the latched one, and the first may just cost one extra round.

use crate::error::{Error, Result};
use crate::transport::Transport;

const MAX_TRIES: usize = 8;

pub fn read_u64(t: &dyn Transport, lo: u32, hi: u32) -> Result<u64> {
    let mut hi1 = t.read32(hi)?;
    for _ in 0..MAX_TRIES {
        let lo_v = t.read32(lo)?;
        let hi2 = t.read32(hi)?;
        if hi1 == hi2 {
            return Ok((u64::from(hi2) << 32) | u64::from(lo_v));
        }
        hi1 = hi2;
    }
    Err(Error::Device(format!(
        "64-bit counter at {lo:#06x}/{hi:#06x}: high word did not settle in {MAX_TRIES} reads"
    )))
}
