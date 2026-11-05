# KiCad 10 custom rules need explicit units and an object prefix on every function

KiCad 10's Custom Rules checker rejects bare numbers (`(min 0.09)` → "Missing units for '0.09'"); write `0.09mm`.
Every expression function needs its object: `A.isCoupledDiffPair()`, not `isCoupledDiffPair()` ("Unknown parent of
function parameters"). Found 2026-11-05 writing the card's PCIe rules from memory of older examples.
