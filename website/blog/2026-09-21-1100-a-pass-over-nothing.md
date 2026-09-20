---
title: "0.19–0.21 — adopter schemas, a number that behaves, and a pass over nothing"
description: >
  Checked JSON Schema imports with source identity and source-pinned normalization recipes; a
  `Binary64` field that preserves signed zero, subnormals and nearest-even rounding across three
  languages; and a conformance suite that records what it left out, so a pass over nothing and a
  pass over everything stop reading the same.
slug: a-pass-over-nothing
tags: [release, ess, conformance]
date: 2026-09-21T11:00:00+02:00
release_tag: "0.21.0"
release_commit: 988f219e90c9a7b105daec2345f77d6035075e92
---

Three releases that each close a place where the toolchain was quietly agreeable: an adopter's own
schemas, floating-point arithmetic, and a green suite.

{/* truncate */}

## Somebody else's schemas, checked

`ess` can import complete JSON Schema roots **with source identity** and generate structural Go,
Rust and TypeScript libraries from them, then run source-pinned normalization recipes over the
result. Rust normalization has a library API; Go and TypeScript support is pending and says so.

Rust and Web synthesis return checked failures rather than emitting a target that would disagree,
and sliced provenance names its digest profile instead of implying one.

The theme is the same as everywhere else in this repository: an adopter's document is an authority
about itself, admitted or refused, never quietly adapted.

## A field that is a floating-point number

Authored `ess/2` adds `Binary64` fields, distinct from integer and decimal values. The reference,
Rust and Go implementations preserve **signed zero, subnormals and nearest-even rounding**, and two
typed operands compare with IEEE equality.

The interesting part is the refusal: synthesis and conformance refuse an unsupported `Binary64`
**before publication**, rather than emitting a target that would disagree with the specification at
run time. A generator that emits something plausible for a semantics it does not implement produces
a passing suite and a wrong system.

`Binary64` requires major 2 or later at every declared position, and map keys refuse it.

## A suite that says what it left out

This is the one that changes what a green run means.

Opt-in `ess-conformance/5` records the selected generated and authored coverage, the omitted
scenarios, the source identities and **every refusal occurrence**. Report 2 then distinguishes
complete passing conformance from empty, unknown or incomplete coverage.

Before it, a suite that executed nothing and a suite that executed everything both reported "all
pass". Ordinary coverage remains unknown, including for an all-pass run — that is stated rather
than inferred. Only a nonempty all-pass execution with a complete inventory and no in-scope refusal
qualifies, and it qualifies **for that exact selection**, not for the system.

An extended suite requires explicit `--report-format 2`; suites /5 and later with report/1 refuse
before execution, including when no output destination is requested. Report 1 keeps its historical
non-pass aggregate and does not establish exact suite-byte identity.
