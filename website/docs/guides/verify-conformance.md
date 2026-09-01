---
title: Verify conformance
description: Generate the semantic suite a specification requires, run it, and emit a standalone ESS conformance report.
---

# Verify conformance

A specification declares more than an interface shape. Command outcomes, refused branches, state
transitions, emitted events, and invariants become semantic scenarios an implementation can be
checked against.

## Generate the suite

```shell-session
$ ess conform synthesize \
    --path examples/billing \
    --out target/billing-suite
```

The suite is deterministic. Its provenance names the model and contract digests from which it was
derived.

## Run a supported target

```shell-session
$ ess conform run \
    --suite target/billing-suite \
    --target billing \
    --report-out target/billing-conformance.json
```

The command executes the generated or committed scenarios against the selected reference target.
The report uses the standalone `ess-conformance-report/1` format and records each scenario outcome.

To run directly from a specification instead of a pre-generated suite:

```shell-session
$ ess conform run \
    --path examples/billing \
    --target oracle-fixture \
    --format json
```

## What the report proves

The report proves what its scenarios observed against the named target and specification digest. It
does not prove that the target is independently operated, deployed in production, or free from
behavior the suite never exercised. A consumer may translate this report into its own evidence
vocabulary at that consumer’s boundary; ESS itself publishes no workflow or planning record.

## Add a target

The built-in targets demonstrate the runner contract. A new target implements the Rust
`ConformanceTarget` boundary and must preserve scenario identity, request/response correlation,
refusal semantics, and deterministic reporting. It must not report its own unobserved success as a
verifier result.
