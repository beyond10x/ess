---
format: aep.planning-md/3
id: story:format-rule-for-relaxations
kind: story
status: draft
title: A relaxation says which format admits it
tags:
- defect
- severity-low
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: docs/design/review-external-requests-2026-09.md
revision: 6
---
## Outcome

SV (`website/docs/reference/spec-versions.md`) states when a rule that ESS relaxes needs a format
version, so an older build does not refuse a newer document with a misleading diagnostic (review
S12).

## Problem

Still true on main for 0.92.0: the section "When the number moves" (SV:15-26) says a version moves
when "meaning, identity, references, canonicalization, names, or the persisted envelope changes"
and says nothing about a relaxation, a refusal lifted so that a document an older build refuses
becomes valid. No page under `website/docs/`, `docs/design/` or `AGENTS.md` states the rule.

The practice has moved since the 2026-09 review without the rule being written down:

- Early relaxations shipped without a format (closed issues #217, #227, #230); #204 was gated.
- Since then the `ess/` history in SV gates most relaxations and names the version in the old
  diagnostic: 18 rows say "below `ess/N` ... its hint naming `ess/N`" (`ess/30`, for example).
- `ess-inputs/2` `requires: ess X.Y.Z` already makes an older build name the release:
  `ess specify validate` with `requires: ess 0.99.0` on 0.92.0 exits 1 with "requires ess 0.99.0 and
  this is ess 0.92.0, which is older; install the required release ...". It reaches only documents
  that pin a release.

## Acceptance

- SV "When the number moves" states the rule for relaxations: when a relaxation takes a format
  version (the newer document is otherwise read with a different meaning by an older build, or a
  refusal's hint must name the version) and when it does not, citing #204 (gated) and
  #217/#227/#230 (not gated) and one `ess/N` row that names its version in the hint.
- The same page states what an older build does with a document that needs a relaxation: with a
  gated relaxation the old diagnostic names `ess/N`; with a `requires:` pin it names the release;
  otherwise it gives the old diagnostic, and the page says so.
- `AGENTS.md` "Determinism and formats" points to that rule in one line.
- `cargo xtask docs` and `cargo xtask format-history --check` pass with the page changed.

## Scope

- cited: website/docs/reference/spec-versions.md
- cited: AGENTS.md
- cited: docs/design/review-external-requests-2026-09.md
- inferred: crates/specify/ess-domain/src/system.rs (FORMAT_HISTORY, if the rule is generated with
  the table)

confidence: high

## Fit review

Not an adopter request: a finding of the repository's own review of external requests (S12). The
four issues it cites are closed and are evidence, not requests. Class: documentation gap. No
language surface, no format, no code beyond the generated table.

## Decisions

None yet: whether the rule is "a relaxation is gated when an older build's refusal would mislead"
(the current practice) or "every relaxation is gated".

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (format-rule-for-relaxations). Drafted, not scheduled; a fit review precedes dispatch.
