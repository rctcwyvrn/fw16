# Check that fold-flat legs fit under the board before recommending them: leg length scales with the case width

On 2026-11-01 I recommended "steel-pin kickstands" for the keyboard halves (decision 0003) without sizing them. A leg
square to the case, near the inner edge, must be ≈ d·tan θ long, where d is its distance from the outer edge the half
pivots on: ≈ 125 mm here, so 33 mm for 15° and 72 mm for 30°. Folded flat, it lies under the key columns. There the
only free height is the floor plus the hot-swap socket gap, and that gap is filled by sockets and diodes on a
staggered grid. Scanning the B-side courtyards found a single 5.8 mm lane, and a 1.2 mm leg buckles at a few
newtons. The user moved tenting to the tray halves.

- Before offering a leg mechanism, compute L = d·tan θ and scan the underside keep-outs (socket B.CrtYd, diodes)
  for a lane L long and wide enough. If there isn't one, legs need their own layer or must live off the case.
