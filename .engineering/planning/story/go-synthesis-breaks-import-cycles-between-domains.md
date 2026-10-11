---
format: aep.planning-md/3
id: story:go-synthesis-breaks-import-cycles-between-domains
kind: story
status: active
title: Go synthesis emits a module that compiles when domain references form a loop
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-11T11:03:12Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-11T11:03:12Z", actor: "human:timo", revision: 3}
---
## Outcome

Go synthesis (`ess generate synthesize --target go`) of every model that `ess specify validate`
accepts emits a module that compiles, or validation and synthesis refuse a cross-domain reference
loop with a diagnostic naming the domains in it. Today the Go target emits one package per domain
under `types/`, and a loop of references between domains makes those packages import each other.

## Evidence

An adopter on ess 0.58.0. Three domains `<sys>.publish`, `<sys>.session` and
`<sys>.subscription`:

- a command outcome in `publish` refuses with an error declared in `session`
  (`error: <sys>.session.StoreUnavailable`);
- `session` references `subscription`;
- `subscription` references `<sys>.publish.Channel`.

`ess specify validate` accepts the model; `ess generate synthesize --target go` writes the module;
`go build` fails with `import cycle not allowed` (publish → session → subscription → publish).
The adopter works around it by declaring its own error in `publish` with the same wire answer.

The Go target already places the refusal type and the owed conversions in packages of their own
because Go forbids an import cycle (`crates/generate/ess-synth/src/go/layout.rs:77-84`,
`crates/generate/ess-synth/src/go/obligation.rs:9-16`); the per-domain type packages have no such
rule. Rust modules may reference each other, so the Rust target is not affected.

## Acceptance

- A Go synthesis test over a three-domain model whose references form the loop above: either the
  generated module builds with `go build` where a Go toolchain is present (shared declarations in a
  common package, or one package per strongly connected set of domains), or validation or synthesis
  refuses it with a diagnostic that names the three domains of the loop.
- The same test without the loop (the error declared in `publish`) synthesizes one package per
  domain, as today, and builds.
- A model whose domain references contain no loop keeps byte-identical Go output.
