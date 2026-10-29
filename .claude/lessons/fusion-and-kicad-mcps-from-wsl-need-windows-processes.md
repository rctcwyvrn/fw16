# From WSL, Fusion and Konnect MCPs must run as Windows processes via interop — no network path reaches them

Fusion360.exe serves its MCP on Windows `127.0.0.1:27182`; WSL here is on NAT networking, so the
port is unreachable from WSL. KiCad 10 (`F:\Program Files\KiCad\10.0`) exposes no TCP port at all
(IPC is Windows-local), so the Linux `~/bin/konnect` can't drive it. Mirrored networking is ruled
out because Tailscale runs on both Windows and WSL.

What works (2026-10-29), registered local to fw16 in WSL `~/.claude.json`:
- `konnect` → `~/bin/konnect-win.exe` (a +x copy of momentum's `pcb/konnect.exe`; WSL interop needs
  the exec bit). Give it Windows paths (`C:\…` / `\\wsl.localhost\…`), never `/home/…`.
- `fusion360` → `~/bin/fusion-mcp-bridge`: Windows `node.exe` + `npx mcp-remote@0.15.2`.

Gotchas found on the way:
- mcp-remote needs Node 18+; Windows Node 14 made npx treat the URL as a package (E405).
- npx started with a `\\wsl.localhost` cwd exits silently — the bridge `cd`s to `/mnt/c` first.
- WSL can't exec `.cmd` files; call `node.exe npx-cli.js` instead of `npx.cmd`.
- Fusion's `initialize` reply is slow; a client that pipelines requests before it arrives gets
  `Missing MCP-Session-Id header`. Claude Code waits, so this only bites hand-rolled tests.

The Windows Claude Code registrations (`C:\Users\17786\.claude.json`, fw16 local) still work too.
