(* Register map v0 — the S3 ABI shared with host/. One 4 KiB block per
   subsystem on BAR0 (XDMA's AXI-Lite master), 32-bit registers, 4-byte
   aligned. Adding a register inside a block never moves another block;
   changing an existing offset after S3 needs a 3-way review.

   v0.2 (docs/regmap-v0.2-proposal.md): the IRQ block, and every RO
   [<P>_LO]/[<P>_HI] pair is tear-free — a read of [_LO] latches the high word,
   which [_HI] returns (Regs implements this for every such pair by name). *)

open Reg

let version_major = 0
let version_minor = 2

let ctl =
  { bname = "CTL"
  ; base = 0x0000
  ; bdoc = "Identity, scratch, status and board telemetry."
  ; regs =
      [ reg "ID" 0x0000 RO ~reset:0x4F4D4E49 "Constant 0x4F4D4E49 ('OMNI')."
      ; reg
          "VERSION"
          0x0004
          RO
          ~reset:((version_major lsl 16) lor version_minor)
          "Register-map version."
          ~fields:[ field "MINOR" 0 ~width:16 ""; field "MAJOR" 16 ~width:16 "" ]
      ; reg "SCRATCH" 0x0008 RW "Free read/write word for host access tests."
      ; reg
          "STATUS"
          0x000C
          RO
          "Live status bits."
          ~fields:
            [ field "LINK_UP" 0 "PCIe user link up (from XDMA)."
            ; field "MEM_READY" 1 "Ring-buffer memory calibrated / ready."
            ; field "SNK_ERR" 2 "Sink has counted at least one error since clear."
            ; field "RING_OVERFLOW" 3 "Ring buffer dropped data since clear."
            ]
      ; reg
          "TEMP"
          0x0010
          RO
          "SYSMON die temperature, raw 10-bit code in [15:6] as SYSMON reports it."
      ; reg "PWR_MW" 0x0014 RO "Card input power in mW from the INA telemetry; 0 if absent."
      ]
  }
;;

let src =
  { bname = "SRC"
  ; base = 0x1000
  ; bdoc = "AXI-Stream test source (to C2H)."
  ; regs =
      [ reg
          "SRC_CTL"
          0x1000
          RW
          "Source control."
          ~fields:
            [ field "EN" 0 "Generate beats while set."
            ; field "MODE" 1 "0 = counter, 1 = PRBS-31."
            ]
      ; reg
          "SRC_RATE"
          0x1004
          RW
          ~reset:0x10000
          "Beats offered per 2^16 clock cycles; >= 0x10000 means every cycle."
      ; reg "SRC_BEATS_LO" 0x1008 RO "Beats accepted downstream, low word; reading it latches _HI."
      ; reg "SRC_BEATS_HI" 0x100C RO "High word as of the last SRC_BEATS_LO read."
      ; reg "SRC_CLEAR" 0x1010 WO_PULSE "Bit 0: reset the beat counter and pattern state."
      ]
  }
;;

let snk =
  { bname = "SNK"
  ; base = 0x2000
  ; bdoc = "AXI-Stream checker (from H2C)."
  ; regs =
      [ reg
          "SNK_CTL"
          0x2000
          RW
          "Sink control."
          ~fields:
            [ field "EN" 0 "Check incoming beats while set (always accepts)."
            ; field "MODE" 1 "Expected pattern: 0 = counter, 1 = PRBS-31."
            ]
      ; reg "SNK_ERRS" 0x2004 RO "Beats that did not match the expected pattern."
      ; reg "SNK_BEATS_LO" 0x2008 RO "Beats received, low word; reading it latches _HI."
      ; reg "SNK_BEATS_HI" 0x200C RO "High word as of the last SNK_BEATS_LO read."
      ; reg "SNK_CLEAR" 0x2010 WO_PULSE "Bit 0: reset counters and pattern state."
      ]
  }
;;

let ring =
  { bname = "RING"
  ; base = 0x3000
  ; bdoc = "Ring buffer between the source and C2H, backed by BRAM or DDR4."
  ; regs =
      [ reg
          "RING_CTL"
          0x3000
          RW
          "Ring control."
          ~fields:
            [ field "LOOPBACK" 0 "Fill from H2C instead of the source (sink still checks)."
            ]
      ; reg "RING_DEPTH" 0x3004 RO "Capacity in 256-bit beats."
      ; reg "RING_FILL" 0x3008 RO "Beats currently stored."
      ; reg "RING_CLEAR" 0x300C WO_PULSE "Bit 0: empty the ring and clear OVERFLOW."
      ]
  }
;;

let ts =
  { bname = "TS"
  ; base = 0x4000
  ; bdoc = "64-bit timestamp counter and PPS capture (Path 3 step 4)."
  ; regs =
      [ reg "TS_LATCH" 0x4000 WO_PULSE "Bit 0: copy the live counter into TS_NOW_*."
      ; reg "TS_NOW_LO" 0x4004 RO "Counter at the last TS_LATCH, low word; reading it latches _HI."
      ; reg "TS_NOW_HI" 0x4008 RO "High word as of the last TS_NOW_LO read."
      ; reg "TS_PPS_LO" 0x400C RO "Counter at the last PPS rising edge, low word; reading it latches _HI."
      ; reg "TS_PPS_HI" 0x4010 RO "High word as of the last TS_PPS_LO read."
      ; reg "TS_PPS_COUNT" 0x4014 RO "PPS edges seen since reset."
      ]
  }
;;

(* One interrupt line (XDMA usr_irq_req[0]) shared by all sources. Bit 3 is
   left free for a sink-done source if one is ever wanted. *)
let irq_sources =
  [ field "FORCE" 0 "IRQ_FORCE write (latency test)."
  ; field "PPS" 1 "PPS rising edge (reads 0 until the TS unit lands)."
  ; field "SNK_ERR" 2 "STATUS.SNK_ERR went 0 -> 1 (once per error run)."
  ]
;;

let irq =
  { bname = "IRQ"
  ; base = 0x5000
  ; bdoc =
      "Interrupts: usr_irq_req[0] is high while IRQ_PENDING & IRQ_EN is non-zero. \
       usr_irq_req is a held level and MSI-X fires only on its rising edge, so after \
       clearing IRQ_PENDING the host must re-read it until it reads 0 under IRQ_EN \
       before waiting for the next interrupt."
  ; regs =
      [ reg "IRQ_EN" 0x5000 RW "Per-source enable." ~fields:irq_sources
      ; reg
          "IRQ_PENDING"
          0x5004
          RW1C
          "Latched sources (set whether or not enabled); write 1 to clear. A set \
           wins over a clear in the same cycle."
          ~fields:irq_sources
      ; reg "IRQ_COUNT" 0x5008 RO "Rising edges of usr_irq_req since reset (wraps)."
      ; reg
          "IRQ_FORCE"
          0x500C
          WO_PULSE
          "Bit 0: set IRQ_PENDING.FORCE now."
          ~fields:[ field "FORCE" 0 "" ]
      ]
  }
;;

let blocks = [ ctl; src; snk; ring; ts; irq ]
