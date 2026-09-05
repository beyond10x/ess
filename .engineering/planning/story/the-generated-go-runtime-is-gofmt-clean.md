---
format: aep.planning-md/1
id: story:the-generated-go-runtime-is-gofmt-clean
kind: story
status: draft
title: The emitted Go runtime is not gofmt-stable, so an adopter's formatter changes it
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/verify/ess-conformance
revision: 6
---
## What is wrong

`crates/ess-conformance/src/go/runtime.go` is a Go file inside a Rust crate. `cargo fmt` does not
see it, `task check` does not `gofmt` it, and its struct tags are aligned by whoever last edited
them.

The generator copies it out verbatim. So an adopter who runs `gofmt` over the tree it was written
into — which is a normal thing to do while editing the hand-written half beside it — changes bytes
the generator did not choose, and the next `ess conform synthesize` puts them back.

Observed in `sbf/acd` on 2026-09-03: `gofmt -w tests/conformance/` while editing
`tests/conformance/target.go` reformatted `tests/conformance/essconform/runtime.go` beside it. The
reformatted bytes were committed and pushed, and `task ess:check` — which regenerates and compares
— went red on a tree nobody had meaningfully changed.

```diff
-	Payload map[string]Node `json:"payload,omitempty"`
-	Shape   map[string]Held `json:"shape,omitempty"`
+	Payload     map[string]Node `json:"payload,omitempty"`
+	Shape       map[string]Held `json:"shape,omitempty"`
```

Nine lines, no semantic difference, and a red drift check on the adopter's branch.

## Why it is this crate's problem and not the adopter's

Every adopter of the Go target has a directory holding one generated file and one hand-written
file, and `gofmt` is directory-scoped. Telling each of them *do not format that directory* is a
convention in a comment; emitting bytes `gofmt` already agrees with is a property.

## Acceptance

- `crates/ess-conformance/src/go/runtime.go` and `predicate.go` are `gofmt`-clean.
- `task check` fails when they are not — a step that runs `gofmt -l` over
  `crates/ess-conformance/src/go/` and refuses non-empty output. Go is not a build dependency of
  this workspace, so this step is skipped rather than failed where `gofmt` is absent, and says so.
- The generated tree in `sbf/acd` is byte-identical before and after `gofmt -w`.

## Scope

Derived 2026-09-05 by `aep-drive:story-scoper` from the story and current repository tree — cited.

- **Primary surface:** `crates/verify/ess-conformance` — cited; the story's pre-area paths now resolve to this crate's `src/go/runtime.go` and `src/go/predicate.go`.
- **Emitter evidence:** `crates/verify/ess-conformance/src/go/mod.rs:49` defines `emit`; lines 56–57 copy both Go source files verbatim through `include_str!`, so formatting those sources directly changes the emitted bytes — cited.
- **Gate surface:** `Taskfile.yml` — cited; the story explicitly requires `task check` to detect formatting drift and report a skip when `gofmt` is absent. Its current formatting task starts at line 4, and the complete gate starts at line 151.
- **Also likely:** `crates/edge/ess-xtask` — inferred; repository guidance requires executable checking logic in Rust, and `src/main.rs:54` and `src/main.rs:105` provide the existing maintenance-command definition and dispatch. This is the likely home for the optional-tool lookup, `gofmt -l` invocation, and non-empty-output refusal.
- **Tests:** any formatter-check coverage and emission-stability checks belong within the corresponding crate surfaces above; no narrower test token is needed — inferred.
- **Documents:** no documentation edit is required by the acceptance — inferred.
- **Confidence:** high — cited; the story identifies both embedded Go sources and the required gate change, and the current emitter confirms verbatim copying. The checker placement remains an implementation choice.
- **Would collide with:** any unit touching the exact tokens `crates/verify/ess-conformance`, `Taskfile.yml`, or the likely checker surface `crates/edge/ess-xtask` — inferred.