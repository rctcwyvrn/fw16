# KiCad cannot save files that live on \wsl.localhost

KiCad 10 on Windows saves atomically: it writes `<file>.kicad-save-NNNN-NN`,
copies the original's Windows permissions onto it, then swaps. Over the
`\wsl.localhost\NixOS\…` 9P share the permission copy fails ("Cannot copy
permissions from … to ….kicad-save-…"), and the save is lost.

- **Konnect's `save_project` (IPC) fails the same way but reports "Board saved
  successfully"** (2026-10-08). Always check the file's size and mtime after a
  save; never trust the message.
- Konnect's *direct* file writes (schematic tools) work fine on the share.
- **Fix adopted:** KiCad opens a Windows working copy
  (`C:\Users\17786\momentum-pcb\`), and `pcb/sync.sh` copies it into the repo.
  Git stays the source of truth.

fw16 (2026-10-31): the card hit the same error. KiCad and Konnect work on `C:\Users\17786\fw16-card\` while the card is
open in KiCad; `fpga/bay/card/sync.sh seed|pull|check` carries the files (it includes `lib/`). Konnect's schematic
tools then target the Windows copy too, so the two never diverge.
