# Get AU15P pin/bank facts from UG575 + PG213 PDFs, not this machine's Vivado

The local Vivado 2025.2 has no Artix UltraScale+ device files, and in batch mode it aborts on a locale error
(`_S_create_c_locale name not valid`; only C.utf8 exists, and `LC_ALL=C` does not help). AMD's per-device
package `.txt` URLs return 404. What works: UG575 via `https://docs.amd.com/api/khub/documents/GnuVZDIoTcrlIuu8Aok5XQ/content`
(594 pages, `pdftotext -layout`, render figure pages with `pdftoppm`) and PG213 v1.3 Appendix C, Table 94.
