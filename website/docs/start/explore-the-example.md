---
title: Explore the repository example
description: Validate, compile, inspect and document the larger billing example from a checkout of the repository, and run its suite against the built-in reference implementation.
---

# Explore the repository example

The repository's `examples/billing/` is a larger specification: two domains, cross-domain bindings,
both view consistencies and a type of every kind. From a checkout of the repository, the commands
below run against it. Replace `ess` with `cargo run --quiet --locked --bin ess --` to use the
checkout's own build.

```shell-session ess-tutorial checkout
$ ess specify validate --path examples/billing
billing v3 — 5 file(s), valid

$ mkdir -p target
$ ess specify compile --path examples/billing --out target/billing.ir.json
billing v3 — 5 file(s), 26 declaration(s), compiled to target/billing.ir.json
```

Compilation writes canonical JSON. Running it twice with the same input produces the same bytes.

Inspect one declaration, or the interaction graph. Names resolve before inspection, so an unknown or
ambiguous name is a refusal rather than an empty result:

```shell-session ess-tutorial checkout
$ ess specify inspect --path examples/billing billing.invoice.Invoice
entities:
  domain: billing.invoice
…
$ ess specify graph --path examples/billing --format mermaid
flowchart TB
…
```

Generate documentation. `docs` writes Markdown with Mermaid diagrams; `site` renders the same pages
as a browsable HTML site with its own stylesheet and diagram renderer:

```shell-session ess-tutorial checkout
$ ess generate --path examples/billing --kind docs --out target/projections
…
docs/index.md — 4140 byte(s)
…
6 artifact(s), written to target/projections
$ ess generate --path examples/billing --kind site --out target/site
…
index.html — 11793 byte(s)
…
9 artifact(s), written to target/site
```

The entry pages are `target/projections/docs/index.md` and `target/site/index.html`.
ESS generates documentation from the typed specification; it does not read Markdown as a
specification, and it does not host the site.

Run the billing suite against the repository's own reference implementation of billing, which is
built into `ess`:

```shell-session ess-tutorial checkout
$ ess verify conform synthesize --path examples/billing --out target/billing-suite.json
32 scenario(s) (0 authored), 0 refusal(s), written to target/billing-suite.json
$ ess verify conform run --suite target/billing-suite.json --target billing
…
  32 scenarios: 32 passed, 0 failed, 0 error, 0 unsupported
```

The built-in targets (`billing`, `oracle-fixture`, `interpreted`) exist to demonstrate the runner;
your own implementation is held to its suite through a runner, as in
[Run your first conformance suite](./first-conformance-run.md).
