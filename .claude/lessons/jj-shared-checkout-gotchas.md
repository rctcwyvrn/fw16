# In the shared jj checkout: `jj commit <paths>` is cwd-relative, and cargo's `target/` must be ignored first

- `jj commit <paths>` resolves paths against the cwd. Run from `bay/tools/` with repo-relative paths it silently
  made an empty commit; recover with `jj squash --from @ --into @- <files>`. Run jj from the repo root.
- jj snapshots every untracked file on any jj command; with the default `snapshot.max-new-file-size` (1 MiB) one
  large rlib makes jj refuse to snapshot, blocking every session sharing the checkout. Put `target/` in a
  `.gitignore` before the first `cargo` run (Path 4 did: `host/.gitignore`).
Found by the Path 1 and Path 4 workers, 2026-10-29.
- When another path lands an ABI change (regmap v0.2, 2026-10-29), `jj git fetch` then re-run `cargo test` before
  building on top: hand-written stand-ins for not-yet-landed constants collide with the generated ones once both
  exist. Re-export rather than redefine, so a collision is harmless.
- **Never `jj rebase -r @-` and then `jj bookmark set main -r @-`.** `rebase -r` re-parents the working copy onto the
  commit's old parent, so `@-` no longer names the new commit; the bookmark set is refused ("backwards or sideways"),
  the push says "Nothing changed", and the commit silently stays local. Capture the change id from `jj commit`'s output
  (or `jj log -r 'mine() & ~::main@origin'`) and use it: `jj rebase -r <id> -d main@origin && jj bookmark set main -r
  <id> && jj git push -b main`. Two commits appending to the same log file from the same base then conflict — resolve
  with `jj new <id>`, fix the file, `jj squash`. Found 2026-10-29 (pcie_sideband/config commits sat unpushed).
- `~` is jj's fileset negation operator, so a KiCad lock file such as `~foo.kicad_pcb.lck` must be quoted as a string
  literal: `jj file untrack 'glob:"dir/~*.lck"'` (fw16, 2026-10-31).
