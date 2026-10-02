---
format: aep.planning-md/3
id: review-result:go-generated-behaviour-adversary-pass2
kind: review-result
status: active
title: Adversary pass 2, story:go-generated-behaviour (wave ui-live-apps-w5)
relations:
- reviews: story:go-generated-behaviour
revision: 1
---
needs-change

Adversary pass 2 on story:go-generated-behaviour at 910ebc38d. Cases executed 48 to 54 (Go-filtered plus the new file), red 2 new (plus the known badge case); introduced 2, pre-existing 1. Test file: crates/generate/ess-synth/tests/adversary_go_behaviour_pass2.rs.

- warning: a word after a variadic `...` counts as package-qualified, so a domain named `truth` does not build.
- warning: behaviour.go imports sort, math/big and strings unaliased; a domain with one of those names is declared twice.
- note, pre-existing: a domain named fmt collides with the server package's fmt import.
- note: reversing TasksByOwner's secondary order key survives the generated suite (no rows tie on owner).
- note: All returning True instead of Unknown survives invariant_check (no fixture negates an all: over an absent value).

Held: generated vs owed matches the plan for billing, gatepass and a combined model (existence, held-state guard, aggregates, invariants, an owed when_related command); the collision row with owed and generated twins builds; BrokenInvariant_ only with an invariant, wire name broken_invariant; served gatepass Rust and Go agree on every command and view value after 8 steps apart from the badge; mutation probes caught 28 of 31 mutants (one survivor is equivalent).

```findings
[{"file":"crates/generate/ess-synth/src/go/behaviour.rs","line":363,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A word after the variadic `...` counts as package-qualified, so a domain named `truth` does not build."},
{"file":"crates/generate/ess-synth/src/go/behaviour.rs","line":193,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"behaviour.go imports sort, math/big and strings unaliased beside domain packages."},
{"file":"crates/generate/ess-synth/src/go/name.rs","line":23,"category":"boundary","severity":"note","verdict":"needs-revision","origin":"pre-existing","message":"A domain named fmt collides with the server package's fmt import."},
{"file":"crates/generate/ess-synth/tests/fixtures/generated-views.yaml","line":117,"category":"mutant","severity":"note","verdict":"approve","origin":"pre-existing","message":"Reversing the secondary order key of TasksByOwner survives the generated suite."},
{"file":"crates/generate/ess-synth/src/go/runtime/invariant.go","line":258,"category":"mutant","severity":"note","verdict":"approve","origin":"introduced","message":"All returning True instead of Unknown survives invariant_check."}]
```
