# The Windows Vivado 2025.2 has Artix UltraScale+ — ask Vivado, don't grep its install tree

Corrected 2026-10-29. `F:\AMDDesignTools\2025.2\Vivado\data\parts\xilinx\` has no `artixuplus/` directory, yet
Vivado lists 18 `xcau15p` parts (`artixuplus` family, `xcau15p-ffvb676-2-e` included) and the IP catalog has
`xdma:4.2` and `pcie4c_uscale_plus:1.0` inside an AU15P project. An earlier lesson concluded "no AU+ here" from the
missing directory — wrong. Check with `vivado -mode batch` + `get_parts`, and `get_ipdefs` only after
`create_project -in_memory -part …` (outside a project the IP catalog returns nothing).

Driving it from WSL works: `cd /mnt/c && cmd.exe /c "F:\AMDDesignTools\2025.2\Vivado\bin\vivado.bat -mode batch
-source $(wslpath -w file.tcl)"`. Running a Linux Vivado inside WSL fails on a locale error
(`_S_create_c_locale name not valid`). For AU15P pin/bank facts UG575 + PG213 remain the reference.
