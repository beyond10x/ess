# Bounded adversary probes

The reports preserve the initial depth-boundary finding and its corrected recheck. `probe.rs` is
the exact reviewer Rust test source; the named `reviewer_documented_depth128_literal_survives_original_byte_admission`
case was red before the correction and green afterward. The implementation's permanent
`direct_returns` regression tests now exercise both suite families and the boundary refusals.

The exploratory `reviewer_authored_depth128_literal_survives_compile_and_admission` case records
the inherited YAML source-transport limit. It remains a diagnostic outside the six declared
contract probes. `corrected-probes.log` records the exact filtered run, including that distinction.
No runtime or admitted-suite response assertion is weakened by the source transport limitation.

Logs replace the local home prefix with `$HOME`; report and probe contents are otherwise retained.
The review is an agent's bounded attack. Test-runner results supply the observed behavior; it does
not assert human approval or broader conformance than the cases it executed.
