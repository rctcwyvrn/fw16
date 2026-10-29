# A Fusion part-intent design rejects STEP assembly import; import into an assembly-intent document

`importManager.importToTarget(createSTEPImportOptions(p), root)` on the `omnimeter` design (designIntent 0, part)
failed with `RuntimeError: 2 : InternalValidationError : results`. A new document set to
`DesignIntentTypes.AssemblyDesignIntentType` imported all three Framework STEPs fine, in their native frame (mm).
Fusion MCP scripts must define `def run(context):` — top-level code alone errors with `NameError: run`.
