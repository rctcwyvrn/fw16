# In Hardcaml, `a &: b <>:. 0` parses as `a &: (b <>:. 0)` — parenthesise comparisons

OCaml gives operators starting with `<`, `=`, `>` higher precedence than those starting with `&`, and those higher
than `|`. With 1-bit operands it still type-checks, so the bug is silent: write `(a &: b) <>:. 0`.
(`d &: ~:m |: s` does parse as `(d &: ~:m) |: s`.) Found by the Path 3c worker, 2026-10-29.
