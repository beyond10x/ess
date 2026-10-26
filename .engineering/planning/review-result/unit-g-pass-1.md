---
format: aep.planning-md/3
id: review-result:unit-g-pass-1
kind: review-result
status: active
title: Unit G adversary pass 1
relations:
- reviews: story:a-dotted-variant-name-synthesizes-as-an-identifier
- reviews: story:ess-generate-types-checks-drift
revision: 1
---
## Adversary pass 1, unit G (ess-0700 wave)

Worktree ess-0700-g, base 3f7602f05; adversary added crates/generate/ess-synth/tests/adversary_g_labels.rs (7 cases, 4 red) and 2 cases in ess-cli tests/model_types.rs (green).

Held: coded dotted variants reach every Rust and Go site (guards, stored-row guards, payload and sets literals, invariants, view filters) and pass cargo check and go vet; derived beside declared and code beside derived collide by name on Rust and Web; check_owned_tree remedies name their own command; an absent --out stays absent.

```findings
- {file: crates/generate/ess-synth/src/go/layout.rs, line: 739, category: correctness, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "a variant code is emitted verbatim as a Go type with no check; code begun gives undefined kernel.begun, code type a syntax error"}
- {file: crates/generate/ess-synth/src/rust/feasibility.rs, line: 42, category: diagnostics, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "an invalid authored code is answered with give the variant a code"}
- {file: crates/generate/ess-synth/src/rust/feasibility.rs, line: 146, category: diagnostics, severity: note, verdict: NEEDS-CHANGE, origin: pre-existing, message: "a union-variant collision offers a code, which a union label cannot carry"}
- {file: crates/generate/ess-synth/src/go/mod.rs, line: 476, category: robustness, severity: warning, verdict: CONFIRMED, origin: pre-existing, message: "the ALIASED model panics in assert_bijection instead of refusing; not reachable through dotted labels"}
- {file: crates/generate/ess-synth/src/rust/feasibility.rs, line: 42, category: diagnostics, severity: note, verdict: CONFIRMED, origin: introduced, message: "the hint leads with naming code, while the variant form is flat name and code"}
- {file: crates/generate/ess-synth/src/go/layout.rs, line: 739, category: design, severity: note, verdict: CONFIRMED, origin: introduced, message: "a code colliding with a package-level Go name gets a trailing underscore, per the Go suffix policy"}
```
