---
title: Getting started
description: Validate, compile, inspect, generate, and check the committed billing specification.
---

# Getting started

## Build the command

ESS is a Rust workspace. From a checkout:

```shell-session
$ cargo build --locked --bin ess
$ cargo run --quiet --locked --bin ess -- --version
ess 0.1.1
```

You can replace `cargo run --quiet --locked --bin ess --` with an installed `ess` binary in every
command below.

## Validate and compile a real specification

The billing example is split into a system document and domain documents. Validation resolves the
whole set before reporting success:

```shell-session
$ cargo run --quiet --locked --bin ess -- validate --path examples/billing
valid

$ cargo run --quiet --locked --bin ess -- compile \
    --path examples/billing --out target/billing.ir.json
wrote target/billing.ir.json
```

Compilation writes canonical JSON. Running it twice with the same validated input produces the same
bytes.

## Inspect the model

```shell-session
$ cargo run --quiet --locked --bin ess -- graph \
    --path examples/billing --format mermaid

$ cargo run --quiet --locked --bin ess -- inspect \
    --path examples/billing billing.invoice.Invoice
```

Names resolve before inspection. An unknown or ambiguous name is a refusal, not an empty result.

## Generate contracts

```shell-session
$ cargo run --quiet --locked --bin ess -- generate \
    --path examples/billing --kind openapi --out target/openapi

$ cargo run --quiet --locked --bin ess -- generate \
    --path examples/billing --kind asyncapi --out target/asyncapi
```

The outputs are projections of validated IR. They are not hand-maintained descriptions beside the
specification.

## Generate and run conformance

```shell-session
$ cargo run --quiet --locked --bin ess -- conform synthesize \
    --path examples/billing --out target/billing-suite

$ cargo run --quiet --locked --bin ess -- conform run \
    --suite target/billing-suite --target billing
```

The report is standalone ESS output. It names the specification digest and the scenarios that were
checked.

## Continue

- [Write a specification](./guides/write-a-specification.md)
- [Generate artifacts](./guides/generate-artifacts.md)
- [Verify conformance](./guides/verify-conformance.md)
- [Import or project infrastructure](./guides/check-infrastructure.md)
- [Use the complete CLI reference](./reference/cli.md)
