# NixOS module for the FW16 FPGA bay card (decision 0009: no kernel module).
# Turns the IOMMU on, hands the card to vfio-pci, and lets members of one
# group drive it from userspace through the fw16-fpga crate (host/):
#
#   imports = [ "${fw16}/fpga/driver/fw16-fpga.nix" ];
#   hardware.fw16-fpga.enable = true;
#   users.users.<you>.extraGroups = [ "fw16-fpga" ];
#
# The card is matched on vendor 0x10EE plus its PCI *subsystem* ID (regmap
# v0.2 §3), not its device ID: XDMA's device ID (10EE:9xxx) is the IP's link-dependent default and
# is shared by every XDMA design, so it names neither this card nor a stable
# value. vfio-pci.ids= takes a subsystem ID only after a VID:DID pair, so
# binding is a udev driver_override instead (see driver/README.md).
{ config, lib, pkgs, ... }:
let
  cfg = config.hardware.fw16-fpga;
  v = cfg.vendor;
  sv = cfg.subsystemVendor;
  sd = cfg.subsystemDevice;

  # Exit 0 iff IOMMU group $1 holds a device with the card's subsystem ID.
  # Used by the /dev/vfio/<group> rule, whose device has no PCI parent to
  # match on.
  groupHasCard = pkgs.writeShellScript "fw16-fpga-group-has-card" ''
    for d in /sys/kernel/iommu_groups/"$1"/devices/*; do
      [ "$(cat "$d/vendor" 2>/dev/null)" = "${v}" ] \
        && [ "$(cat "$d/subsystem_vendor" 2>/dev/null)" = "${sv}" ] \
        && [ "$(cat "$d/subsystem_device" 2>/dev/null)" = "${sd}" ] && exit 0
    done
    exit 1
  '';
in
{
  options.hardware.fw16-fpga = {
    enable = lib.mkEnableOption "the FW16 FPGA bay card, bound to vfio-pci for the fw16-fpga userspace driver";

    vendor = lib.mkOption {
      type = lib.types.str;
      default = "0x10ee";
      description = ''
        The card's PCI vendor, as sysfs prints it. Matched with the subsystem
        ID; the device ID is deliberately not matched.
      '';
    };
    subsystemVendor = lib.mkOption {
      type = lib.types.str;
      default = "0x10ee";
      description = "The card's PCI subsystem vendor, as sysfs prints it (lower-case hex, 0x prefix).";
    };
    subsystemDevice = lib.mkOption {
      type = lib.types.str;
      default = "0x4650";
      description = "The card's PCI subsystem device ('FP'), as sysfs prints it.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "fw16-fpga";
      description = "Group given the card's /dev/vfio/<iommu group> node (rw) and the memlock limit.";
    };

    memlock = lib.mkOption {
      type = lib.types.str;
      default = "unlimited";
      example = "1048576";
      description = ''
        RLIMIT_MEMLOCK (KiB, or "unlimited") for the group's login sessions:
        VFIO_IOMMU_MAP_DMA pins every DMA buffer and counts it against this.
        systemd services do not go through PAM; give them LimitMEMLOCK.
      '';
    };

    kernelParams = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ "amd_iommu=on" "iommu=pt" ];
      description = ''
        IOMMU kernel parameters. iommu=pt keeps host-owned devices on
        identity mappings (no translation cost); vfio still gets a
        translated domain for the card.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    boot.kernelParams = cfg.kernelParams;
    # vfio-pci has no device table, so nothing autoloads it.
    boot.kernelModules = [ "vfio-pci" ];

    users.groups.${cfg.group} = { };

    services.udev.extraRules = ''
      # fw16-fpga: the card goes to vfio-pci. No in-tree PCI driver claims
      # XDMA's 10EE:9xxx, so there is nothing to unbind; drivers_probe binds a
      # card that appears after vfio-pci loaded (rescan after reprogramming).
      ACTION=="add", SUBSYSTEM=="pci", ATTR{vendor}=="${v}", ATTR{subsystem_vendor}=="${sv}", ATTR{subsystem_device}=="${sd}", ATTR{driver_override}="vfio-pci", RUN+="${pkgs.runtimeShell} -c 'echo $kernel > /sys/bus/pci/drivers_probe'"
      # fw16-fpga: its IOMMU group's node to the group. Other vfio groups
      # (e.g. a GPU passed to a VM) keep the default root-only node.
      SUBSYSTEM=="vfio", KERNEL=="[0-9]*", PROGRAM="${groupHasCard} %k", GROUP="${cfg.group}", MODE="0660"
    '';

    security.pam.loginLimits = [
      { domain = "@${cfg.group}"; type = "-"; item = "memlock"; value = cfg.memlock; }
    ];
  };
}
