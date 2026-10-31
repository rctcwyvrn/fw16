# An e-marked USB-C cable carries its own Ra, so custom USB-C links that rely on Ra/Rd detection need a non-e-marked cable

Each plug of an e-marked cable puts Ra on its VCONN pin: Ra at the far end of an e-marked cable reads as Ra/Ra (audio
accessory), and the e-marker alone looks like Ra to a discrete CC detector. The keyboard's inter-half link therefore
requires a USB 2.0 C-to-C cable without an e-marker (`keyboard/docs/pcb-spec.md`). K-elec worker, 2026-10-31.
