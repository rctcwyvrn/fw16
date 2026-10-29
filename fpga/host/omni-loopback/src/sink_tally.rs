//! The card sink's verdict after an H2C run: its error count, plus an error
//! if it saw a different number of beats than the host sent (a sink that
//! was not enabled, or dropped or duplicated beats).

use omni::pattern::BEAT_BYTES;
use omni::regs::*;
use omni::{Result, Transport, counters};

pub fn read(t: &mut dyn Transport, bytes_sent: u64, notes: &mut Vec<String>) -> Result<u64> {
    let errs = t.read32(OMNI_SNK_ERRS)?;
    let seen = counters::read_u64(t, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI)?;
    let sent = bytes_sent / BEAT_BYTES as u64;
    if errs != 0 {
        notes.push(format!("card sink: SNK_ERRS {errs}"));
    }
    let mismatch = seen != sent;
    if mismatch {
        notes.push(format!("card sink saw {seen} beats, host sent {sent}"));
    }
    Ok(u64::from(errs) + u64::from(mismatch))
}
