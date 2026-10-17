---
format: aep.planning-md/3
id: story:go-synthesis-takes-a-module-path
kind: story
status: draft
title: Go synthesis takes a module path
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess generate synthesize --target go` takes `--module <path>`, as `ess generate client` already
does, and the emitted Go module, its imports and its `natsjs/` adapter module carry that path.
Without the flag the output keeps its bytes.

## Evidence

On ess 0.59.0, `ess generate synthesize --help` lists `--target` but no `--module`, while
`ess generate client --help` documents `--module <MODULE>` ("Go module identity, required only for
Go"). Synthesis fixes the host at `example.invalid`
(`crates/generate/ess-synth/src/go/layout.rs:34`, the module path `example.invalid/<name>` at
`layout.rs:66`). A consumer that vendors the synthesized module works around it with a `replace`
directive in its own `go.mod`.

## Acceptance

- `--module example.com/acme/billing` on the Go target emits `module example.com/acme/billing` and
  every import of the generated packages, the `natsjs/` module included, under that path; `go build`
  and `go vet` of the output pass.
- An invalid module path is refused before anything is written, with the rule it breaks.
- `--module` with any target other than `go` is refused by name.
- Without `--module`, every synthesized Go module keeps its bytes.
