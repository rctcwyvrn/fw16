# nixpkgs `kernel.config` only knows nixpkgs' own overrides — read `kernel.configfile` for real values

`linuxPackages_latest.kernel.config.isYes "VFIO"` returned false for VFIO, VFIO_PCI, IOMMUFD and AMD_IOMMU
even though the built kernel has them (`=m`/`=y`). The `.config` attribute only covers options nixpkgs sets in
its structured config; everything left to Kconfig defaults/autoModules reads as unset. To check a kernel
option, build the generated file and grep it:

```sh
nix build --no-link --print-out-paths "$NIXPKGS#linuxPackages_latest.kernel.configfile"   # → linux-config-<ver>
grep CONFIG_VFIO <that path>
```

It is a small derivation and was in the binary cache on 2026-10-28 (pin `58deea65`, kernel 7.3).
