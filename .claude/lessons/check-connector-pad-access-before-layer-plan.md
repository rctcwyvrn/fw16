# Map which sides of a dense connector's pads are open before picking a layer plan for the lanes

The card's FXBeam J1 (`FXBeam_2x74_DB`) has its lane pads in columns at 0.6 mm pitch (0.4 mm pads, 0.2 mm gaps)
with a second column 0.87 mm beside each one (0.17 mm gap), so a lane pad can only be reached from one open side, never
between pads. Within a column the pads run RX1, RX0, TX0, TX1 from the U14 side, while U14's balls run lane 0 left of
lane 1, so two TX pairs in the same column cannot both reach U14 on L1 without crossing. Claude proposed "L1 pairs
from the left, RX pairs from the right" (2026-11-05) without dumping the neighbouring columns, and the user approved
a plan that cannot be routed. Next time: dump every pad within ~3 mm of the target pads (net, size, position) and
check the escape topology (order along the open side against order at the other end) before offering options.
