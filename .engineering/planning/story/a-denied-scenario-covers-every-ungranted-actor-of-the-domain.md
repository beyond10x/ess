---
format: aep.planning-md/3
id: story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain
kind: story
status: implemented
title: A grant-denied scenario catches a target that admits an ungranted actor of the same domain
summary: The denied cell reads as the lowest-named actor across domains, so a same-domain leak passes (a2a wave 1, M6); defect.
tags:
- adopter-report
- defect
- ess-0.70.0
relations:
- serves: vision:O2
- depends_on: story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued
- depends_on: story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant
- decomposes: epic:adopter-gaps-synthesis
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/view_grant.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/view_grant_denied.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:54Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:55Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-03T02:29:29Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **M6** (a2a wave-1 gaps list, line 40): the `AllTasks` grant/read/denied scenario picks the
  lowest-named ungranted actor across all domains, so it reads as `a2a.discovery.Client` and never
  as `a2a.tasks.Client`. A target that wrongly admits `a2a.tasks.Client` still passes.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/view_grant.rs:185-189`: the denied actor is the
  first actor in `ir.actors()` order whose grant does not name the view (`.find(|actor|
  !actor.may_read(view))`), with no regard to the view's domain.
- `view_grant.rs:229-236` (`denied_reads`): the denied read is sent as that actor, and always as no
  actor.
- `crates/verify/ess-conformance/src/synthesize/grant.rs:9-11` documents the command rule (one
  `<command>/grant/denied`, "sent as the lowest-named such actor"). It is not changed here.

## Fit review

2026-10-22, assessed with `.agents/skills/assessing-external-requests/SKILL.md`.

1. **Need.** A denied read must exercise an actor a target is likely to admit by mistake: one of
   the view's own domain. The adopter proposed no syntax.
2. **Class: defect.** The rule picks an actor from an unrelated domain, so the scenario cannot catch
   the leak the adopter names.
3. **Already expressible?** No authored idiom picks the denied actor.
4. **Fit.** Two options. (a) One denied scenario per ungranted actor: needs the actor in
   `ScenarioId::ReadGrant`, which changes scenario identity, a format change. (b) Prefer an
   ungranted actor of the view's own domain, falling back to today's choice: no identity change,
   and the choice still does not move when an unrelated actor is declared.
5. **Second adopter.** Any system with a public discovery domain beside a private one, each with
   its own `Client` actor.
6. **Cost.** (b): no format version, no new diagnostic; suites change only where the view's domain
   has an ungranted actor other than the one chosen today.
7. **Decision.** (b), see Decisions.

## Decisions

2026-10-22: **accept, redesigned.** No change to `ScenarioId::ReadGrant`, no edit to `grant.rs:9-11`, no
format version.

Superseded on 2026-11-02 by wave W1 (`approval-record:ess-w1-wave`, commit 509854bee): the denied read is
sent as every ungranted actor of the view's own domain, in name order, then as no actor; where that
domain has none, as the lowest-named declared actor the grant does not name
(`crates/verify/ess-conformance/src/synthesize/view_grant.rs:187-202`). The 2026-10-22 limit (only the
lowest-named actor of the domain read, a second one unchecked) no longer holds. Per-actor scenarios
are still not filed: the actors are reads within the one `ReadGrant` scenario.

Depends on `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued` and
`story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant` (unit A2, shares the
grant-denied runner path); this story lands after them.

## Acceptance

Fixture, written by the test: domains `demo.public` and `demo.tasks`, actors
`demo.public.Client` and `demo.tasks.Client` neither granted the view `demo.tasks.AllTasks`, and a
granted `demo.tasks.Agent`.

- **The denied read prefers the view's domain.** Transition: synthesis of the view's denied read.
  Before: it is sent as `demo.public.Client`. After: as `demo.tasks.Client`. Test:
  `crates/verify/ess-conformance/tests/view_grant_denied.rs`, case
  `the_denied_read_prefers_an_actor_of_the_views_domain`. Failing before: the case reads the
  suite's sender and fails on the current tree.
- **A same-domain leak is caught.** Transition: a mutation target admits `demo.tasks.Client` to
  `AllTasks`. Before: the suite passes. After: the denied scenario fails. Same file, case
  `a_target_admitting_an_ungranted_actor_of_the_views_domain_fails`.
- **Fallback keeps bytes.** Transition: synthesis for a view whose domain has no ungranted actor.
  After: the denied sender and the suite are byte-identical to the current tree's. Same file, case
  `the_denied_read_falls_back_to_the_lowest_named_actor`; every existing case in
  `tests/view_grant_denied.rs` and `tests/view_grant_security.rs` keeps its verdict.
- **A2's case still passes.** `tests/grant_denied.rs` case
  `a_refused_send_keeps_the_earlier_read_token` stays green after this change.

## Schedule

Release 0.70.0 (defect, wave A unit A3, after unit A2). The release tag on the artifact marks it.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (read-grant synthesis) — cited.
- **Files:** `crates/verify/ess-conformance/src/synthesize/view_grant.rs:185-189` (denied-actor pick) and `:229-263` (`denied_reads`); `crates/verify/ess-conformance/tests/view_grant_denied.rs` (three new cases and a two-domain fixture) — cited.
- **Symbols:** `ResolvedActor.domain` (`crates/specify/ess-compiler/src/ir.rs:2674`), `ResolvedView.domain` (`ir.rs:2331`); the owner field is set at `crates/specify/ess-compiler/src/resolve.rs:2175-2177` — cited.
- **Documents:** `website/docs/guides/synthesize.md:429`, `docs/design/view-grants.md:47` and `website/docs/reference/spec-versions.md:625` state the lowest-named rule and go stale on this change — cited.
- **Confidence:** high — the story and the tree both put the defect at `view_grant.rs:185-189`.
- **Would collide with:** any unit editing `synthesize/view_grant.rs` (read-grant witness and module doc, lines 9-17) or `tests/view_grant_denied.rs` — cited.
- **Safety fact:** the denied actor reaches only `denied_reads` (`view_grant.rs:194`) and the `denied.is_none()` note (`:196`) — cited; byte identity for other models is unproven.
- **Size:** medium, about 150 lines; no suite format change (`one_time_response.rs:30,32`: ORDINARY 34, COVERAGE 35, reused at `view_grant.rs:24`; no new ScenarioId or step) — cited.
- **ess-conformance targets:** `view_grant_denied`, `view_grant_security`, `grant_denied` — inferred.
- **Note:** the Schedule line (0.70.0) is stale; the defect is still present on 0.83.0 — cited.

## Delivered

W1 (D394), 2026-11-02: a denied scenario covers every ungranted actor of the view's domain. Unit 509854bee, with review cases 7f3ecac9f, d5181ba7a and da9a26af1; merged 7835e2196 and ad36f5642.
