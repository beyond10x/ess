---
format: aep.planning-md/3
id: story:feature-request-280
kind: story
status: proposed
title: An enum-and-presence input guard is honoured by synthesis
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#280
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: docs/design/input-guard-overlap-precedence.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T06:31:26Z", actor: "human:timo", revision: 7}
---
## Outcome

An input guard that combines an enum value with an optional input's presence is honoured by synthesis, and no synthesized accepting step ever satisfies an earlier refusal's guard.

## Acceptance

- On the minimal specification attached to #280, `SetQuota/outcome/set` sends only inputs no refusal guard selects; `provider-not-allowed` and `provider-missing` are witnessed.
- A synthesis-wide check refuses to emit an accepting step whose input satisfies an earlier refusal's guard, naming both outcomes; a test runs it over every fixture and example.
- A reference target that honours the guards passes the suite.

## Origin

beyond10x/ess#280, reported downstream on 0.48.0 and reproduced by the coordinator.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect (synthesis emits a scenario contradicting the specification, with no refusal). No authored surface.

## Decisions

- **accept** (coordinator, 2026-09-30), with the synthesis-wide guard so the class fails loudly.

## Scope

Derived 2026-10-01 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs.

- **Files:** `crates/verify/ess-conformance/src/synthesize.rs` — cited: `reach` :4488, `plain_guards` :4921, `admits_plain` :4974, `decides` :5026, `searched_guards` :4672, `sibling_refusals` :4736; the default branch's input passes through `arranged` :2486 → `existence::fresh_created` → `freshened` :6585 / `distinguished` :6671 after it is chosen. The synthesis-wide check fits in `synthesize_plain` :1534 with a new `RefusalCause` (:472) — inferred
- **Also likely:** `src/witness.rs` (`search` :494, `omissions` :1364), `src/input.rs` (`flatten` :73) — inferred
- **Reference target:** `src/interpret/execute.rs` `select` :390 already answers the first matching refusal — cited, expected unchanged
- **Confidence:** medium — the code read did not locate the defect; the coordinator reproduced the wrong scenario on 0.48.0 (#280), so the implementor starts from a red run
- **Would collide with:** 265, 266, 267, 268, 269, 273 on `synthesize.rs`
- **Safety fact:** the check only refuses and adds no scenario; the interpreter already answers the first matching refusal, so every suite it passes today stays green — unproven
