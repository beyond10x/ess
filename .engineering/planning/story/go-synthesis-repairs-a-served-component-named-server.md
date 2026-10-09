---
format: aep.planning-md/3
id: story:go-synthesis-repairs-a-served-component-named-server
kind: story
status: draft
title: Go synthesis of a served component named server does not claim one file twice
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
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

- A Go synthesis test over a neutral two-role contract whose served component is named `server`:
  synthesis exits 0, both files are written under distinct names, and the generated module builds.
- A collision between two emitted paths is a refusal naming both artifacts, not a panic.

## Cause confirmed

The adopter's contract declares its only served component as `- component: server`, which is the
inferred cause above: the component's surface file and the server package's helpers file both
resolve to `server/server.go`. No reproducer follows; the test writes its own neutral contract.

## Reproduced (ess 0.56.0)

The adopter reran its contract: with the component named `server`, `--target go` panics on
`server/server.go`; with the same component renamed (any other name), it synthesizes
(48 capabilities, 45 generated, 3 obligations, 0 refused). The component name alone decides it.
