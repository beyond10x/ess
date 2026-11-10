---
format: aep.planning-md/3
id: story:ess-verify-diff-reports-a-changed-exchange
kind: story
status: draft
title: ess verify diff reports a changed exchange
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-exchange-is-declared-in-the-specification
scope:
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/compatibility.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/raw.rs
- confidence: inferred
  path: crates/verify/ess-diff/tests/exchanges.rs
- confidence: inferred
  path: docs/design/message-exchanges.md
revision: 6
---
## Need

Two revisions of a specification that differ in an exchange (added, removed, a message moved
between sides, a `closes` state added, `max_followers` changed) must be classified by
`ess verify diff`, as every other model construct is. Without it a breaking change to a conversation
passes a diff gate silently. Reproduction: an exchange cannot be declared on 0.92.0
(`triage/650/repro/run.txt`), so no diff reads one.

## Acceptance

- `cargo test -p ess-diff --locked --test exchanges` (new file) passes, with one case per pair of
  specifications differing only in an exchange, each asserting that `ess verify diff` reports an
  `exchange` change of this class: an added exchange, compatible; a message added to
  `initiator_sends`, compatible; a removed exchange, breaking; a message removed from a side,
  breaking; a message moved to the other side, breaking; a lowered `max_followers.count`,
  breaking; a changed `closes`, breaking.
- `cargo test -p ess-diff --locked` passes with no existing file under
  `crates/verify/ess-diff/tests/` changed (`git diff --stat` against the base lists only
  `tests/exchanges.rs` there): two specifications without exchanges diff as before.
- `docs/design/message-exchanges.md` names the `exchange` change kind and the class of each of the
  seven cases above.

## Scope

- cited: crates/verify/ess-diff/src/change.rs
- cited: crates/verify/ess-diff/src/compatibility.rs
- inferred: crates/verify/ess-diff/src/raw.rs
- inferred: crates/verify/ess-diff/tests/exchanges.rs (new)
- inferred: docs/design/message-exchanges.md (written by story:an-exchange-is-declared-in-the-specification; this story adds the change kind)

confidence: medium

Shared surfaces: no format row and no byte-pin row; it proposes its `CHANGELOG.md` sentence.

## Fit review

1. **Need.** A changed exchange is classified by the diff. The earlier body of
   `story:an-exchange-is-declared-in-the-specification` carried this line; split out so that story
   stays in two crates.
2. **Class: gap** (follows from the new construct).
3. **Already expressible?** No.
4. **Fit.** Reuses the diff's compatible/breaking classification; the skill's question 4 names
   `ess verify diff` as a target every construct handles or refuses.
5. **Second adopter.** Any adopter gating releases with `ess verify diff` who declares an exchange.
6. **Cost.** New change kinds; a diff format version only if a new kind needs one
   (`docs/design/generic-language-features.md`, § Migration and format consequences).
7. **Alternatives.** (a) Fold into the exchange story: three crates in one change. (b) Leave the
   diff blind: a target ignoring a construct is a red flag of the assessing skill. (c) Chosen: its
   own story after the exchange.

## Decisions

**accept as proposed**, part of ADR `messages-are-exchanged-independent-of-the-wire`
(accepted 2026-10-28).

## Source

github:beyond10x/ess#650
