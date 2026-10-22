---
format: aep.planning-md/3
id: story:an-authored-claim-may-compare-a-nested-field
kind: story
status: draft
title: An authored scenario may compare a nested field
summary: Nested-field comparison is refused with ESS-AUTHOR-014/013 (a2a wave 1, G6).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **G6** (a2a wave-1 gaps list, line 15): an authored scenario cannot compare a nested field (a
  member of a struct in a response, event or view row). Refused with ESS-AUTHOR-014 /
  ESS-AUTHOR-013.

## Where it lives

- `crates/verify/ess-conformance/src/authored.rs:1981` and `:1985`: ESS-AUTHOR-013 is
  `Cause::UndeclaredField`, ESS-AUTHOR-014 is `Cause::MissingField` (numbered by
  `impl Cause => u16` at `:1938`).
- The refusal sites for nested shapes are not traced (candidates in `authored.rs` raise both causes
  from `ShapeError` at `:4805-4810`).

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch. Check
first whether a partial (subset) claim on a struct value already expresses it.

## Acceptance (provisional)

- An authored claim may name a nested field by path; it passes when that member equals the literal
  and fails otherwise.
- A path naming no member is refused with ESS-AUTHOR-013 naming the full path.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1.
