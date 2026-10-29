//! One XDMA channel (H2C 0 or C2H 0) driving one transfer at a time.
//!
//! A transfer: lay the chain out in the channel's own chain buffer, zero the
//! writebacks, program SGDMA with the first descriptor, set RUN with
//! poll-mode writeback, then watch the completed-descriptor count the
//! engine writes into host memory (polling RAM, not MMIO). Completion is
//! polled, not interrupt-driven: no channel interrupt is enabled.
//!
//! Whatever happens — done, error, timeout, abort — the channel is stopped
//! (RUN cleared, BUSY seen low, STATUS_RC read) before `transfer` returns,
//! so the device is no longer touching a buffer the caller gets back.

use std::time::{Duration, Instant};

use super::desc::{self, Desc, Dir, Layout};
use super::port::{DeviceMem, Port};
use super::regs::*;
use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};

/// The data side of a transfer: H2C reads the caller's buffer, C2H writes it.
pub enum Data<'a> {
    H2c(&'a DmaBuf),
    C2h(&'a mut DmaBuf),
}

impl Data<'_> {
    fn dir(&self) -> Dir {
        match self {
            Data::H2c(_) => Dir::H2c,
            Data::C2h(_) => Dir::C2h,
        }
    }

    fn buf(&self) -> &DmaBuf {
        match self {
            Data::H2c(b) => b,
            Data::C2h(b) => b,
        }
    }
}

/// Polls of BUSY allowed while stopping (each one a `Port::wait`).
const STOP_POLLS: u32 = 1000;

pub struct Channel {
    dir: Dir,
    /// The chain; grown (never shrunk) to the largest transfer seen.
    descs: Option<DmaBuf>,
    /// C2H result records, one per descriptor; same growth.
    results: Option<DmaBuf>,
    /// Poll-mode writeback: the u32 at byte 0.
    wb: Option<DmaBuf>,
}

impl Channel {
    pub fn new(dir: Dir) -> Channel {
        Channel {
            dir,
            descs: None,
            results: None,
            wb: None,
        }
    }

    fn chan(&self) -> u32 {
        match self.dir {
            Dir::H2c => H2C_CHAN,
            Dir::C2h => C2H_CHAN,
        }
    }

    fn sgdma(&self) -> u32 {
        match self.dir {
            Dir::H2c => H2C_SGDMA,
            Dir::C2h => C2H_SGDMA,
        }
    }

    /// Make sure the channel's own buffers hold `n` descriptors.
    fn reserve(&mut self, n: usize, alloc: &dyn Fn(usize) -> Result<DmaBuf>) -> Result<()> {
        let fits = |b: &Option<DmaBuf>, bytes| b.as_ref().is_some_and(|b| b.len() >= bytes);
        if !fits(&self.descs, n * DESC_BYTES) {
            self.descs = None; // free (and unmap) before mapping the bigger one
            self.descs = Some(alloc(n * DESC_BYTES)?);
        }
        if self.dir == Dir::C2h && !fits(&self.results, n * RESULT_BYTES) {
            self.results = None;
            self.results = Some(alloc(n * RESULT_BYTES)?);
        }
        if self.wb.is_none() {
            self.wb = Some(alloc(32)?);
        }
        Ok(())
    }

    /// Move `len` bytes of `data` (whole beats, at most its length). On a
    /// timeout returns `Error::Timeout { done }`, where `done` counts whole
    /// descriptors only: progress rounds down to [`desc::DATA_PER_DESC`].
    pub fn transfer(
        &mut self,
        port: &mut dyn Port,
        alloc: &dyn Fn(usize) -> Result<DmaBuf>,
        mut data: Data<'_>,
        len: usize,
        timeout: Duration,
    ) -> Result<()> {
        assert_eq!(data.dir(), self.dir, "transfer on the wrong channel");
        data.buf().check_len(len)?;
        let iova = |b: &DmaBuf| {
            b.iova()
                .ok_or(Error::InvalidArgument("buffer has no bus address".into()))
        };
        let n = desc::count(len);
        self.reserve(n, alloc)?;
        let layout = Layout {
            dir: self.dir,
            data: iova(data.buf())?,
            len,
            descs: iova(self.descs.as_ref().unwrap())?,
            results: match &self.results {
                Some(r) => iova(r)?,
                None => 0,
            },
        };
        self.load(&desc::chain(&layout));

        let deadline = Instant::now() + timeout;
        self.start(port, &layout)?;
        let outcome = self.watch(port, &mut data, n, deadline);
        // Count before stopping: RUN rising is what resets the register.
        let completed = port.read32(self.chan() + CHAN_COMPLETED_DESC);
        let status = self.stop(port, &mut data)?;
        // An engine error outranks whatever the watch saw (a WB error flag,
        // a timeout because the engine halted).
        let finished = match outcome {
            Ok(_) | Err(Error::Device(_)) if status & STAT_ERRORS != 0 => {
                return Err(self.status_error(status));
            }
            o => o?,
        };
        match finished {
            true => {
                if self.dir == Dir::C2h {
                    self.check_results(&layout, n)?;
                }
                Ok(())
            }
            false => {
                let k = (completed? & WB_COUNT_MASK) as usize;
                Err(Error::Timeout {
                    done: desc::bytes_done(len, k.min(n)),
                })
            }
        }
    }

    /// Write the chain and clear every writeback, with the channel stopped.
    fn load(&mut self, chain: &[Desc]) {
        let descs = self.descs.as_mut().unwrap().words_mut();
        for (d, w) in chain.iter().zip(descs.as_chunks_mut::<4>().0) {
            *w = d.encode();
        }
        if let Some(r) = &mut self.results {
            r.words_mut()[..chain.len() * RESULT_BYTES / 8].fill(0);
        }
        self.wb.as_mut().unwrap().words_mut().fill(0);
    }

    fn start(&self, port: &mut dyn Port, l: &Layout) -> Result<()> {
        let (chan, sg) = (self.chan(), self.sgdma());
        let wb = self.wb.as_ref().unwrap().iova().unwrap();
        port.write32(chan + CHAN_CONTROL, 0)?;
        port.read32(chan + CHAN_STATUS_RC)?;
        port.write32(chan + CHAN_INTR_ENABLE, 0)?;
        port.write32(sg + SGDMA_DESC_LO, l.descs as u32)?;
        port.write32(sg + SGDMA_DESC_HI, (l.descs >> 32) as u32)?;
        port.write32(sg + SGDMA_DESC_ADJ, desc::first_adj(desc::count(l.len)))?;
        port.write32(chan + CHAN_POLL_WB_LO, wb as u32)?;
        port.write32(chan + CHAN_POLL_WB_HI, (wb >> 32) as u32)?;
        // C2H stream writeback stays on (CTRL_C2H_STREAM_WB_DISABLE = 0).
        port.write32(chan + CHAN_CONTROL, CTRL_RUN | CTRL_POLL_MODE_WB)
    }

    fn mem<'a>(&'a mut self, data: &'a mut Data<'_>) -> DeviceMem<'a> {
        let mut mem = DeviceMem::new();
        mem = match data {
            Data::H2c(b) => mem.ro(b),
            Data::C2h(b) => mem.rw(b),
        };
        for b in [&mut self.descs, &mut self.results, &mut self.wb]
            .into_iter()
            .flatten()
        {
            mem = mem.rw(b);
        }
        mem
    }

    /// Wait for the writeback to count `n`. `Ok(true)` when done, `Ok(false)`
    /// on timeout (or when the device can do no more).
    fn watch(
        &mut self,
        port: &mut dyn Port,
        data: &mut Data<'_>,
        n: usize,
        deadline: Instant,
    ) -> Result<bool> {
        let mut more = true;
        loop {
            let wb = self.wb.as_ref().unwrap().read_volatile_u32(0);
            if wb & WB_ERR != 0 {
                return Err(Error::Device(format!(
                    "{:?} engine reported an error after {} descriptors",
                    self.dir,
                    wb & WB_COUNT_MASK
                )));
            }
            if (wb & WB_COUNT_MASK) as usize >= n {
                return Ok(true);
            }
            if !more || Instant::now() >= deadline {
                return Ok(false);
            }
            more = port.wait(&self.mem(data), deadline)?;
        }
    }

    /// Clear RUN and wait for BUSY to drop; returns the status flags the
    /// transfer left (read-and-cleared).
    fn stop(&mut self, port: &mut dyn Port, data: &mut Data<'_>) -> Result<u32> {
        let chan = self.chan();
        port.write32(chan + CHAN_CONTROL, 0)?;
        let mut polls = 0;
        while port.read32(chan + CHAN_STATUS)? & STAT_BUSY != 0 {
            polls += 1;
            if polls > STOP_POLLS {
                return Err(Error::Device(format!(
                    "{:?} engine still busy after RUN was cleared",
                    self.dir
                )));
            }
            let soon = Instant::now() + Duration::from_millis(1);
            port.wait(&self.mem(data), soon)?;
        }
        port.read32(chan + CHAN_STATUS_RC)
    }

    fn status_error(&self, status: u32) -> Error {
        let mut what = Vec::new();
        for (bits, name) in [
            (STAT_ALIGN_MISMATCH, "align mismatch"),
            (STAT_MAGIC_STOPPED, "bad descriptor magic"),
            (STAT_INVALID_LENGTH, "invalid length"),
            (STAT_READ_ERROR, "read error"),
            (STAT_WRITE_ERROR, "write error"),
            (STAT_DESC_ERROR, "descriptor error"),
        ] {
            if status & bits != 0 {
                what.push(name);
            }
        }
        Error::Device(format!(
            "{:?} engine status {status:#010x}: {}",
            self.dir,
            what.join(", ")
        ))
    }

    /// Every C2H result record must carry the magic and the full length:
    /// the shell never asserts `tlast` (`ring_buffer.ml` ties it low), so no
    /// descriptor may end short.
    fn check_results(&self, l: &Layout, n: usize) -> Result<()> {
        let r = self.results.as_ref().unwrap();
        for i in 0..n {
            let status = r.read_volatile_u32(i * RESULT_BYTES);
            let length = r.read_volatile_u32(i * RESULT_BYTES + 4);
            let want = desc::bytes_done(l.len, i + 1) - desc::bytes_done(l.len, i);
            if status >> 16 != RESULT_MAGIC {
                return Err(Error::Device(format!(
                    "C2H result {i} has no writeback ({status:#010x}); \
                     is C2H CONTROL bit 27 (stream writeback disable) set?"
                )));
            }
            if length as usize != want {
                return Err(Error::Device(format!(
                    "C2H descriptor {i} moved {length} bytes, not {want}"
                )));
            }
        }
        Ok(())
    }
}
