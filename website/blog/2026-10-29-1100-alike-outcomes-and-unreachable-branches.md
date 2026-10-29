---
title: "0.77 — Alike HTTP outcomes, wider OpenAPI import, and branches an earlier refusal hides"
description: >
  0.77.0 lets an HTTP route answer two outcomes alike, imports open objects from OpenAPI, keeps
  declared field order in generated types on request, and refuses a branch an earlier refusal
  always answers first.
slug: alike-outcomes-and-unreachable-branches
tags: [release, ess]
date: 2026-10-29T11:00:00+01:00
release_tag: "0.77.0"
---

0.77.0 is a feature release with no new source format: every model keeps its meaning, its IR bytes
and its compiled digest. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md)
lists every change since 0.70.0; this note covers 0.77.0 and the releases since the last note.

{/* truncate */}

## HTTP bindings

- A route may declare `alike:` groups of outcomes whose answers are equal, so two outcomes share a
  status and a body instead of being refused as indistinguishable; OpenAPI writes one response
  naming every member ([#652](https://github.com/beyond10x/ess/issues/652)).
- An outcome may answer a `String` field as `text/plain` with `encoding: text`. A route may declare
  its own `refusals:`, a literal or empty body for `empty`, and `body: array` for a view that is not
  paged ([#673](https://github.com/beyond10x/ess/issues/673)).

## Import and generated types

- OpenAPI import admits open objects, the empty and `true` schemas as JSON values, and a nullable
  `anyOf`/`oneOf` pair as an optional value
  ([#647](https://github.com/beyond10x/ess/issues/647)).
- `ess generate schema types-bundle --integer-format-widths` realizes `int32` and `int64` at native
  width ([#648](https://github.com/beyond10x/ess/issues/648)).
- `ess generate types --declared-field-order` keeps the order the specification declares, and
  `--rust-box-only-recursion` boxes only the references that close a cycle
  ([#661](https://github.com/beyond10x/ess/issues/661)). Without these options the output keeps
  its bytes.
- Rust and web synthesis accept a response field that shares a name with an input field
  ([#659](https://github.com/beyond10x/ess/issues/659)).

## Validation and mutation

- `ess specify validate` refuses a `when:` branch whose input guard implies the guard of an
  input-guarded refusal answered before it, as `unreachable_branch`
  ([#702](https://github.com/beyond10x/ess/issues/702)). Before, only synthesis reported it.
- `ess verify conform mutate` scores a precedence swap of two guards no input selects together as
  `equivalent`, also for wide enums, membership lists and disjunctions
  ([#703](https://github.com/beyond10x/ess/issues/703)).

## Since 0.70.0

- 0.76.0: schemas publish entity invariants and `Binary64`, text length and distinct-list bounds as
  keywords; a `defined` / `not defined` pair of branches is exhaustive without a default;
  `ess-composition/5` compares events, commands, constraints and variant attributes; HTTP bindings
  name operation ids and servers.
- 0.75.0: the workspace builds on its declared minimum Rust version again, and seeded synthesis
  explains the seeds it cannot use.
- 0.74.0: seeds are admitted for models whose actors declare caller attributes.
- 0.71.0 to 0.73.0: synthesis covers more set updates, deletes, bindings and related-row guards that
  it refused before.
