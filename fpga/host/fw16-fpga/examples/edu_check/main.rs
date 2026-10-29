//! `edu_check [BDF] [--suspend]` — the vfio layer against QEMU's `edu`
//! device, run inside the VM from `host/vm/` (as root, edu bound to
//! vfio-pci, a vIOMMU present). Proves, through `fw16_fpga::vfio::VfioPci`:
//! BAR mmap, MSI → eventfd, IOMMU-mapped DMA both ways, that an unmapped
//! IOVA is not reachable, and the logind delay inhibitor.
//!
//! Prints one PASS/FAIL line per check and `EDU-CHECK PASS|FAIL` last;
//! exit 0 only if everything passed.

mod edu;
mod sleep;

use std::time::Duration;

use edu::{DEV_BUF, DEV_BUF_LEN, Edu, IRQ_DMA_DONE};
use fw16_fpga::vfio::{Bdf, IrqKind, VfioPci};
use fw16_fpga::{Error, Result};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let suspend = args.iter().any(|a| a == "--suspend");
    let bdf = match args.iter().find(|a| !a.starts_with("--")) {
        Some(s) => Bdf::parse(s).expect("BDF like 0000:00:03.0"),
        None => find_edu().expect("no edu device (1234:11e8) in /sys/bus/pci/devices"),
    };
    println!("edu at {bdf}");

    let mut failed = 0;
    let mut report = |name: &str, r: std::result::Result<String, String>| match r {
        Ok(note) => println!("PASS  {name:<12} {note}"),
        Err(why) => {
            failed += 1;
            println!("FAIL  {name:<12} {why}");
        }
    };

    match VfioPci::open(bdf) {
        Err(e) => report("open", Err(e.to_string())),
        Ok(mut pci) => {
            report("open", bars(&pci));
            report("mmio", s(mmio(&pci)));
            report("msi", s(msi(&mut pci)));
            report("dma", s(dma(&pci)));
            report("iommu", s(iommu(&pci)));
        }
    }
    report("inhibitor", sleep::check(suspend));

    println!("EDU-CHECK {}", if failed == 0 { "PASS" } else { "FAIL" });
    std::process::exit(if failed == 0 { 0 } else { 1 });
}

fn s(r: Result<String>) -> std::result::Result<String, String> {
    r.map_err(|e| e.to_string())
}

fn bars(pci: &VfioPci) -> std::result::Result<String, String> {
    let bar = pci.bar(0).map_err(|e| e.to_string())?;
    if !bar.is_mapped() {
        return Err("BAR0 not mmapped (fell back to pread/pwrite)".into());
    }
    Ok(format!("BAR0 {:#x} bytes, mmapped", bar.size()))
}

fn mmio(pci: &VfioPci) -> Result<String> {
    let edu = Edu::new(pci)?;
    let id = edu.id()?;
    if id & 0xFFFF != 0x00ed {
        return Err(Error::Device(format!("ID {id:#010x} is not edu's")));
    }
    for v in [0, 0xFFFF_FFFF, 0x1234_5678, 0xA5A5_5A5A] {
        let got = edu.liveness(v)?;
        if got != !v {
            return Err(Error::Device(format!(
                "liveness {v:#x} → {got:#x}, want {:#x}",
                !v
            )));
        }
    }
    Ok(format!("ID {id:#010x}, liveness inverts"))
}

fn msi(pci: &mut VfioPci) -> Result<String> {
    let (kind, n) = pci.enable_irqs()?;
    if kind != IrqKind::Msi {
        return Err(Error::Device(format!("edu should offer MSI, got {kind:?}")));
    }
    let edu = Edu::new(pci)?;
    let mut lat = Vec::new();
    for _ in 0..100 {
        pci.clear_irq(0)?;
        let t0 = std::time::Instant::now();
        edu.raise(1)?;
        pci.wait_irq(0, Duration::from_secs(5))?;
        lat.push(t0.elapsed());
        if edu.status()? & 1 == 0 {
            return Err(Error::Device("interrupt without status bit".into()));
        }
        edu.ack(1)?;
    }
    if pci.wait_irq(0, Duration::from_millis(100)).is_ok() {
        return Err(Error::Device("spurious interrupt after ack".into()));
    }
    lat.sort();
    Ok(format!(
        "{kind:?} x{n}, 100 raised/received, latency min {:?} median {:?} max {:?}",
        lat[0], lat[50], lat[99]
    ))
}

fn dma(pci: &VfioPci) -> Result<String> {
    let edu = Edu::new(pci)?;
    let mut out = pci.alloc(DEV_BUF_LEN)?;
    let mut back = pci.alloc(DEV_BUF_LEN)?;
    for (i, w) in out.words_mut().iter_mut().enumerate() {
        *w = 0x0DD0_0000_0000_0000 | (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 8;
    }
    back.words_mut().fill(0xFFFF_FFFF_FFFF_FFFF);
    let (a, b) = (iova(&out)?, iova(&back)?);
    let t_out = edu.dma(a, DEV_BUF, DEV_BUF_LEN, false)?;
    let t_back = edu.dma(DEV_BUF, b, DEV_BUF_LEN, true)?;
    let bad = out
        .words()
        .iter()
        .zip(back.words())
        .filter(|(x, y)| x != y)
        .count();
    if bad != 0 {
        return Err(Error::Device(format!(
            "{bad} of {} words differ after the round trip",
            out.words().len()
        )));
    }
    Ok(format!(
        "4 KiB RAM→edu→RAM via IOVAs {a:#x}/{b:#x}, identical; {t_out:?} / {t_back:?}"
    ))
}

/// Free a buffer, then aim edu's DMA at its old IOVA: the IOMMU must
/// refuse it (the guest kernel logs a DMAR fault), no live buffer may
/// change, and the device must still work afterwards.
fn iommu(pci: &VfioPci) -> Result<String> {
    let edu = Edu::new(pci)?;
    // `live` first: allocated after the free, first-fit would hand it the
    // stale IOVA and the stray DMA would land in it legitimately.
    let mut live = pci.alloc(DEV_BUF_LEN)?;
    live.words_mut().fill(0x5A5A_5A5A_5A5A_5A5A);
    let stale = iova(&pci.alloc(DEV_BUF_LEN)?)?;
    let stray = edu.dma(DEV_BUF, stale, DEV_BUF_LEN, true);
    if live.words().iter().any(|&w| w != 0x5A5A_5A5A_5A5A_5A5A) {
        return Err(Error::Device(
            "a DMA to a freed IOVA changed a live buffer".into(),
        ));
    }
    // The stray transfer may still raise its done interrupt; drain it.
    let _ = edu.ack(IRQ_DMA_DONE);
    let _ = pci.clear_irq(0);
    let check = dma(pci)?;
    Ok(format!(
        "DMA to freed IOVA {stale:#x} ({}); live buffer untouched; device still works: {check}",
        match stray {
            Ok(_) => "edu reported done".to_string(),
            Err(e) => format!("edu: {e}"),
        }
    ))
}

fn iova(buf: &fw16_fpga::DmaBuf) -> Result<u64> {
    buf.iova()
        .ok_or_else(|| Error::Device("vfio buffer without an IOVA".into()))
}

fn find_edu() -> Option<Bdf> {
    let hex = |p: std::path::PathBuf| {
        let s = std::fs::read_to_string(p).ok()?;
        u16::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok()
    };
    std::fs::read_dir("/sys/bus/pci/devices")
        .ok()?
        .flatten()
        .find_map(|e| {
            let p = e.path();
            (hex(p.join("vendor")) == Some(edu::VENDOR)
                && hex(p.join("device")) == Some(edu::DEVICE))
            .then(|| Bdf::parse(&e.file_name().to_string_lossy()))
            .flatten()
        })
}
