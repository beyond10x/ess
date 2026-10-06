---
title: "0.55 — Json in CLI results, and optional values an implementation supplies"
description: >
  0.55.0 lets a CLI result carry a JSON document as JSON, and routes an optional generated value
  through a port the implementation answers.
slug: json-cli-results-and-optional-generator-ports
tags: [release, ess]
date: 2026-10-06T22:00:00+02:00
release_tag: "0.55.0"
---

0.55.0 is a feature release. Specifications, suites and deltas keep their formats. It follows
0.54.0, released the same day with source format `ess/23`; the
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists both.

{/* truncate */}

## A CLI result carries JSON as JSON

A CLI result field may now be typed `Json`, alone or under `Optional`, `List` or a `String`-keyed
`Map`. The generated adapter writes the value as JSON, so a caller parses the answer once instead
of decoding JSON text out of a JSON answer.

- The result check accepts any JSON value in that field and refuses a reply outside the declared
  shape as `cli_result`.
- A `Json` input field is refused: an argv word has no JSON spelling a binding declares. A `Json`
  error field is refused too.
- Generated CLI packages carry new `src/wire.rs` bytes, so `ess generate cli --check` reports drift
  until they are regenerated (beyond10x/ess#468).

## An optional generated value comes from the implementation

`{generated: true}` on a value of type `Optional<T>` used to be filled absent every time. From
0.55.0 the generated Rust and Go behaviours read it from a context port that answers an optional
`T`, so a value that is absent only when there is nothing to report can reach the event.

- The port is `generate_optional_<t>` returning `Option<T>` in Rust and `GenerateOptional<T>`
  returning `*T` in Go. A context that implements the port adds the method; the generated
  demonstration context answers it absent.
- A field nothing sets stays absent and asks no port (beyond10x/ess#467).
