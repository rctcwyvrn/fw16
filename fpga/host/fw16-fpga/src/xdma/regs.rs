//! XDMA register map (BAR1, the "config BAR"), descriptor and writeback
//! formats — the subset this engine uses, from PG195 (DMA/Bridge Subsystem
//! for PCIe, "Register Space"). Offsets and bit positions were cross-checked
//! against upstream Linux `drivers/dma/xilinx/xdma-regs.h` and AMD's
//! `libxdma.h` (both read for facts only; no code taken).
//!
//! BAR1 address bits [15:12] select the target block; channel blocks repeat
//! every 0x100 bytes for channel 1..3 (we only use channel 0 of each).

/// Channel block bases (target 0 = H2C, 1 = C2H).
pub const H2C_CHAN: u32 = 0x0000;
pub const C2H_CHAN: u32 = 0x1000;
pub const IRQ_BLOCK: u32 = 0x2000;
/// SGDMA block bases: where a channel's first descriptor is programmed.
pub const H2C_SGDMA: u32 = 0x4000;
pub const C2H_SGDMA: u32 = 0x5000;

// --- Channel registers (offset from H2C_CHAN / C2H_CHAN) -----------------
/// `[31:20]` = 0x1FC, `[19:16]` = target, `[15]` = AXI-Stream, `[7:0]` version.
pub const CHAN_IDENTIFIER: u32 = 0x00;
pub const CHAN_CONTROL: u32 = 0x04;
/// Completion and error flags; `STATUS_RC` returns them and clears them.
pub const CHAN_STATUS: u32 = 0x40;
pub const CHAN_STATUS_RC: u32 = 0x44;
/// Descriptors completed since RUN last rose.
pub const CHAN_COMPLETED_DESC: u32 = 0x48;
pub const CHAN_POLL_WB_LO: u32 = 0x88;
pub const CHAN_POLL_WB_HI: u32 = 0x8C;
/// Channel interrupt enable mask (same layout as CONTROL's IE bits).
pub const CHAN_INTR_ENABLE: u32 = 0x90;

pub const ID_MAGIC: u32 = 0x1FC;
pub const ID_MAGIC_SHIFT: u32 = 20;
pub const ID_TARGET_SHIFT: u32 = 16;
pub const ID_TARGET_MASK: u32 = 0xF << ID_TARGET_SHIFT;
pub const ID_STREAM: u32 = 1 << 15;
pub const TARGET_H2C: u32 = 0;
pub const TARGET_C2H: u32 = 1;
pub const TARGET_IRQ: u32 = 2;

// CONTROL bits. The IE_* bits enable the matching STATUS bit as a channel
// interrupt; this engine polls, so it leaves them 0.
pub const CTRL_RUN: u32 = 1 << 0;
/// Write the completed-descriptor count to host memory (CHAN_POLL_WB_*)
/// as descriptors complete.
pub const CTRL_POLL_MODE_WB: u32 = 1 << 26;
/// C2H stream only: *disables* the per-descriptor result writeback. PG195
/// documents this bit for C2H AXI-ST; AMD's header names it
/// `XDMA_CTRL_STM_MODE_WB`. This engine leaves it 0 and checks every
/// result record, so a wrong reading shows up as a missing-record error.
pub const CTRL_C2H_STREAM_WB_DISABLE: u32 = 1 << 27;

// STATUS bits.
pub const STAT_BUSY: u32 = 1 << 0;
pub const STAT_DESC_STOPPED: u32 = 1 << 1;
pub const STAT_DESC_COMPLETED: u32 = 1 << 2;
pub const STAT_ALIGN_MISMATCH: u32 = 1 << 3;
pub const STAT_MAGIC_STOPPED: u32 = 1 << 4;
pub const STAT_INVALID_LENGTH: u32 = 1 << 5;
pub const STAT_IDLE_STOPPED: u32 = 1 << 6;
pub const STAT_READ_ERROR: u32 = 0x1F << 9;
pub const STAT_WRITE_ERROR: u32 = 0x1F << 14;
pub const STAT_DESC_ERROR: u32 = 0x1F << 19;
/// Everything that means the transfer went wrong.
pub const STAT_ERRORS: u32 = STAT_ALIGN_MISMATCH
    | STAT_MAGIC_STOPPED
    | STAT_INVALID_LENGTH
    | STAT_READ_ERROR
    | STAT_WRITE_ERROR
    | STAT_DESC_ERROR;

// --- SGDMA registers (offset from H2C_SGDMA / C2H_SGDMA) -----------------
pub const SGDMA_DESC_LO: u32 = 0x80;
pub const SGDMA_DESC_HI: u32 = 0x84;
/// Descriptors adjacent to (contiguous after) the first one.
pub const SGDMA_DESC_ADJ: u32 = 0x88;

// --- IRQ block (offset from IRQ_BLOCK) -----------------------------------
pub const IRQ_IDENTIFIER: u32 = 0x00;
/// Bit n enables `usr_irq_req[n]`.
pub const IRQ_USER_INT_EN: u32 = 0x04;
/// Bit order: H2C channels, then C2H channels.
pub const IRQ_CHAN_INT_EN: u32 = 0x10;
/// MSI-X vector of `usr_irq_req[n]`: 5 bits per byte, byte n%4 of word n/4.
pub const IRQ_USER_VEC_NUM: u32 = 0x80;

// --- Descriptor (32 bytes, little-endian u32s) ---------------------------
//   [0] control: magic[31:16] | nxt_adj[13:8] | flags[7:0]
//   [1] bytes (28 bits)   [2..3] src   [4..5] dst   [6..7] next
pub const DESC_BYTES: usize = 32;
pub const DESC_MAGIC: u32 = 0xAD4B;
pub const DESC_STOP: u32 = 1 << 0;
pub const DESC_COMPLETED: u32 = 1 << 1;
pub const DESC_EOP: u32 = 1 << 4;
pub const DESC_ADJ_SHIFT: u32 = 8;
pub const DESC_ADJ_MASK: u32 = 0x3F << DESC_ADJ_SHIFT;
/// The engine fetches up to this many contiguous descriptors in one block
/// (the Linux driver's limit, below PG195's 6-bit field) ...
pub const ADJ_BLOCK: usize = 32;
/// ... and a block never crosses this boundary.
pub const ADJ_BOUNDARY: u64 = 4096;

// --- Writebacks ----------------------------------------------------------
/// Poll-mode writeback (one u32): completed count, error flag.
pub const WB_COUNT_MASK: u32 = 0x00FF_FFFF;
pub const WB_ERR: u32 = 1 << 31;
/// C2H stream result record (32 bytes, one per descriptor, written where
/// the descriptor's *src* address points): [0] status, [1] length.
pub const RESULT_BYTES: usize = 32;
pub const RESULT_MAGIC: u32 = 0x52B4;
pub const RESULT_EOP: u32 = 1 << 0;
