---
format: aep.planning-md/3
id: story:a-type-conformance-compares-value-constraints-and-variant-attributes
kind: story
status: active
title: A type conformance compares value constraints and enum variant attributes
summary: 'Adopter reports N17/N18: compose compares structure only; invariants, alphabet, prefix and variant attributes drift unseen (ess#671).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#671
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:a-composition-conformance-may-name-an-event-or-command
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/compose_conformance_v5.rs
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/constraint_conformance.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/constraint-conformance
- confidence: cited
  path: docs/design/composition-type-conformance.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:27Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:28Z", actor: "human:timo", revision: 7}
---
## Problem

Compose compares a type conformance's structure only. An owner type's invariants, newtype alphabet and prefix, and enum variant attributes can change without a consumer's compose failing; `conformance.rs:16-17` and `docs/design/composition-type-conformance.md:62-65` say these are not compared. Adopter reports N17 and N18, ess 0.68.0.

## Fit review

1. Need: when an owner tightens a value constraint or changes a variant's facts, a consumer's copy should fail compose. A repro kept outside the repository (`/4`, 0.74.0) exits 0 with three differences: an owner newtype invariant `value != "x"` the local copy lacks; `value >= 2` against `>= 1`; and `Kind.Closed` with `terminal` `false` against `true` and `handled_by` `robot` against `desk`. Requested: `type_conformance_drift` naming the type, the variant and the attribute or invariant.
2. Class: gap against a documented limit. `conformance.rs:16-17` and `docs/design/composition-type-conformance.md:62-65` say invariants, alphabet and prefix are "not compared", and admit that a stricter local copy "can reject a value the owner sends".
3. Expressible: no. The comparison never reads `ResolvedBody::{Newtype,Struct}.invariants`, `alphabet` or `prefix` (`ess-compiler/src/ir.rs:322-346`), or `EnumVariant.attributes` (`ess-domain/src/types.rs:664-705`). The enum arm compares names and wire spellings only (`conformance.rs:304-316`).
4. Fit: close the whole row of value constraints, not just invariants: newtype `alphabet`, `prefix` and `invariants`, struct `invariants`, and enum variant `attributes`. Display names and summaries stay out because they constrain no value. The rules:
   - Exact: equal invariant sets compared by parsed `Predicate` (it derives `PartialEq` and implements `Display`; `ess-primitives/src/predicate.rs:1471,4375`), so spelling is ignored; equal `alphabet` and `prefix`; the same declared attribute names with conforming types (a named attribute type is walked by `Comparison::named`); an equal value for each variant matched by name.
   - Reader (`reader: true`): every local invariant must appear in the imported set; the local alphabet must contain the imported one; the local prefix must be absent or a prefix of the imported one; every attribute the local enum declares must agree on each variant matched by wire name; the reader may omit attributes. This is the reader question "can the local reject an imported value?" (`composition-type-conformance.md:91-92`).
   - Payload ends from `story:a-composition-conformance-may-name-an-event-or-command` reach these rules through their fields.
5. A second adopter: a scheduler's copy of a `Priority` enum carries `escalates: Boolean`. The owner flips `High.escalates` and the copy goes stale.
6. Cost: no new code or keys. The `type_conformance_drift` meaning text widens (shared with the #664 story). The design page's "Not compared" paragraph is rewritten for `/5`. Known limit: invariants are compared syntactically, so a reader's weaker but differently written invariant (`>= 1` against owner `>= 2`) is reported as drift. A `String` attribute that holds a type name (the adopter's `request`/`response`) is compared as text.
7. Alternatives: (a) change nothing and keep the documented limit, which fails the adopter's 33 conformances silently; (b) compare under every format, which breaks "an `ess-composition/4` document keeps today's results"; (c) chosen: `/5` only, with the whole constraint row. Changed from the request: alphabet, prefix and struct invariants added; a subset rule for reader assertions.

## Decisions

Accept, redesigned, under `ess-composition/5`, the same format as `story:a-composition-conformance-may-name-an-event-or-command`. A `/2`-`/4` composition that holds today would start failing, which changes meaning (AGENTS.md "Determinism and formats"), so the new comparison runs only under `/5`. No `ess/N`.

Below `/5` the comparison is today's, byte for byte and result for result. Drift detail examples:

- `the type itself: invariant `value != "x"` in `wire.server.Code` and not in `demo.gate.Code``
- `variant `Closed`: attribute `handled_by` is `desk` where `wire.server.Kind` has `robot``

This story depends on `story:a-composition-conformance-may-name-an-event-or-command`, which lands first in the same unit; both share `conformance.rs`, `lib.rs` and the `/5` format.

## Acceptance

Command: `cargo test -p ess-composition --locked --test constraint_conformance <fn>`.

- `compose_compares_invariants`: owner `value != "x"`, local none, so drift naming the type and the invariant.
- `compose_compares_invariant_bounds`: `>= 1` against `>= 2`.
- `compose_compares_variant_handled_by`, `compose_compares_variant_terminal`, `compose_compares_variant_message_types` (`request`/`response` `String` attributes), each naming the type, the variant and the attribute.
- `compose_compares_struct_invariants` and `compose_compares_newtype_alphabet_and_prefix`.
- `an_invariant_spelled_differently_conforms`: `value>=1` against `value >= 1`.
- `a_reader_may_omit_an_owner_invariant_and_attribute` and `a_reader_invariant_the_owner_lacks_is_drift`.
- `compose_still_refuses_structural_drift`: wire spelling, a renamed or dropped variant, a field type, a dropped field and a presence change still fail.
- `v4_keeps_todays_result`: the repro shape composes `Ok` under `/4` and drifts under `/5`.
- CLI exit code: `compose_exits_non_zero_on_an_invariant_drift_under_v5` in `crates/edge/ess-cli/tests/compose_conformance_v5.rs`.

## Scope

- `crates/specify/ess-composition/src/conformance.rs`: `bodies` (282-381) gains constraint and attribute arms; module doc (1-30).
- `crates/specify/ess-composition/src/lib.rs`: `/5` passed into `Comparison` (it already carries `reader`); catalogue meaning.
- `docs/design/composition-type-conformance.md` (62-65, 106-107, 119-136), `website/docs/reference/spec-versions.md`, `formats.md`, `CHANGELOG.md`.
- Inferred: `crates/specify/ess-composition/tests/constraint_conformance.rs` and `tests/fixtures/constraint-conformance/` (the `demo` and `wire` systems); `crates/edge/ess-cli/tests/compose_conformance_v5.rs` (shared with the #664 story).
- Confidence: high for the crate and the documented limit; the defect site and the documented limit are both read in the tree.

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
