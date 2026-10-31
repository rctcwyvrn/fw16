//! The regmap v0.3 IRQ block (`gateware/shell/irq.ml`).
//! `IRQ_PENDING & IRQ_EN` is the shell's `usr_irq_req[0]`; XDMA
//! sends one MSI-X message per rising edge of it, so a source that stays
//! pending does not interrupt again until the host clears it (RW1C) and it
//! is raised anew. `SLOT` (bit 3) is the slot's `slot_irq` *level*: it
//! re-latches every cycle the level is high, so its clear sticks only once
//! the project has dropped it.

use crate::regs::*;

const SOURCES: u32 = FW16_FPGA_IRQ_EN_FORCE_MASK
    | FW16_FPGA_IRQ_EN_PPS_MASK
    | FW16_FPGA_IRQ_EN_SNK_ERR_MASK
    | FW16_FPGA_IRQ_EN_SLOT_MASK;

#[derive(Default)]
pub(super) struct Irq {
    en: u32,
    pending: u32,
    count: u32,
    line: bool,
    /// The slot's `slot_irq` level.
    slot_irq: bool,
    /// Messages sent to vector 0 and not yet consumed by `wait_irq`.
    undelivered: u32,
}

impl Irq {
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            FW16_FPGA_IRQ_EN => self.en,
            FW16_FPGA_IRQ_PENDING => self.pending,
            FW16_FPGA_IRQ_COUNT => self.count,
            _ => 0,
        }
    }

    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            FW16_FPGA_IRQ_EN => self.en = value,
            FW16_FPGA_IRQ_PENDING => self.pending &= !value,
            FW16_FPGA_IRQ_FORCE => self.pending |= value & FW16_FPGA_IRQ_EN_FORCE_MASK,
            _ => {}
        }
        self.update();
    }

    /// Latch source bits (an `IRQ_EN`-layout mask) into `IRQ_PENDING`.
    pub fn raise(&mut self, sources: u32) {
        self.pending |= sources & SOURCES;
        self.update();
    }

    /// Drive the slot's `slot_irq` level. Only tests drive it: the mock's
    /// echo project has no interrupt.
    #[cfg(test)]
    pub fn set_slot_irq(&mut self, level: bool) {
        self.slot_irq = level;
        self.update();
    }

    fn update(&mut self) {
        // The level re-latches after any clear (the next cycle on the card).
        if self.slot_irq {
            self.pending |= FW16_FPGA_IRQ_PENDING_SLOT_MASK;
        }
        let line = self.pending & self.en != 0;
        if line && !self.line {
            self.count = self.count.wrapping_add(1);
            self.undelivered += 1;
        }
        self.line = line;
    }

    /// Consume one delivered message, if any.
    pub fn take(&mut self) -> bool {
        let any = self.undelivered > 0;
        if any {
            self.undelivered -= 1;
        }
        any
    }
}
