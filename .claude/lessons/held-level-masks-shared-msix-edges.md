# A level source held high on a shared edge-triggered MSI-X vector hides every other source's edge

With `usr_irq_req = |(IRQ_PENDING & IRQ_EN)` on one XDMA vector, an enabled source that holds its level (the project
slot's `slot_irq`) keeps the request high, so a second source going pending makes no edge and no MSI-X message.
Masking the held source out of the service loop is not enough: the waiter must poll `IRQ_PENDING` while that source is
enabled and pending (the crate polls every 1 ms, `irq::HELD_POLL`), or the other sources time out silently. A separate
`usr_irq` bit per source (its own vector) avoids it. Found by the F4-sw worker, 2026-11-01.
