# Konnect `edit_schematic_component` field rotation behaves as relative to a rotated symbol

On a `Device:R` placed at rotation 90 (horizontal), setting `field_placements.Value.rotation = 0` rendered the
text **vertical**; `rotation = 90` rendered it horizontal (KiCad 10.0.7, Konnect 0.13.0, 2026-10-29). To make a
field on a rotated symbol read horizontally, give it the symbol's own rotation, then check a cropped render
(`render_schematic_png` at 3200 px + `magick -crop` from `nix-shell -p imagemagick`). `inline: true` on a full
sheet exceeds the tool-result limit; render to a Windows path and Read the PNG instead.
