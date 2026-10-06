---
format: aep.planning-md/3
id: story:feature-request-275
kind: story
status: implemented
title: The caller-swapped run draws fresh identity inputs
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#275
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_275_pass2.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T06:31:25Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-01T06:31:54Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:05Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":3,"review_outcome":3,"verification":1}}}
---
## Outcome

The caller-swapped run of a synthesized scenario draws fresh values for caller-supplied identity inputs, so a command with an `existing_instance` refusal is not sent the same identity twice while expecting success.

## Acceptance

- On a minimal specification with a caller-supplied identity input and an `existing_instance` refusal, every synthesized scenario passes against the reference target under both caller orders.
- No synthesized scenario sends one caller-supplied identity twice while expecting the accepting outcome the second time.

## Origin

beyond10x/ess#275, reported downstream on 0.48.0 with a reproducing patch; site `crates/verify/ess-conformance/src/synthesize/caller.rs`.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`. Need: the suite must not contradict `existing_instance`. Class: defect (synthesis only; no authored surface). Existing idiom: none needed.

## Decisions

- **accept as proposed** (coordinator, 2026-09-30).

## Scope

Derived 2026-10-01 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs.

- **Files:** `crates/verify/ess-conformance/src/synthesize/caller.rs` — cited: `synthesize` (:152) appends the swapped run, `rename` (:636) suffixes only `instance`/`instant` keys, so literal input values pass through unchanged
- **Also likely (read, not edited):** `synthesize.rs:8463` (`identity_inputs`); `synthesize/existence.rs` (`identity_at`, `Fresh`, `FRESH_SLOTS` :81) if made `pub(super)` — inferred
- **Tests:** a new regression test beside `tests/caller_values.rs` — inferred
- **Confidence:** high
- **Would collide with:** none within this epic (no sibling names `caller.rs` or `existence.rs`)
- **Safety fact:** the swapped copy is built only when `caller::uses(ir)` (`synthesize.rs:1514`), so models whose actors declare no attributes are untouched — unproven

## Bundle isolation compatibility correction — 2026-10-03

Candidate `ba4591de292f8b0db3b337ad359c04b8c7fea37a` corrects only the stale global identity-uniqueness assertion in `adversary_275_pass2.rs`. The accepted suite 34/35 contract starts each scenario in an empty logical namespace, so two independent scenarios may each use identities 1 and 2. The existing assertion also fails on pre-282 baseline `72167e08`; no production regression is inferred from that result. Within each scenario the fresh-identity requirement remains unchanged. The replacement preserves swapped/unswapped accounting and includes actual native execution plus an injected within-scenario identity collision that must fail with ESS-CF-OUTCOME. The complete target passes 8/8, strict focused Clippy passes, and independent review pass 1 approves with no findings. Review publication SHA256: `91fba5b06d80067c022942f083f2be5517878fd3995a9917b9e89665ddeeda75`. Broader bundle verification remains pending.
