//! SG descriptors: encoding one, and laying out the chain for one transfer.
//!
//! A transfer is one IOVA-contiguous buffer (every `DmaBuf` is), cut into
//! [`DATA_PER_DESC`] pieces. The chain sits in its own buffer, descriptor i
//! at `descs + 32 i`, each `next` pointing at the following one. The last
//! carries STOP (that is what halts the engine) and COMPLETED; on H2C it
//! also carries EOP, so the stream sees `tlast` at the end of the transfer.
//!
//! `nxt_adj` tells the engine how many descriptors *after the next one* sit
//! contiguously behind it, so it can fetch them in one burst. A burst is at
//! most [`ADJ_BLOCK`] descriptors and never crosses a 4 KiB boundary; with
//! the chain buffer page-aligned, that means bursts stay inside aligned
//! 32-descriptor (1 KiB) blocks.

use super::regs::*;

/// Data bytes per descriptor. One page, like AMD's driver: small enough
/// that a timed-out transfer reports progress at page granularity (the
/// engine only reports whole descriptors), large enough that a 64 MiB
/// transfer needs a 512 KiB chain.
pub const DATA_PER_DESC: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    H2c,
    C2h,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Desc {
    /// `DESC_STOP | DESC_COMPLETED | DESC_EOP` subset.
    pub flags: u32,
    /// Descriptors contiguous after the one `next` points at.
    pub adj: u32,
    pub bytes: u32,
    pub src: u64,
    pub dst: u64,
    pub next: u64,
}

impl Desc {
    /// The 32 bytes the engine fetches, as four little-endian u64 words.
    pub fn encode(&self) -> [u64; 4] {
        let control = DESC_MAGIC << 16 | (self.adj << DESC_ADJ_SHIFT) & DESC_ADJ_MASK | self.flags;
        [
            u64::from(control) | u64::from(self.bytes) << 32,
            self.src,
            self.dst,
            self.next,
        ]
    }

    /// The inverse of [`encode`](Self::encode), plus the magic it carried.
    pub fn decode(w: [u64; 4]) -> (Desc, u32) {
        let control = w[0] as u32;
        let desc = Desc {
            flags: control & 0xFF,
            adj: (control & DESC_ADJ_MASK) >> DESC_ADJ_SHIFT,
            bytes: (w[0] >> 32) as u32,
            src: w[1],
            dst: w[2],
            next: w[3],
        };
        (desc, control >> 16)
    }
}

/// Where one transfer's pieces live on the bus.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub dir: Dir,
    /// The data buffer and the transfer length (whole beats, non-zero).
    pub data: u64,
    pub len: usize,
    /// The chain buffer; page-aligned.
    pub descs: u64,
    /// C2H only: one result record per descriptor; page-aligned.
    pub results: u64,
}

pub fn count(len: usize) -> usize {
    len.div_ceil(DATA_PER_DESC)
}

/// Bytes moved by the first `k` descriptors of a `len`-byte transfer.
pub fn bytes_done(len: usize, k: usize) -> usize {
    (k * DATA_PER_DESC).min(len)
}

/// The value for `SGDMA_DESC_ADJ`: descriptors contiguous after the first.
pub fn first_adj(n: usize) -> u32 {
    (ADJ_BLOCK - 1).min(n - 1) as u32
}

/// `nxt_adj` for descriptor `i` of `n`: how many follow descriptor `i + 1`
/// in the same aligned block.
fn next_adj(i: usize, n: usize) -> u32 {
    let next = i + 1;
    if next >= n {
        return 0;
    }
    (ADJ_BLOCK - 1 - next % ADJ_BLOCK).min(n - 1 - next) as u32
}

pub fn chain(l: &Layout) -> Vec<Desc> {
    assert!(l.len > 0, "empty transfer");
    assert!(
        l.descs.is_multiple_of(ADJ_BOUNDARY),
        "chain buffer not page-aligned"
    );
    let n = count(l.len);
    (0..n)
        .map(|i| {
            let off = (i * DATA_PER_DESC) as u64;
            let bytes = DATA_PER_DESC.min(l.len - i * DATA_PER_DESC) as u32;
            let last = i + 1 == n;
            let mut flags = 0;
            if last {
                flags |= DESC_STOP | DESC_COMPLETED;
                if l.dir == Dir::H2c {
                    flags |= DESC_EOP;
                }
            }
            let (src, dst) = match l.dir {
                // AXI-ST H2C ignores dst.
                Dir::H2c => (l.data + off, 0),
                // AXI-ST C2H: src is where this descriptor's result goes.
                Dir::C2h => (l.results + (i * RESULT_BYTES) as u64, l.data + off),
            };
            Desc {
                flags,
                adj: next_adj(i, n),
                bytes,
                src,
                dst,
                next: if last {
                    0
                } else {
                    l.descs + ((i + 1) * DESC_BYTES) as u64
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: u64 = 0x10_0000;
    const DESCS: u64 = 0x20_0000;
    const RESULTS: u64 = 0x30_0000;

    fn layout(dir: Dir, len: usize) -> Layout {
        Layout {
            dir,
            data: DATA,
            len,
            descs: DESCS,
            results: RESULTS,
        }
    }

    #[test]
    fn encodes_by_hand() {
        let d = Desc {
            flags: DESC_STOP | DESC_COMPLETED,
            adj: 5,
            bytes: 0x1000,
            src: 0x1_2345_6000,
            dst: 0,
            next: 0x20_0020,
        };
        // control = 0xAD4B << 16 | 5 << 8 | 0b11 = 0xAD4B_0503
        assert_eq!(
            d.encode(),
            [0x0000_1000_AD4B_0503, 0x1_2345_6000, 0, 0x20_0020]
        );
        assert_eq!(Desc::decode(d.encode()), (d, DESC_MAGIC));
    }

    #[test]
    fn h2c_chain_spanning_two_blocks() {
        // 40 descriptors: block 0 = 0..31, block 1 = 32..39; last is short.
        let len = 39 * DATA_PER_DESC + 64;
        let c = chain(&layout(Dir::H2c, len));
        assert_eq!(c.len(), 40);
        assert_eq!(first_adj(40), 31);
        // nxt_adj of i counts the descriptors after i+1 in i+1's block.
        assert_eq!(c[0].adj, 30); // next = 1: 2..31 follow
        assert_eq!(c[30].adj, 0); // next = 31, the block's last
        assert_eq!(c[31].adj, 7); // next = 32, block 1: 33..39 follow
        assert_eq!(c[38].adj, 0); // next = 39, the chain's last
        assert_eq!(c[39].adj, 0);
        assert_eq!(c[0].src, DATA);
        assert_eq!(c[0].next, DESCS + 32);
        assert_eq!(c[39].src, DATA + 39 * 4096);
        assert_eq!(c[39].bytes, 64);
        assert_eq!(c[39].next, 0);
        assert_eq!(c[39].flags, DESC_STOP | DESC_COMPLETED | DESC_EOP);
        assert!(c[..39].iter().all(|d| d.flags == 0 && d.dst == 0));
        // The encoded first and last, byte for byte.
        assert_eq!(c[0].encode()[0], 0x0000_1000_AD4B_1E00);
        assert_eq!(c[39].encode()[0], 0x0000_0040_AD4B_0013);
    }

    #[test]
    fn c2h_points_src_at_result_records() {
        let c = chain(&layout(Dir::C2h, 3 * DATA_PER_DESC));
        assert_eq!(first_adj(3), 2);
        assert_eq!([c[0].adj, c[1].adj, c[2].adj], [1, 0, 0]);
        for (i, d) in c.iter().enumerate() {
            assert_eq!(d.src, RESULTS + 32 * i as u64);
            assert_eq!(d.dst, DATA + 4096 * i as u64);
        }
        assert_eq!(c[2].flags, DESC_STOP | DESC_COMPLETED, "no EOP on C2H");
    }

    #[test]
    fn one_descriptor_chain() {
        let c = chain(&layout(Dir::H2c, 32));
        assert_eq!(c.len(), 1);
        assert_eq!(first_adj(1), 0);
        assert_eq!(c[0].adj, 0);
        assert_eq!(c[0].bytes, 32);
    }

    #[test]
    fn bursts_never_cross_a_page() {
        // Every burst the engine may fetch (first: 1 + first_adj; then from
        // i+1: 1 + adj[i]) stays inside one 4 KiB page of the chain buffer.
        for n in [1, 2, 31, 32, 33, 127, 128, 129, 300] {
            let c = chain(&layout(Dir::H2c, n * DATA_PER_DESC));
            let burst = |start: usize, adj: u32| {
                let first = DESCS + (start * 32) as u64;
                let last = first + u64::from(adj) * 32;
                assert_eq!(first / 4096, last / 4096, "n={n} start={start}");
                assert!(start + (adj as usize) < n, "n={n} past the end");
            };
            burst(0, first_adj(n));
            for (i, d) in c.iter().enumerate().take(n - 1) {
                burst(i + 1, d.adj);
            }
        }
    }

    #[test]
    fn progress_rounds_to_descriptors() {
        let len = 2 * DATA_PER_DESC + 32;
        assert_eq!(count(len), 3);
        assert_eq!(bytes_done(len, 0), 0);
        assert_eq!(bytes_done(len, 2), 8192);
        assert_eq!(bytes_done(len, 3), len);
    }
}
