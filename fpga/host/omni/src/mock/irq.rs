//! The regmap v0.2 IRQ block (`gateware/shell/irq.ml`).
//! `IRQ_PENDING & IRQ_EN` is the shell's `usr_irq_req[0]`; XDMA
//! sends one MSI-X message per rising edge of it, so a source that stays
//! pending does not interrupt again until the host clears it (RW1C) and it
//! is raised anew.

use crate::regs::*;

const SOURCES: u32 = OMNI_IRQ_EN_FORCE_MASK | OMNI_IRQ_EN_PPS_MASK | OMNI_IRQ_EN_SNK_ERR_MASK;

#[derive(Default)]
pub(super) struct Irq {
    en: u32,
    pending: u32,
    count: u32,
    line: bool,
    /// Messages sent to vector 0 and not yet consumed by `wait_irq`.
    undelivered: u32,
}

impl Irq {
    pub fn read(&self, offset: u32) -> u32 {
        match offset {
            OMNI_IRQ_EN => self.en,
            OMNI_IRQ_PENDING => self.pending,
            OMNI_IRQ_COUNT => self.count,
            _ => 0,
        }
    }

    pub fn write(&mut self, offset: u32, value: u32) {
        match offset {
            OMNI_IRQ_EN => self.en = value,
            OMNI_IRQ_PENDING => self.pending &= !value,
            OMNI_IRQ_FORCE => self.pending |= value & OMNI_IRQ_EN_FORCE_MASK,
            _ => {}
        }
        self.update();
    }

    /// Latch source bits (an `IRQ_EN`-layout mask) into `IRQ_PENDING`.
    pub fn raise(&mut self, sources: u32) {
        self.pending |= sources & SOURCES;
        self.update();
    }

    fn update(&mut self) {
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
