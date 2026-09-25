---
format: aep.planning-md/2
id: story:a-conformance-suite-runs-against-any-realization
kind: story
status: archived
title: A conformance suite runs against any realization
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

```console
$ ess verify conform run --help
      --target <TARGET>  [possible values: billing, oracle-fixture]
```

Two built-in reference implementations, and no route to a realization somebody wrote. The Rust route
is the `ess-conformance` crate, used by this repository's own
`examples/billing-realization/tests/conformance.rs`:

```console
$ cargo search ess-conformance
(no result)
```

Unpublished, so an adopter depending on it needs a checkout of this repository at a path, which
makes their workspace unbuildable for anyone without one.

## What that cost

The adopting project generated **47 scenarios** from its model and executed none of them. What held
its realization to the specification was 71 hand-written assertions — which check the fake against
the author's reading of the model, which is the thing conformance exists to replace.

## Acceptance

Either `ess verify conform run` accepts a realization an adopter supplies — a process speaking the
JSON wire the `web` target already speaks would be enough — or `ess-conformance` is published so the
Rust route is open to somebody without this repository.

## Cheapest form

The suite is JSON and the `web` target's module is JSON in, JSON out. A runner that feeds one to the
other needs neither the crate nor a new CLI surface.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
