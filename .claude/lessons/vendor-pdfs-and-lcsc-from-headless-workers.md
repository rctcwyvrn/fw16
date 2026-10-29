# Headless workers can't fetch AMD datasheet PDFs; LCSC's detail API works where search doesn't

- docs.amd.com serves datasheets as per-section HTML (`/r/en-US/ds931-…/<Section-Name>`, readable via WebFetch, no
  page numbers); the PDFs aren't readable by WebFetch and `curl` is denied headless. Page refs need an interactive
  download (TI PDFs: WebFetch saves the binary, then `nix-shell -p poppler-utils --run pdftotext`).
- `wmsc.lcsc.com/ftps/wm/product/detail?productCode=C…` returns stock/price/alternates; LCSC search endpoints
  return 403. Get C numbers from a web search (`lcsc <MPN>`), then read the detail API.
- MPS's AU10P/15P reference rail figures (0.75 A MGTAVCC, 3 A MGTAVTT) are for all 12 GTH — scale per quad used.
Found by the Path 2b worker, 2026-10-29.
