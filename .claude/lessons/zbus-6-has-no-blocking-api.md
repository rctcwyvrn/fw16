# zbus 6.0 removed `zbus::blocking`; pin zbus 5 for a synchronous D-Bus client

For the logind sleep inhibitor, `zbus = { version = "5", default-features = false, features = ["blocking-api",
"async-io"] }`, or write against the async API. Found by the Path 4b worker, 2026-10-29.
