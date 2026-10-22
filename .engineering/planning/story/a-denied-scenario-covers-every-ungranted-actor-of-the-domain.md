---
format: aep.planning-md/3
id: story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain
kind: story
status: draft
title: A grant-denied scenario catches a target that admits an ungranted actor of the same domain
summary: The denied cell reads as the lowest-named actor across domains, so a same-domain leak passes (a2a wave 1, M6); defect.
tags:
- adopter-report
- defect
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued
- depends_on: story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/view_grant.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/view_grant_denied.rs
revision: 4
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

2026-10-22: **accept, redesigned.** The denied read is sent as the lowest-named ungranted actor of
the view's own domain; where that domain has none, as today's lowest-named ungranted actor. No
change to `ScenarioId::ReadGrant`, no edit to `grant.rs:9-11`, no format version. Per-actor denied
scenarios are declined for now (format change); a second ungranted actor of the same domain stays
unchecked, and that limit is recorded, not fixed.

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
