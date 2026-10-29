//! A logind *delay* inhibitor for suspend, so a vfio handle hears about
//! sleep before it happens (decision 0009, "Suspend").
//!
//! vfio-pci in 7.3 has no system-sleep callbacks: the PCI core restores
//! config space on resume, but nothing tells userspace, and DMA left running
//! into a suspend is undefined. So the handle holds `Inhibit("sleep", …,
//! "delay")`; on `PrepareForSleep(true)` it runs the owner's hook (stop
//! DMA), then closes the inhibitor fd so the suspend proceeds; on
//! `PrepareForSleep(false)` it runs the hook again (re-check `ID`, re-init
//! XDMA — Path 4 step 5) and takes a fresh inhibitor. logind waits at most
//! `InhibitDelayMaxSec` (5 s by default) for the hook.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedFd;

use crate::error::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SleepEvent {
    /// The system is about to sleep: stop DMA now.
    Suspending,
    /// The system woke (or the sleep was cancelled): re-check the device.
    Resumed,
}

pub type SleepHook = Box<dyn Fn(SleepEvent) + Send + Sync>;

pub struct SleepInhibitor {
    shared: Arc<Shared>,
    conn: Connection,
}

struct Shared {
    /// The inhibitor; `None` between `Suspending` and `Resumed`.
    fd: Mutex<Option<OwnedFd>>,
    suspends: AtomicU64,
    resumes: AtomicU64,
}

impl SleepInhibitor {
    /// Take the inhibitor and start listening. `why` shows up in
    /// `systemd-inhibit --list`. Fails without a system bus or logind.
    pub fn start(why: &str, hook: SleepHook) -> Result<SleepInhibitor> {
        let conn = Connection::system()?;
        let proxy = Proxy::new(
            &conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )?;
        // Subscribe before inhibiting, so no PrepareForSleep can slip
        // between the two.
        let signals = proxy.receive_signal("PrepareForSleep")?;
        let shared = Arc::new(Shared {
            fd: Mutex::new(Some(inhibit(&proxy, why)?)),
            suspends: AtomicU64::new(0),
            resumes: AtomicU64::new(0),
        });

        let why = why.to_string();
        let s = shared.clone();
        std::thread::Builder::new()
            .name("fw16-fpga-sleep".into())
            .spawn(move || {
                // Ends when the connection closes (`Drop`) or errors.
                for msg in signals {
                    let Ok(going_down) = msg.body().deserialize::<bool>() else {
                        continue;
                    };
                    if going_down {
                        hook(SleepEvent::Suspending);
                        s.suspends.fetch_add(1, Ordering::SeqCst);
                        *s.fd() = None;
                    } else {
                        hook(SleepEvent::Resumed);
                        s.resumes.fetch_add(1, Ordering::SeqCst);
                        // If logind went away there is nothing to hold.
                        *s.fd() = inhibit(&proxy, &why).ok();
                    }
                }
            })
            .map_err(|e| crate::Error::Os {
                what: "spawn the sleep listener".into(),
                err: e,
            })?;
        Ok(SleepInhibitor { shared, conn })
    }

    /// Whether the inhibitor is currently held.
    pub fn held(&self) -> bool {
        self.shared.fd().is_some()
    }

    /// `(suspends, resumes)` seen since `start`.
    pub fn events(&self) -> (u64, u64) {
        (
            self.shared.suspends.load(Ordering::SeqCst),
            self.shared.resumes.load(Ordering::SeqCst),
        )
    }
}

impl Shared {
    fn fd(&self) -> std::sync::MutexGuard<'_, Option<OwnedFd>> {
        self.fd.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        *self.shared.fd() = None;
        // Closing the connection ends the listener's signal stream; the
        // thread is not joined, since a hook may be mid-run.
        let _ = self.conn.clone().close();
    }
}

fn inhibit(proxy: &Proxy<'_>, why: &str) -> Result<OwnedFd> {
    Ok(proxy.call("Inhibit", &("sleep", "fw16-fpga", why, "delay"))?)
}
