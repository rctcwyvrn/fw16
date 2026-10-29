//! The VFIO userspace driver layer (decision 0009): no kernel module; the
//! card is bound to vfio-pci and driven from here.
//!
//! - [`VfioPci`] — any vfio-pci function: BARs, pinned DMA buffers,
//!   MSI/MSI-X eventfds. Tested against QEMU's `edu` device in a VM
//!   (`host/vm/`, `examples/edu_check.rs`).
//! - [`Card`] — the fw16-fpga card on top of it: the `vfio:<BDF>` transport.
//!
//! The other files are its parts: `bdf` (addresses, sysfs checks), `bar`
//! (one mapped BAR), `iova` (bus-address allocator), `pinned` (IOMMU-mapped
//! memory), `irq` (eventfds), `card_dma` (the card's XDMA channels and
//! their suspend handling).

mod bar;
mod bdf;
mod card;
mod card_dma;
mod iova;
mod irq;
mod pci;
mod pinned;

pub use bar::Bar;
pub use bdf::Bdf;
pub use card::Card;
pub use irq::IrqKind;
pub use pci::VfioPci;
pub(crate) use pinned::Pinned;
