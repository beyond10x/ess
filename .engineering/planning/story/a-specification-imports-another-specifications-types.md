---
format: aep.planning-md/3
id: story:a-specification-imports-another-specifications-types
kind: story
status: draft
title: ess specify vendor generates a checked copy of another specification's type
summary: 'Adopter report N9: No way to import types across specifications (ess#663).'
tags:
- ess-0.71.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#663
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
- decomposes: epic:zero-open-issues-2026-11
revision: 6
---
## Need

A consumer specification that uses an owner specification's types cannot name them: a type of
another specification is an undeclared reference, and `ess-inputs` refuses files outside the root.
The designed path is a local copy under the consumer's own names, proved against the pinned owner by
`ess specify compose` `conformances:` (`docs/design/composition-type-conformance.md`). That path
works and catches drift, but the copy is written by hand, field by field and nested type by nested
type, and re-written by hand when the owner changes. Later reports (2026-10-26) show the cost: other
consumers hold an owner's document type as `Json`, or copy it and convert one to one.

Minimal reproduction on 0.92.0 (owner `example.owner` declaring `example.owner.orders.OrderSummary`
with a newtype and an enum; consumer `example.consumer`):

```text
$ ess specify validate --path consumer-ref        # names example.owner.orders.OrderSummary
  - error[ESS-TYPE-001]: `example.owner.orders.OrderSummary` is not a declared type
exit 1
$ ess specify compose --path composition.yaml --service owner=owner --service consumer=consumer
example — 2 exact component surface(s), 0 semantic reference(s), compiled
exit 0                                             # the hand copy, proved
$ ess specify compose --path composition-drift.yaml --service owner=owner --service consumer=consumer-drift
[type_conformance_drift] `consumer` type `example.consumer.board.OrderSummary` does not conform to `owner` type `example.owner.orders.OrderSummary`: field `stage`: variants [`Placed`] where `example.owner.orders.Stage` has [`Placed`, `Shipped`]
exit 1
$ ess specify vendor --help
error: unrecognized subcommand 'vendor'
exit 2
```

## Acceptance

- `ess specify vendor --from owner --type example.owner.orders.OrderSummary --into
  example.consumer.board --out vendored.yaml --conformances conformances.yaml` (exit 0) writes:
  `vendored.yaml`, a source file with `domain: example.consumer.board` declaring
  `example.consumer.board.OrderSummary`, `.OrderId` and `.Stage` (the named closure of the type,
  renamed into the domain, every field, wire name, presence, invariant and variant attribute kept),
  headed by a comment naming the owner's system, version and IR digest; and `conformances.yaml`,
  the owner's `services:` entry pinned to that digest and one `conformances:` entry per vendored
  type.
- Dropping `vendored.yaml` into the consumer and composing with the written entries passes
  `ess specify compose` with no `type_conformance_drift`; the same command run twice writes
  identical bytes.
- `ess specify vendor --check` with the same arguments exits 0 when the files match what it would
  write and exits 1 naming each differing type when the owner changed (scenario
  `an_imported_type_changed_upstream_is_refused_until_repinned`).
- Refused by name, exit 1: a `--type` the owner does not declare; a renamed type colliding with a
  type the consumer already declares under another shape; an `--into` that is not a qualified
  domain name.
- No source, composition or IR format changes; `docs/design/composition-type-conformance.md` gains
  a section "Generating the copy".

## Scope

- cited: crates/edge/ess-cli/src/main.rs (`SpecifyCommand`, line 147)
- inferred: crates/edge/ess-cli/src/vendor.rs (new)
- inferred: crates/specify/ess-composition/src/vendor.rs (new; reuses the shape walk of crates/specify/ess-composition/src/conformance.rs)
- inferred: crates/specify/ess-composition/src/lib.rs
- inferred: crates/edge/ess-cli/tests/vendor.rs (new)
- cited: docs/design/composition-type-conformance.md
- inferred: website/docs/reference/cli.md
confidence: medium

## Fit review

1. **Need.** A consumer uses an owner's type without writing and maintaining a copy by hand, and
   still has the copy checked against the pinned owner. Reproduction above
   (`triage/663/repro/run.txt`). Requester's proposal: a source-level import of another
   specification's types (issue body).
2. **Class: convenience.** The fact is expressible and checked today: the hand copy plus
   `conformances:` passes and catches drift (run above;
   `docs/design/composition-type-conformance.md:13-37`, comparison at 39-76). What is missing is
   the writing of the copy.
3. **Already expressible?** Yes, and the idiom keeps the checked figure in the suite (composition
   refuses a drifted copy), so it passes policy 1 (`docs/design/generic-language-features.md:588-594`).
   The operator chose to remove the hand work with a generator rather than decline
   (`approval-record:d132-types-across-specifications-by-vendor`).
4. **Fit.** No grid cell (`docs/design/language-grids.md`): no authored construct. Reuses the
   composition's conformance as the proof and its structural comparison to define what "a copy" is;
   ownership is unchanged (the copy is owned by the consumer's domain). Targets: generated code,
   synthesis, `ess verify diff` and Entity Runtime see an ordinary local type and need nothing.
5. **Second adopter.** A reporting service that reads an orders service's `OrderSummary` and a
   billing service's `InvoiceLine` (the 2026-10-26 reports: two more consumers).
6. **Cost.** One CLI subcommand and a library function; no format, no diagnostic code in a source
   format, no migration. Generated code still has two types (owner's and consumer's), as today.
7. **Alternatives** (`docs/plan/wave-ess-issues-1.md` § #663): (A) a source `imports:` key: a new
   `ess/N` and IR shape, every IR reader changes, exceptions to the one-owner rule; not chosen, stays
   possible later. (B) chosen: `ess specify vendor`. (C) decline with the idiom: leaves the copies
   hand-written.

## Decisions

**accept, redesigned.** The requested source import is not adopted; the need is met by
`ess specify vendor`, which generates the copy and the `conformances:` entries that prove it, with no
source-format change (D132, `approval-record:d132-types-across-specifications-by-vendor`, approved
2026-10-28). The reserved scenario names change: `an_imported_type_resolves_by_pinned_digest` becomes
`a_vendored_copy_composes_without_drift`; `generated_code_references_the_imported_type_once` is
dropped (two types remain by design).

## Source

github:beyond10x/ess#663
