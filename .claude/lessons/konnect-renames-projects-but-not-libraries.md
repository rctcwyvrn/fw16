# Konnect's `rename_project` renames a KiCad project safely, but Konnect has no tool that renames a library

`rename_project` (dry_run first) renames `.kicad_pro/.kicad_sch/.kicad_pcb` and rewrites every
`(project "<name>")` symbol-instance block in all sub-sheets. Without those rewrites KiCad drops every
reference designator. A library nickname (`<lib>:<item>` lib_ids, the lib-table entries, and the
`.kicad_sym`/`.pretty` filenames) has no Konnect tool. On 2026-10-29 the user approved a one-off `sed` for
that case: the lib_id strings carry no UUIDs. Ask before doing it again, because the repo's default rule is
Konnect-only.
