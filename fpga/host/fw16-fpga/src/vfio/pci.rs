//! `VfioPci`: one PCI function bound to vfio-pci, opened from userspace —
//! container → group → device, its BARs mapped, bus mastering on, DMA
//! buffers pinned into its IOMMU domain, interrupts as eventfds.
//!
//! Device-agnostic on purpose: the card's transport ([`super::card`]) and
//! the QEMU `edu` VM test both sit on it, so the VM test exercises exactly
//! the code the card will run.

use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use vfio_bindings::bindings::vfio::{
    VFIO_PCI_CONFIG_REGION_INDEX, VFIO_PCI_MSI_IRQ_INDEX, VFIO_PCI_MSIX_IRQ_INDEX,
    VFIO_REGION_INFO_FLAG_MMAP,
};
use vfio_ioctls::{VfioContainer, VfioDevice, VfioOps, VfioRegionInfoCap};

use super::bar::Bar;
use super::bdf::Bdf;
use super::iova::IovaAlloc;
use super::irq::{IrqKind, Irqs};
use super::pinned::{Domain, Pinned};
use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};

const PCI_COMMAND: u64 = 0x04;
const PCI_COMMAND_MEMORY: u16 = 1 << 1;
const PCI_COMMAND_MASTER: u16 = 1 << 2;

pub struct VfioPci {
    bdf: Bdf,
    // Field order is drop order: unmap the BARs and free the eventfds while
    // the device fd is still open, then close the device (which releases
    // the group), and only then let go of the domain.
    bars: [Option<Bar>; 6],
    irqs: Option<Irqs>,
    device: VfioDevice,
    domain: Arc<Domain>,
}

impl VfioPci {
    pub fn open(bdf: Bdf) -> Result<VfioPci> {
        bdf.check_bound()?;
        let container = Arc::new(VfioContainer::new(None)?);
        let ops: Arc<dyn VfioOps> = container.clone();
        let device = VfioDevice::new(&bdf.sysfs(), ops, false)?;
        device.reset();
        enable_bus_master(&device);

        let mut bars: [Option<Bar>; 6] = Default::default();
        for (i, slot) in bars.iter_mut().enumerate() {
            let i = i as u32;
            let size = device.get_region_size(i);
            if size == 0 {
                continue;
            }
            let sparse = device
                .get_region_caps(i)
                .iter()
                .any(|c| matches!(c, VfioRegionInfoCap::SparseMmap(_)));
            let mmap = device.get_region_flags(i) & VFIO_REGION_INFO_FLAG_MMAP != 0 && !sparse;
            *slot = Some(Bar::new(
                device.as_raw_fd(),
                device.get_region_offset(i),
                size,
                mmap,
            )?);
        }

        Ok(VfioPci {
            bdf,
            bars,
            irqs: None,
            device,
            domain: Arc::new(Domain {
                container,
                iova: Mutex::new(IovaAlloc::default()),
            }),
        })
    }

    pub fn bdf(&self) -> Bdf {
        self.bdf
    }

    pub fn bar(&self, index: usize) -> Result<&Bar> {
        self.bars
            .get(index)
            .and_then(Option::as_ref)
            .ok_or_else(|| Error::Device(format!("{} has no BAR{index}", self.bdf)))
    }

    /// A DMA buffer pinned and mapped for this device; its bus address is
    /// [`DmaBuf::iova`].
    pub fn alloc(&self, len: usize) -> Result<DmaBuf> {
        DmaBuf::new_pinned(len, |bytes| Pinned::new(&self.domain, bytes))
    }

    /// Route the device's interrupts to eventfds: every MSI-X vector if it
    /// has MSI-X, else every MSI vector. Returns the kind and vector count.
    pub fn enable_irqs(&mut self) -> Result<(IrqKind, usize)> {
        let count = |index| {
            self.device
                .get_irq_info(index)
                .map_or(0, |i| i.count as usize)
        };
        let (kind, index, n) = match (
            count(VFIO_PCI_MSIX_IRQ_INDEX),
            count(VFIO_PCI_MSI_IRQ_INDEX),
        ) {
            (n, _) if n > 0 => (IrqKind::Msix, VFIO_PCI_MSIX_IRQ_INDEX, n),
            (_, n) if n > 0 => (IrqKind::Msi, VFIO_PCI_MSI_IRQ_INDEX, n),
            _ => {
                return Err(Error::Device(format!(
                    "{} has neither MSI-X nor MSI",
                    self.bdf
                )));
            }
        };
        self.disable_irqs();
        let irqs = Irqs::new(kind, n)?;
        self.device.enable_irq(index, irqs.fds())?;
        self.irqs = Some(irqs);
        Ok((kind, n))
    }

    /// Wait for interrupt `vector`; returns how often it fired since the
    /// last wait. Needs [`enable_irqs`](Self::enable_irqs) first.
    pub fn wait_irq(&self, vector: u32, timeout: Duration) -> Result<u64> {
        self.irqs()?.wait(vector, timeout)
    }

    /// Drop interrupts already counted on `vector`, e.g. before a latency
    /// measurement.
    pub fn clear_irq(&self, vector: u32) -> Result<()> {
        self.irqs()?.clear(vector)
    }

    fn irqs(&self) -> Result<&Irqs> {
        self.irqs
            .as_ref()
            .ok_or_else(|| Error::InvalidArgument("interrupts not enabled".into()))
    }

    fn disable_irqs(&mut self) {
        if let Some(irqs) = self.irqs.take() {
            let index = match irqs.kind {
                IrqKind::Msi => VFIO_PCI_MSI_IRQ_INDEX,
                IrqKind::Msix => VFIO_PCI_MSIX_IRQ_INDEX,
            };
            let _ = self.device.disable_irq(index);
        }
    }
}

impl Drop for VfioPci {
    fn drop(&mut self) {
        self.disable_irqs();
    }
}

/// vfio-pci enables the device but leaves bus mastering to the owner (QEMU
/// does it when the guest asks); without it every DMA is dropped.
fn enable_bus_master(device: &VfioDevice) {
    let mut cmd = [0u8; 2];
    device.region_read(VFIO_PCI_CONFIG_REGION_INDEX, &mut cmd, PCI_COMMAND);
    let v = u16::from_le_bytes(cmd) | PCI_COMMAND_MEMORY | PCI_COMMAND_MASTER;
    device.region_write(VFIO_PCI_CONFIG_REGION_INDEX, &v.to_le_bytes(), PCI_COMMAND);
}
