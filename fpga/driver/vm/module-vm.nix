# A throwaway NixOS guest that runs driver/fw16-fpga.nix for real, with
# QEMU's `edu` device standing in for the card: the module is pointed at
# edu's subsystem ID, so its udev rules must bind edu to vfio-pci and hand
# its /dev/vfio node to the group — nothing else in this config does.
# On boot a oneshot runs /tmp/shared/module-check.sh (copied in by
# run-check.sh), which writes its log into the share and powers off.
{ pkgs, modulesPath, ... }:
{
  imports = [
    (modulesPath + "/virtualisation/qemu-vm.nix")
    ../fw16-fpga.nix
  ];

  system.stateVersion = "26.05";
  networking.hostName = "fw16-fpga-module";

  virtualisation = {
    graphics = false;
    diskImage = null;
    memorySize = 1024;
    cores = 2;
    # Same machine as host/vm/edu-vm.nix: q35, vIOMMU with interrupt
    # remapping (vfio needs it), edu.
    qemu.options = [
      # No VGA: QEMU's std VGA (1234:1111) shares edu's vendor and every
      # QEMU device shares its subsystem, so the rule would claim it too.
      "-vga none"
      "-machine q35,kernel-irqchip=split"
      "-device intel-iommu,intremap=on"
      "-device edu"
    ];
  };

  boot.kernelPackages = pkgs.linuxPackages_latest;
  # The vIOMMU is Intel's, which needs intel_iommu=on; the module adds iommu=pt.
  boot.kernelParams = [ "intel_iommu=on" ];

  # The module asserts it; edu_check's inhibitor step must pass as tester.
  security.polkit.enable = true;

  hardware.fw16-fpga = {
    enable = true;
    # edu: vendor 1234 (only edu once VGA is off), subsystem QEMU's default
    # for every PCI device (Red Hat/QEMU) — so vendor + subsystem is unique
    # here, as 10EE + 10EE:4650 is on the FW16.
    vendor = "0x1234";
    subsystemVendor = "0x1af4";
    subsystemDevice = "0x1100";
  };

  # The non-root user the module exists for.
  users.users.tester = {
    isNormalUser = true;
    extraGroups = [ "fw16-fpga" ];
  };

  services.getty.autologinUser = "root";

  systemd.services.module-check = {
    wantedBy = [ "multi-user.target" ];
    after = [ "multi-user.target" "systemd-logind.service" ];
    path = with pkgs; [ bash pciutils util-linux systemd coreutils gnugrep gawk gnused hostname ];
    serviceConfig = {
      Type = "oneshot";
      StandardOutput = "journal+console";
    };
    script = ''
      bash /tmp/shared/module-check.sh 2>&1 | tee /tmp/shared/module-check.log
      systemctl poweroff --no-block
    '';
  };
}
