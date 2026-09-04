---
title: Getting started
description: Validate, compile, inspect, generate, and check the committed billing specification.
---

# Getting started

## Install the command

The preferred installation is a verified release archive. Each release publishes the `ess` binary
for four native targets:

| Machine | Target |
|---|---|
| Linux x86-64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

Choose the target for your machine. This example downloads the current `0.9.1` release for Apple
Silicon, verifies the archive before extracting it, and runs the binary in place:

```shell-session
$ version=0.9.1
$ target=aarch64-apple-darwin
$ archive="ess-${version}-${target}.tar.gz"
$ base="https://github.com/beyond10x/ess/releases/download/${version}"
$ curl --fail --location --remote-name "${base}/${archive}"
$ curl --fail --location --remote-name "${base}/SHA256SUMS"
$ grep -F "  ${archive}" SHA256SUMS | shasum -a 256 --check
ess-0.9.1-aarch64-apple-darwin.tar.gz: OK
$ tar -xzf "${archive}"
$ "./ess-${version}-${target}/ess" --version
ess 0.9.1
```

`SHA256SUMS` covers all four archives. Filtering the exact filename lets the checksum tool verify
the one archive you downloaded without treating the other three as missing files.

If your machine is not in the release matrix, or you need to work from current `main`, build the
locked Rust workspace from a source checkout:

```shell-session
$ cargo build --locked --release --bin ess
$ ./target/release/ess --version
ess 0.9.1
```

The walkthrough uses the source-checkout form so every path names a file in the repository. You can
replace `cargo run --quiet --locked --bin ess --` with the verified `ess` binary in every command
below.

The first level of `ess` is the four areas the tool is built out of — `specify`, `generate`,
`verify`, `infra` — and every verb is also spelled flat at the top level as a hidden alias, so
`ess validate --path .` still runs `ess specify validate --path .` and prints the same bytes. See
[Flat spellings](./reference/cli.md#flat-spellings).

## Validate and compile a real specification

The billing example is split into a system document and domain documents. Validation resolves the
whole set before reporting success:

```shell-session
$ cargo run --quiet --locked --bin ess -- specify validate --path examples/billing
billing v3 — 5 file(s), valid

$ cargo run --quiet --locked --bin ess -- specify compile \
    --path examples/billing --out target/billing.ir.json
billing v3 — 5 file(s), 26 declaration(s), compiled to target/billing.ir.json
```

Compilation writes canonical JSON. Running it twice with the same validated input produces the same
bytes.

## Inspect the model

```shell-session
$ cargo run --quiet --locked --bin ess -- specify graph \
    --path examples/billing --format mermaid

$ cargo run --quiet --locked --bin ess -- specify inspect \
    --path examples/billing billing.invoice.Invoice
```

Names resolve before inspection. An unknown or ambiguous name is a refusal, not an empty result.

## Generate documentation from the specification

`docs` projects the validated model into Markdown and Mermaid. `site` projects the same pages with
YAML frontmatter and a deterministic `sidebar.json`, ready to feed into a static site generator:

```shell-session
$ cargo run --quiet --locked --bin ess -- generate \
    --path examples/billing --kind docs --out target/projections

$ cargo run --quiet --locked --bin ess -- generate \
    --path examples/billing --kind site --out target/projections
```

The resulting entry pages are `target/projections/docs/index.md` and
`target/projections/site/index.md`; the site inventory is
`target/projections/site/sidebar.json`.

The direction matters: ESS reads the typed YAML specification and generates documentation from it.
It does not interpret an existing Markdown document as a specification. The `site` projection also
does not emit HTML, a theme, or a hosted site; those remain the responsibility of the documentation
system that consumes the generated Markdown and sidebar.

## Generate contracts

```shell-session
$ cargo run --quiet --locked --bin ess -- generate \
    --path examples/billing --kind openapi --out target/projections

$ cargo run --quiet --locked --bin ess -- generate \
    --path examples/billing --kind asyncapi --out target/projections
```

The outputs are projections of validated IR. They are not hand-maintained descriptions beside the
specification.

`--out` is the output-tree root. Each generated path already begins with its projection name, so
these commands write beneath `target/projections/openapi/` and
`target/projections/asyncapi/` rather than directly beneath the path supplied.

## Generate and run conformance

```shell-session
$ cargo run --quiet --locked --bin ess -- verify conform synthesize \
    --path examples/billing --out target/billing-suite.json

$ cargo run --quiet --locked --bin ess -- verify conform run \
    --suite target/billing-suite.json --target billing
```

The report is standalone ESS output. It names the specification digest and the scenarios that were
checked.

## Continue

- [Write a specification](./guides/write-a-specification.md)
- [Generate artifacts](./guides/generate-artifacts.md)
- [Verify conformance](./guides/verify-conformance.md)
- [Import or project infrastructure](./guides/check-infrastructure.md)
- [Use the complete CLI reference](./reference/cli.md)
