---
format: aep.planning-md/3
id: review-result:go-generated-behaviour-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:go-generated-behaviour (wave ui-live-apps-w5)
relations:
- reviews: story:go-generated-behaviour
revision: 1
---
needs-change

Adversary pass 1 on story:go-generated-behaviour at 5ce49862d. Cases executed 10 to 14 (Go-filtered), red 4; introduced 4, pre-existing 1. Test file: crates/generate/ess-synth/tests/adversary_go_behaviour_pass1.rs.

- warning: after register and admit, VisitById returns a badge from the Rust gatepass server and none from Go; the spec stores none (examples/gatepass/domains/visit.yaml:206-216); the Rust realization writes it by hand (visit.rs:147). Coordinator decision: the spec decides; the Rust realization drops the write.
- warning: two owed seams with one Go method name are left off Generated with no TARGET.md row.
- warning: a domain named equal, some or verity clashes with a behaviour.go helper.
- note: a field named broken_invariant collides with the BrokenInvariant method.
- warning, pre-existing: adversary_served_pass1_gatepass.rs sends no Authorization; every command answers 403, so parity and race never reach a behaviour.
- note: a nil behaviour.Ports field compiles and panics at run time; TARGET.md does not say so.
- note: existence.rs (must-not-change) gained a doc-only edit (coordinator: the Go line was stale; accepted).

Held: no data race over 8x15 authorised calls (120 rows); evaluation order, invariant timing, context minting, existence lookup, aggregates at the i128 limits, ties-to-even, list order; the Decimal exponent limit is unreachable; generated/rust, PLAN.md, plan.json unchanged; TARGET.md 7 to 5 correct; integer increment overflow matches Rust release.

```findings
[{"file":"examples/gatepass-go-realization/visit.go","line":12,"category":"contract-drift","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"After register and admit, GET /visits/views/by-id returns the badge from the Rust server and no badge from the Go server."},
{"file":"crates/generate/ess-synth/src/go/mod.rs","line":594,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"Two owed seams with the same Go method name are left off Generated with no TARGET.md row."},
{"file":"crates/generate/ess-synth/src/go/behaviour.rs","line":518,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"A domain package named equal, some or verity clashes with a behaviour.go helper and the Go module does not build."},
{"file":"crates/generate/ess-synth/src/go/invariant.rs","line":132,"category":"boundary","severity":"note","verdict":"needs-revision","origin":"introduced","message":"An entity field broken_invariant exports to the same name as the BrokenInvariant method."},
{"file":"crates/generate/ess-synth/tests/adversary_served_pass1_gatepass.rs","line":106,"category":"mutant","severity":"warning","verdict":"needs-revision","origin":"pre-existing","message":"The parity and race tests send no Authorization header, so every command answers 403."},
{"file":"crates/generate/ess-synth/src/go/behaviour.rs","line":185,"category":"judgement","severity":"note","verdict":"approve","origin":"introduced","message":"A nil field in behaviour.Ports compiles and panics at runtime, and TARGET.md does not say so."},
{"file":"crates/generate/ess-synth/src/existence.rs","line":6,"category":"judgement","severity":"note","verdict":"approve","origin":"introduced","message":"A must-not-change file was edited, though only its doc comment."}]
```
