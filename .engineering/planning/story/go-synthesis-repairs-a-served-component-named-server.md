---
format: aep.planning-md/3
id: story:go-synthesis-repairs-a-served-component-named-server
kind: story
status: draft
title: Go synthesis of a served component named server does not claim one file twice
tags:
- adopter-report
- defect
- ess-0.70.0
- ess-0.70.0-first
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/go.rs
revision: 9
---
## Outcome

Go synthesis of a contract whose served component is named `server` writes each file once and
does not panic; the component's surface file takes a repaired name, as its package directory
already does.

## Evidence

An adopter on ess 0.56.0: Go synthesis of a two-role (server, client) protocol contract exits 101
with "panicked at crates/generate/ess-synth/src/lib.rs:489:5: two artifacts claimed
`server/server.go`; that is a defect in ess-synth". Rust synthesizes the same contract (45 of 48
capabilities, 3 obligations).

On `main` (read, not reproduced): the assertion is `insert` at
`crates/generate/ess-synth/src/lib.rs:490-491`. Two Go emitters can write `server/server.go`: the
helpers file of the server package (`go/http.rs:69`, named after the package, `go/layout.rs:57-58`)
and the per-component surface file (`go/http.rs:1207`, `{package.dir}/{module_stem(component)}.go`,
`module_stem` at `http.rs:1443` being the raw `package_ident`). `go/layout.rs:210-240` already
renames a component's package directory when `server` is taken; `module_stem` skips that repair.
Inferred cause: a served component named `server`. No commit since 0.56.0 touches it.

## Acceptance

Fixture: the brand-free `demo` model above (component `server`, `reached_by: network`), written by
the test.

- **No double claim.** Transition: `ess generate synthesize --target go` with the served component
  named `server`. Before: exit 101, `two artifacts claimed server/server.go` at
  `crates/generate/ess-synth/src/lib.rs:560`. After: exit 0; `server/server.go` is the package
  helpers file and the component's surface file takes a repaired name distinct from it, by the
  same repair `go/layout.rs:210-240` applies to package directories, here applied in
  `module_stem` (`go/http.rs:1600`). Test: `crates/generate/ess-synth/tests/go.rs`, case
  `a_served_component_named_server_gets_a_repaired_surface_file`; where `go` is on `PATH` the case
  also runs `go build ./...` on the output. Failing before: the case panics on the current tree.
  Mutation check: reverting the `module_stem` repair makes the case panic again.
- **Other names keep their bytes.** Transition: the control model with the component named
  `gateway`. After: the output tree is byte-identical to the current tree's (`server/server.go`
  beside `server/gateway.go`). Same file, case `a_served_component_not_named_server_keeps_its_file_names`.
- **A collision is a refusal.** Transition: two emitters claiming one path. Before: a panic at
  `lib.rs:560`. After: a refusal naming both artifacts and the path. Test: a unit test beside the
  claim in `crates/generate/ess-synth/src/lib.rs`, case `two_artifacts_claiming_one_path_are_refused`.

## Cause confirmed

The adopter's contract declares its only served component as `- component: server`, which is the
inferred cause above: the component's surface file and the server package's helpers file both
resolve to `server/server.go`. No reproducer follows; the test writes its own neutral contract.

## Reproduced (ess 0.56.0)

The adopter reran its contract: with the component named `server`, `--target go` panics on
`server/server.go`; with the same component renamed (any other name), it synthesizes
(48 capabilities, 45 generated, 3 obligations, 0 refused). The component name alone decides it.

## Adopter report 2026-10-22 (#10)

The same defect reported again on 2026-10-22 (item 10 of the second batch, relayed by the orchestrator), re-probed on 0.67.0
by the adopter, and reproduced here on 0.68.0 and 0.67.0 with a neutral model.

Brand-free fixture, `format: ess/29`: a system `demo` with domain `demo.server` (entity `Client`, commands `Handshake` and
`Publish`) and `components.yaml`:

```yaml
components:
  - component: server
    reached_by: network
    owns: {domains: [demo.server]}
    accepts: {commands: [demo.server.Handshake, demo.server.Publish]}
    publishes: {events: [demo.server.Handshaken, demo.server.Published]}
```

`ess generate synthesize --path <model> --target go --out <dir>`, exit 101 on 0.68.0 and 0.67.0, identical:

```text
thread 'main' panicked at crates/generate/ess-synth/src/lib.rs:560:5:
two artifacts claimed `server/server.go`; that is a defect in ess-synth
```

- `reached_by: network` is required: without it no `server/` package is generated and synthesis exits 0. `--http` is not needed.
- Control: the same model with the component named `gateway` synthesizes (exit 0), writing
  `server/server.go` (the package helpers file) beside `server/gateway.go` (the surface file).
- Current lines: the helpers file is `Package::file` (`crates/generate/ess-synth/src/go/layout.rs:57`, `{dir}/{name}.go`), the
  surface file `crates/generate/ess-synth/src/go/http.rs:1306` (`{package.dir}/{module_stem(component)}.go`), `module_stem` at
  `http.rs:1600`, the panic at `lib.rs:560`.
- Not a 0.68.0 regression: no commit in `0.67.0..52bd110dd` touches `go/http.rs` or `go/layout.rs`. Tagged for 0.69.0 first.

## Decisions

2026-10-22: wave A unit A6, released in 0.70.0. It lands before
`story:a-command-input-and-response-field-may-share-a-name-in-rust` (unit A7), which depends on it:
A7 reaches `lib.rs` (`:63`) and checks the Go target, whose layout this story changes. No format
change.
