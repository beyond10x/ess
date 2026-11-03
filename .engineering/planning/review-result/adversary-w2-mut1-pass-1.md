---
format: aep.planning-md/3
id: review-result:adversary-w2-mut1-pass-1
kind: review-result
status: active
title: 'W2 MUT1 adversary pass 1: grant classes, unmutated sites, payload error-swap and the scenario clock'
summary: NEEDS-CHANGE, 381 to 388 executed, 6 red; F1-F4 introduced to fix round 1, F5 to backlog, F6 decided, F7 fixed
relations:
- reviews: story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock
- reviews: story:the-mutation-audit-reports-the-sites-it-could-not-mutate
- reviews: story:error-swap-mutates-refusals-that-carry-a-payload
- reviews: story:the-mutation-audit-covers-actor-grants
revision: 1
---
unit: W2 MUT1 (story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock, story:the-mutation-audit-reports-the-sites-it-could-not-mutate, story:error-swap-mutates-refusals-that-carry-a-payload, story:the-mutation-audit-covers-actor-grants), beea98d674, ebed16afd6, 578fde2652 and ad383e41a5 merged as a48d077e02, reviewed in tree ess-w2-mut1 at ad383e41a5
verdict: NEEDS-CHANGE
cases: executed 381→388, red 6
origin: introduced 5 / pre-existing 1 / undecided 0
wrote-outside-worktree: the wave scratch directory mut1/adversary-scratch (diff, cargo logs)
needs-coordinator: finding 6 (#709 Acceptance) and finding 7 (spec-versions.md), both decided by the lead

**F1 (introduced, blocker, fix round 1).** A `sets-retarget` literal mutant on a field no view or
derivation reads cannot be told from the original, yet is scored `survived`
(`crates/verify/ess-conformance/src/mutate.rs:1869`, `literal_sites`), so the default audit of the
#711 toy exits 1, against "The default exit status is unchanged". Red case
`the_711_toy_full_audit_still_exits_0`.

**F2 (introduced, warning, fix round 1).** `--class transition-to` over the #711 toy refuses with
ESS-MUTATE-003 ("no site") and names the listed site nowhere (`mutate.rs:5608`, ess-cli
`finish_mutation_audit`); the base named it as a stillborn mutant. Red case
`transition_to_alone_over_the_711_toy_names_its_site`.

**F3 (introduced, warning, fix round 1).** `audited_grants` (`synthesize/grant.rs:226-231`) skips
grants on commands no `reached_by: network` component accepts and the system actor's grants
without listing them as unmutated sites; `examples/billing` (`Customer`) is one. Red cases
`billings_grant_is_a_mutant_or_listed`, `the_system_actors_grant_is_a_mutant_or_listed`.

**F4 (introduced, warning, fix round 1).** `grant-add` is documented as killed by
`grant/admitted/<actor>`, but where an actor with attributes holds the command synthesis skips the
rotation (`grant.rs:519`) and the mutant survives (`mutate.rs:157`, the guide, the design doc). Red
case `grant_add_is_killed_when_the_holder_carries_attributes`.

**F5 (pre-existing, note, out of scope).** In a single-error domain, a refusal without a payload
has no other error and is skipped silently (`mutate.rs:2296`; at base `mutate.rs:1742`), while its
payload-carrying twin is listed `no_compatible_error_alternative`. Filed as the backlog draft
story:an-error-swap-site-without-an-alternative-error-is-listed. Red case
`error_swap_lists_both_refusals_of_a_single_error_domain`.

**F6 (judgement, note).** The #709 Acceptance contradicts itself: bullet 1c drops the payload only
into a fieldless error and otherwise lists the site; bullet 2 wants both toy refusals live, and on
the toy every candidate error has different non-empty fields. The code drops the entry whenever the
fields differ, as the issue text asks. Lead's decision: the code stands; a Resolution line in the
story records bullet 2 and the issue text as the pinned acceptance.

**F7 (introduced, note, fixed in the release prepare).** `website/docs/reference/spec-versions.md`
had no paragraph for `ess-mutation-report/8` and `ess-mutation-manifest/7`; added in c0258f0d0d.

**Attacked, not broken.** Billing's three `no_admissible_state` texts name their transition, its
`to:` and ESS-ENTITY-011; both payload-carrying refusals of the #709 toy are killed by name and
neither mutant equals the original; a payload-less refusal keeps its id and verdict; grant-drop of
an actor's only grant is killed by `grant/denied`; a `/6` manifest carrying `unmutated_sites` or a
grant mutant is refused naming `/7`; a selection without grant classes keeps its baseline and ids;
the clocked runner differs from `Runner::for_suite` only in its wall clock; the lead's four ess-cli
re-pins agree with #711. Peak target/ 9G, lowest / 167G.
