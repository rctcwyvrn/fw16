//! `fw16-fpga-loopback` as a library, so the mode runners can be tested against a
//! fault-injecting [`fw16_fpga::Mock`] as well as run from the command line.

pub mod args;
pub mod c2h;
pub mod h2c;
pub mod irq_latency;
pub mod report;
pub mod ring_loop;
mod sink_tally;

use std::time::Duration;

/// Per-transfer timeout: generous, since a real stall means a dead link.
pub const DMA_TIMEOUT: Duration = Duration::from_secs(1);
