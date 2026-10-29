# A throwaway NixOS guest for testing omni's vfio layer (Path 4 step 4)
# without the card: q35 + an emulated Intel IOMMU (vIOMMU) + QEMU's `edu`
# DMA device, bound to vfio-pci at boot. On boot, a oneshot runs
# /tmp/shared/edu_check (copied in by run-edu-check.sh), writes the log
# into the share and powers off. Root is tmpfs; nothing persists.
{ pkgs, modulesPath, ... }:
{
  imports = [ (modulesPath + "/virtualisation/qemu-vm.nix") ];

  system.stateVersion = "26.05";
  networking.hostName = "omni-edu";

  virtualisation = {
    graphics = false;
    diskImage = null;
    memorySize = 1024;
    cores = 2;
    qemu.options = [
      # q35: intel-iommu needs PCIe. kernel-irqchip=split only matters
      # under KVM (interrupt remapping needs it); TCG ignores it.
      "-machine q35,kernel-irqchip=split"
      # intremap=on: vfio refuses groups behind an IOMMU without interrupt
      # remapping (unless allow_unsafe_interrupts).
      "-device intel-iommu,intremap=on"
      # edu: MSI (not MSI-X), 28-bit DMA mask by default — omni's IOVA
      # window starts at 1 MiB, so small buffers stay under it.
      "-device edu"
    ];
  };

  # 7.3, the kernel the FW16 runs (decision 0009), not the pin's 6.18
  # default: the vfio-pci behaviour under test is that kernel's.
  boot.kernelPackages = pkgs.linuxPackages_latest;
  boot.kernelParams = [ "intel_iommu=on" ];
  boot.kernelModules = [ "vfio-pci" ];
  boot.extraModprobeConfig = "options vfio-pci ids=1234:11e8";

  services.getty.autologinUser = "root";

  systemd.services.edu-check = {
    wantedBy = [ "multi-user.target" ];
    after = [ "multi-user.target" "systemd-logind.service" ];
    path = with pkgs; [ pciutils util-linux systemd coreutils gnugrep ];
    serviceConfig = {
      Type = "oneshot";
      # Also on the serial console, so a hang is visible in console.log.
      StandardOutput = "journal+console";
    };
    script = ''
      log=/tmp/shared/edu-check.log
      args=$(cat /tmp/shared/edu-check.args 2>/dev/null || true)
      {
        echo "== kernel: $(uname -r)"
        echo "== cmdline: $(cat /proc/cmdline)"
        echo "== edu:"; lspci -nnk -d 1234:11e8
        echo "== IOMMU groups:"
        for d in /sys/kernel/iommu_groups/*/devices/*; do echo "   $d"; done
        echo "== edu_check $args"
        rc=0
        # shellcheck disable=SC2086
        /tmp/shared/edu_check $args || rc=$?
        echo "== kernel log (iommu/dmar/vfio):"
        dmesg | grep -iE 'dmar|vfio|iommu' | tail -n 40
        echo "== exit $rc"
      } 2>&1 | tee "$log"
      systemctl poweroff --no-block
    '';
  };
}
