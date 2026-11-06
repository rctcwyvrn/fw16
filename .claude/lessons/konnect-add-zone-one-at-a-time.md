# Add zones through Konnect one call at a time

Each `add_zone` over IPC refills the zone in KiCad. Sent in parallel, the calls after the first come back AS_BUSY with
the board unchanged (2026-11-06: 5 of 8 refused). Vias and traces can be sent in parallel batches, but zones must go one
at a time. `run_drc` with `sync_live_board` + `refill_zones` can also hit AS_BUSY right after. For the true count, sync
without the refill, then run the CLI refill DRC (`refill_zones` alone).
