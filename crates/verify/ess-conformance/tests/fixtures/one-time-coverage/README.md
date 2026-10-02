# Shared actual coverage/35 execution

`model.yaml` and `scenario.yaml` go through the real coverage builder with GeneratedAndAuthored
origins. The one authored rotation ID is selected using AdmittedInput::select, retaining the complete
unfiltered parent/inventory in `input.json`. Runtime adapters execute that original input carrier.
`manifest.json` pins actual five-category counts, required diagnostic code and real callback traces
for healthy rotation, repeated disclosure, target error and unsupported target controls. Every
observed plaintext is checked absent from native scenario/count evidence. This proves coverage/35
execution; it does not replace the broader ordinary/34 feature controls.

Creating this fixture exposed an actual producer gap: coverage_build calls authored::compile_one
directly, bypassing policy installation formerly only in authored::compile. The same source-produced
rotation test failed before policy attachment moved to the shared compile_one boundary, then passed.
