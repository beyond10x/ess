---
format: aep.planning-md/3
id: review-result:security-w1-syn1-pass-1
kind: review-result
status: active
title: 'W1 SYN1 security pass 1: grant/denied beside a ranked view and every ungranted actor'
summary: APPROVE, 226 to 232 executed, 0 red; one introduced note fixed in the store text
relations:
- reviews: story:grant-denied-is-synthesized-beside-a-ranked-view
- reviews: story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain
revision: 1
---
unit: W1 SYN1 (story:grant-denied-is-synthesized-beside-a-ranked-view, story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain), e5905e8e6 and 509854bee merged as 7835e2196, reviewed in tree ess-w1-syn1 at 509854bee
verdict: APPROVE
cases: executed 226→232, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory syn1/security-scratch (cargo log)
needs-coordinator: none

**Findings.**
- **F1** (note, introduced, inside the acceptance): the Decisions section of
  `story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain` (2026-10-22) still said only the
  lowest-named actor of the domain is read, while `synthesize/view_grant.rs:190-202` reads every
  ungranted actor of the view's domain. Fixed in the W1 store: the Decisions section now states the
  shipped rule (no new story, D398).

**Cases kept.** Six cases in `tests/adversary_w1_syn1_security.rs`, committed with the unit as 7f3ecac9f
(formatted d5181ba7a).

Relayed by the orchestrator (2026-11-02); the full report is in the pass's own output.
