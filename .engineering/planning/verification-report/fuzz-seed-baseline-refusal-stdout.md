---
format: aep.planning-md/1
id: verification-report:fuzz-seed-baseline-refusal-stdout
kind: verification-report
status: draft
title: Read back the actual target-refusal stdout from the fuzz seed baseline
relations:
- verifies: story:fuzz-the-specification-surface
revision: 1
---
# Companion readback of retained target-refusal stdout

This corrects the excerpt channel in verification-report:fuzz-seed-baseline-go-panic.
That immutable baseline copied stderr for four nonzero Rust/Web target results, but their
actual diagnostics were printed on stdout. Its four excerpt blocks are therefore empty.
The full original stdout/stderr files and hashes were retained and are unchanged. No command
was rerun and no new producer result is claimed by this readback.

The CLI used its default text output; these are textual target-failure diagnostics, not emitted
JSON failure envelopes. The original description of structured target refusals refers to the
source's typed failure path, not to the format of these retained CLI bytes.

## system-types / synthesize-rust

Actual exit 1; retained stdout SHA256 2055fa676334c5b171a8ad7e92a294cc4822c4bea7c49db186fc01a4e05222d4.

```text
rust target cannot emit this workspace
demo.Code: the Rust workspace has no domain module allocated for this system-level type
```

## system-types / synthesize-web

Actual exit 1; retained stdout SHA256 909cf491cf10f0160c2eaab8bc3b47a2ee7d84f2289280704e4fd6d14ee483fa.

```text
web target cannot emit this workspace
demo.Code: the Rust workspace has no domain module allocated for this system-level type
```

## optional-binding / synthesize-rust

Actual exit 1; retained stdout SHA256 1bc395707a7241eb4e61dc9c623d75048667ea0772fe7a285d2abce360c05efc.

```text
rust target cannot emit this workspace
demo.core.Fired.value, demo.core.Handle.value, handle-fired: binding `handle-fired` emits a plain clone of `String` for `value` of type `Optional<String>`; this assignment needs an explicit target representation
```

## optional-binding / synthesize-web

Actual exit 1; retained stdout SHA256 2555b7679c96cc492b0f06d45bee3b38f814d364dae8946d15e569468d984a75.

```text
web target cannot emit this workspace
demo.core.Fired.value, demo.core.Handle.value, handle-fired: binding `handle-fired` emits a plain clone of `String` for `value` of type `Optional<String>`; this assignment needs an explicit target representation
```

Source grounding at unchanged 239996d: ess-synth/src/rust/feasibility.rs:180 constructs the
MissingTypeOwner cause before owner-dependent naming; its bindings check constructs the
BindingAssignment cause. TargetFailure Display in src/failure.rs prints target/source/detail.
These observations do not turn the separately retained Go panic into a refusal or change the
33-command baseline counts. The future fuzz scope still needs the concrete Go repair owner.
