---
format: aep.planning-md/3
id: verification-report:consumer-backlog-20261002
kind: verification-report
status: draft
title: Consumer backlog reconciliation and delivery ledger
relations:
- verifies: task:consumer-backlog-20261002
revision: 17
---
## Intake

Observed 2026-10-02: 82 open issues; 161 nonterminal AEP stories, of which 87 have no structured GitHub reference. Inclusion in intake is not a completion claim. The source set must be refreshed before completion.

## GitHub issues

| Issue | Report | Current disposition and evidence |
|---|---|---|
| #385 | PLAN.md and a generated comment list input guards before when_related refusals, against the precedence the suite checks | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #379 | Go, web and clap targets still refuse existing_instance branches (follow-up to #310) | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #365 | ess-ui/1: no row filter on a read or collection (where: over row and page params) | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #364 | ess-ui/1: a column cannot show a field of a related view (label_from / lookup) | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #363 | Aggregates: no conditional count or sum (count where state == Done) | Pending verification and fit review; no completion claim. |
| #362 | Synthesis: an aggregate grouped by lifecycle state alone is refused (ESS-SYNTH-016) | Open synthesis design question: state-only grouping cannot isolate rows under current shared-target assumption. Needs witnessed isolation or grouped-delta semantics; not fixed by #309. |
| #361 | Synthesis: a parameter or copied field on an aggregate group key is refused (ESS-SYNTH-017) | Pending verification and fit review; no completion claim. |
| #360 | Synthesis: a view parameter over a field copied from a related row is refused (ESS-SYNTH-005) | Copied-source creation/later arrangement and view exclusion implemented and independently reviewed with307. Honest target passes; wrong-row/ignored-parameter mutants fail. Integrated7474bb5c5; full package pending. |
| #358 | ess-ui/1: metric cannot count or sum rows of its read | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #357 | ess-ui/1 React: charts have no series legend and x labels overflow | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #356 | ess-ui/1 TUI: record-tab forms and header actions are not rendered or drivable; ess ui test cannot address nested rows | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #355 | ess-ui/1: switch_to drops the current page params | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #354 | ess-ui/1: no live on composites in tabs or on header nodes, and a header title cannot read the record | Pending verification and fit review; no completion claim. |
| #353 | ess-ui/1: a literal string widget argument is evaluated as an expression (renders false) | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #352 | ess-ui/1: graph_editor takes nodes and edges from one read; no separate node and edge views | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #351 | ess-ui/1: group_by orders groups by data order, not the enum or a declared order | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #348 | ess-ui/1: the TUI and React format integers differently (1840 vs 1,840), so a text assertion passes in only one | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #347 | Conformance suite: expect_not_granted cannot check unpublished events, since no step exposes the target's event log | PR387 documents existing before/after event-log observation. Private consumer runner remains unverified; keep issue open. |
| #346 | ess ui test: expect_command compares values as text, so 7500 and '7500' both pass | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #342 | synthesize: a when_subject refusal beside a deletes: branch makes the closed and invariant-after scenarios assert the deleted row | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #330 | ess-ui/1: options cannot name an enum declared in the model | Pending verification and fit review; no completion claim. |
| #329 | ess-ui/1 TUI: dotted struct fields are sent flat and as text instead of nested and typed | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #328 | ess-ui/1: a choice over a view takes its value from row.id; no way to name the value or label field | Partially delivered in 0.51.0 via PR #343; explicit value/label fields and standalone/filter choices remain open per PR body. |
| #327 | ess ui test: choose acts only on filter-bar choices, not on form choice fields | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #326 | ess-ui/1: an inline confirm has no testable node path, and a confirm overlay without does closes without running the action (React) | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #325 | ess-ui/1 TUI: a burst of live refetch events re-reads once per event instead of once per batch | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #324 | ess-ui/1 React: overlay params are evaluated without the opener's row in scope | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #323 | ess-ui/1 TUI: a form overlay submits only params.id, not its other params | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #322 | ess ui check does not resolve names inside pages (param types, form fields, bind keys, overlay params, columns) | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #320 | ess-ui/1: rows are keyed only by live.match, so a section without a channel over a view keyed by another field gets empty row keys | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #319 | Code targets: generate behaviours for commands guarded by when_related | Pending verification and fit review; no completion claim. |
| #318 | Synthesized servers: generate an in-memory store and a server entry point for components reached by network | Implemented and independently reviewed: 25 focused cases pass, including strict generated Rust/Go HTTP targets and legacy context compatibility. Integrated in next carrier7474bb5c5; full package and publication pending. |
| #317 | No scenario re-creates an identity after deletes: removed it, so a lookup that finds removed rows passes | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #316 | Generated Rust creation ignores an identity the payload takes from the input | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #314 | Go target: generate determined command behaviours, view queries, a store and a server main, as the Rust target does | Behavior/query/invariants merged through PR386 (fa08de5); generated store and server entry remain in #318. Keep issue open. |
| #312 | Suites assume an empty target per scenario without saying so, and never act on a row as a different caller than arranged it | Pending verification and fit review; no completion claim. |
| #311 | ess-ui/1 renderers cannot run against a live served component (fixtures only) | Served view parameters merged through PR386 and UI transport preserved in PR381. Complete live consumer verification remains; no blanket closure. |
| #309 | Aggregate with two group keys filled from one input is refused for a move the source does not have | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #308 | A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #307 | when_subject over a field copied from a related row at creation finds no candidate (ESS-SYNTH-003, then ESS-SYNTH-004) | Both Optional policies and named transitions implemented and independently reviewed with57 grouped tests and14 decisive mutants. Integrated in7474bb5c5 with360; full package and publication pending. |
| #305 | ess-ui/1: ess ui check accepts any Field.as value; the schema lists a closed set | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #304 | when_related through an Optional input: an absent reference takes the not-found branch instead of skipping the guard | Pending verification and fit review; no completion claim. |
| #303 | ess-ui/1: three document faults ess ui check reports nothing on (duplicate nav entry, unknown shell, shell without page outlet) | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #300 | ess-ui/1: widget expansion is exponential in nesting depth; a valid document can hang ess ui check | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #299 | No read of a row selected by a filter (the instance with field == input.x) in a guard or in sets: | Pending verification and fit review; no completion claim. |
| #298 | A Boolean input is not treated as a closed domain | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #297 | Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected | Pending verification and fit review; no completion claim. |
| #296 | Retrofit: no way to declare intended behaviour that the implementation is known not to meet, and count it apart | Pending verification and fit review; no completion claim. |
| #295 | mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited | Confirmed capability gap: sole-event drop makes ordinary outcomes invalid; source/test pin matches0.51.0. Deleting ExpectEvent would weaken the suite and is rejected. Compatible event-swap needs operator/payload/no-alternative design; see story fit review. |
| #294 | mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001) | Pending verification and fit review; no completion claim. |
| #293 | Explorer excludes every command with an Optional input (and every command with an unknown_instance branch) | Pending verification and fit review; no completion claim. |
| #292 | check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2) | Pending verification and fit review; no completion claim. |
| #291 | conform run --target interpreted answers wrong_state for an unknown identity where synthesis expects the declared not-found refusal | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #290 | ess verify diff: no way to fail on a breaking change; a narrowing exits 0 | Pending verification and fit review; no completion claim. |
| #288 | An affects: filter over subject.<identity> validates but synthesis refuses it (ESS-SYNTH-001) | Merged in PR387 at1ff305685; all15 checks passed onad45061626 and merged tree is identical. GitHub issue closed. Source on main; release after0.51.0 pending. |
| #286 | No read grant on a view: an actor's may: covers commands only | Pending verification and fit review; no completion claim. |
| #285 | {related:} follows only one required reference: no read through an Optional reference or across two references (ESS-COMMAND-002) | Pending verification and fit review; no completion claim. |
| #284 | ess ui check does not check that a page actor is granted the commands it binds | Pending verification and fit review; no completion claim. |
| #283 | A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004) | Pending verification and fit review; no completion claim. |
| #282 | ESS-COMMAND-004 refuses a when_related refusal beside a wrong_state outcome; no precedence is stated | Pending verification and fit review; no completion claim. |
| #281 | ess-ui/1: a section has no heading, and a page cannot leave out a section its kind contributes | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #279 | synthesize: a moving command with stored-field guards is taken to rewrite an aggregate group key (ESS-SYNTH-017), not reduced | Merged in PR381 at b4da64e, 15 required checks passed; GitHub issue closed. Source is on main; release after 0.51.0 remains pending. |
| #273 | authoring: an event expectation cannot compare an identity field with a captured instance | Pending verification and fit review; no completion claim. |
| #269 | bindings: per-refusal failure policy | Pending verification and fit review; no completion claim. |
| #268 | bindings: react to one outcome of a command, not only to an event | Pending verification and fit review; no completion claim. |
| #267 | ESS-SYNTH-010 refuses binding flow/delivery into a command with a wrong_state branch, and on-failure drop | Pending verification and fit review; no completion claim. |
| #266 | synthesize: scenario arrangement ignores bindings that move state | Pending verification and fit review; no completion claim. |
| #244 | Feature request: guards over elapsed time since a stored instant and over calendar windows | Pending verification and fit review; no completion claim. |
| #237 | Distinct list members and a count across records | Pending verification and fit review; no completion claim. |
| #236 | mutate --emit has no --component: a repository implementing one component cannot score mutants | Pending verification and fit review; no completion claim. |
| #233 | Value expressions: dotted input paths in sets:/payload:, field arithmetic, sibling-field comparison, byte length | Pending verification and fit review; no completion claim. |
| #231 | Entity Runtime lowering refuses constructs ess/15-16 validate: unknown_instance, existing_instance, {related:}, {increment}, {cleared}, alphabet:, text .count, now | Pending verification and fit review; no completion claim. |
| #229 | No guard on another entity's state for non-creating commands, and no effect on related records | Pending verification and fit review; no completion claim. |
| #228 | No way to declare a multi-field key unique within a scope (equality, one arranged row) | Pending verification and fit review; no completion claim. |
| #225 | Feature request: a guard comparing two identity-typed inputs (a self-edge check) | Pending verification and fit review; no completion claim. |
| #223 | Feature request: explorer strings reach .count boundaries and example: values | Pending verification and fit review; no completion claim. |
| #222 | Feature request: validate checks an authored scenario step's expected outcome against the command's guards | Pending verification and fit review; no completion claim. |
| #221 | Feature request: the explorer draws commands with existing_instance, subject_state or subject_predicate outcomes | Pending verification and fit review; no completion claim. |
| #212 | ess verify conform mutate does not emit sets-drop, outcome-order-flip or the ==/!= and ±1 guard-boundary arms | Existing adopted operator design remains unimplemented: sets-drop, precedence swap and additional guard arms. Closed MutantClass decoding requires format compatibility decision; nine current classes unchanged from0.51.0. See updated story audit. |
| #200 | Feature request: a search parameter on a view (a view parameter as the operand of contains / starts_with) | Pending verification and fit review; no completion claim. |
| #197 | Feature request: a refused command cannot declare the compensating change the service makes before answering | Pending verification and fit review; no completion claim. |
| #194 | Feature request: a binding cannot invoke only when an Optional path is present (ESS-BINDING-015 leaves no way to say 'skip') | Pending verification and fit review; no completion claim. |

## Nonterminal AEP stories

Source audit covers all 87 stories initially lacking structured issue references; 17 hidden references are now attached. Source corroboration is not test evidence. Stale candidates require full acceptance/release verification before closure; product work remains accounted for rather than silently excluded.

| Artifact | Current status | Disposition | Evidence / next verification |
|---|---|---|---|
| story:a-branch-may-clear-the-field-it-owns | implemented | verified delivered | Release0.51.0 guide now covers clearing and literal distinction; four domain cases and two-act synthesis assertion passed in retained f863ee run. Independent acceptance audit agrees complete. |
| story:a-browser-that-answered-http-once-is-still-a-slow-start | active | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-killed-childs-outcome-says-which-signal-ended-it | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-macro-invoked-twice-in-one-module-refuses-the-consumer-gate | active | parked by AGENTS consumer-accounting decision | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-marked-region-is-not-a-scan-of-what-runs | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-no-view-arranged-half-probes-the-row-through-the-command | draft | source-corroborated consumer defect/gap | crates/verify/ess-conformance/tests/adversary_arrangement_pass2.rs:551 explicitly asserts refusal mutation is not caught without a view. |
| story:a-refusal-records-the-document-it-was-read-from | draft | source-corroborated consumer defect/gap | crates/specify/ess-compiler/tests/locator_citations.rs:252 still ignores the duplicate-source provenance acceptance. |
| story:a-report-says-why-a-scenario-was-skipped | draft | consumer gap with compatibility/design work | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-skip-says-why-the-target-could-not-answer | active | stale-state candidate; acceptance remains to verify | CHANGELOG.md:2128-2141 records log reasons shipped but persisted report reasons absent; parent acceptance requires explicit split. |
| story:acceptance-runs-as-toolchain-scenarios | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:adopter-reviewed-delta | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:aggregate-views | implemented | stale-state candidate | Body cites #96; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:binding-delivery-at-most-once | proposed | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:browser-fixture-startup-deadline | active | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:change-fragment-upgrade-obligation | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:collection-quantifiers-witnessed-everywhere | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:collections-reach-their-upper-count-boundary | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#196 |
| story:concurrent-history-records-inputs | draft | consumer gap with compatibility/design work | crates/verify/ess-conformance/src/history.rs:195 Operation lacks input; story body:85 leaves wire compatibility unresolved. |
| story:consumer-accounting-baseline-never-extended | active | parked by AGENTS consumer-accounting decision | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:count-guards-above-one-are-synthesized | implemented | verified delivered | All three acceptance items met in0.51.0; fourteen stored-field adversary cases passed at f863ee, including bounded input/stored count witnesses and diagnostic. |
| story:create-only-command-cannot-refuse | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:cross-runtime-verdict-equivalence | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:cross-system-relation-target | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:crosswalk-verb-external-names-held-to-declarations | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:deleting-a-scratch-tmpdir-breaks-sccache-for-every-other-agent | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:delivery-trust-fixture-race | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:diff-classifies-error-payload-sources | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#253 |
| story:empty-projection-is-refused-or-explained | active | delivered; historical evidence qualification | Released explanation/strict-refusal design meets issue102. Four current CLI tests passed in browser-full-cli.log; original red-first chronology remains unlocated. |
| story:ess-ui-type-grammar-aligns | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:external-requests-are-assessed-before-adoption | active | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:feature-request-194 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#194 |
| story:feature-request-197 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#197 |
| story:feature-request-200 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#200 |
| story:feature-request-212 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#212 |
| story:feature-request-221 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#221 |
| story:feature-request-222 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#222 |
| story:feature-request-223 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#223 |
| story:feature-request-225 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#225 |
| story:feature-request-228 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#228 |
| story:feature-request-229 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#229 |
| story:feature-request-231 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#231 |
| story:feature-request-233 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#233 |
| story:feature-request-236 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#236 |
| story:feature-request-237 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#237 |
| story:feature-request-244 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#244 |
| story:feature-request-251 | implemented | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#251 |
| story:feature-request-257 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#257 |
| story:feature-request-265 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#265 |
| story:feature-request-266 | proposed | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#266 |
| story:feature-request-267 | proposed | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#267 |
| story:feature-request-268 | proposed | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#268 |
| story:feature-request-269 | proposed | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#269 |
| story:feature-request-270 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#270 |
| story:feature-request-271 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#271 |
| story:feature-request-272 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#272 |
| story:feature-request-273 | proposed | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#273 |
| story:feature-request-274 | implemented | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#274 |
| story:feature-request-275 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#275 |
| story:feature-request-276 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#276 |
| story:feature-request-278 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#278 |
| story:feature-request-279 | proposed | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#279 |
| story:feature-request-280 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#280 |
| story:feature-request-281 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#281 |
| story:feature-request-282 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#282 |
| story:feature-request-283 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#283 |
| story:feature-request-284 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#284 |
| story:feature-request-285 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#285 |
| story:feature-request-286 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#286 |
| story:feature-request-287 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#287 |
| story:feature-request-288 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#288 |
| story:feature-request-289 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#289 |
| story:feature-request-290 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#290 |
| story:feature-request-291 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#291 |
| story:feature-request-292 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#292 |
| story:feature-request-293 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#293 |
| story:feature-request-294 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#294 |
| story:feature-request-295 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#295 |
| story:feature-request-296 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#296 |
| story:feature-request-297 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#297 |
| story:feature-request-298 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#298 |
| story:feature-request-299 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#299 |
| story:feature-request-300 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#300 |
| story:feature-request-301 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#301 |
| story:feature-request-303 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#303 |
| story:feature-request-305 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#305 |
| story:feature-request-306 | implemented | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#306 |
| story:feature-request-307 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#307 |
| story:feature-request-308 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#308 |
| story:feature-request-309 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#309 |
| story:feature-request-310 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#310 |
| story:feature-request-312 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#312 |
| story:field-sensitivity-class | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:format-rule-for-relaxations | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:go-and-typescript-read-current-suites | draft | source-corroborated consumer defect/gap | crates/verify/ess-conformance/src/go/mod.rs:362 caps generated Go and TypeScript suite admission at /27. |
| story:go-generated-behaviour | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#314 |
| story:go-normalization-pattern-semantics | active | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:go-numbers-compare-by-value | active | delivered; historical evidence qualification | Nine primitive corpus tests passed at f863ee, including actual Go event/view number-carrier comparison. Files unchanged from0.51.0; historical red-first unlocated. |
| story:held-state-has-one-operand | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:host-context-has-one-shape | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:integrate-source-driven-realizations | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-bindings-and-unmet-obligations | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-eventual-views | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-scenario-supplied-facts | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-trust-gate | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:java-conformance-target | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:list-and-text-guards-are-synthesized | implemented | stale-state candidate | Body cites #94; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:mutation-audit-and-model-runner | active | stale-state candidate | Body cites #114; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:native-realization-ci | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:normalization-equality-eligibility | draft | product feature/design backlog; assess before scheduling | Story body:63-75 freezes formats 5/6; body:118-128 is design-only. Do not silently change current persisted equality. |
| story:optional-guards-mean-what-they-say | implemented | stale-state candidate | Body cites #93; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:outcome-decided-by-environment | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:outcome-groups | active | delivered; documentation/evidence qualification | Released expansion design matches105, with29 domain passes and exact-suite equivalence at f863ee. Compiler IR test exists; design header still says proposed, original red-first chronology unlocated. |
| story:payload-fields-have-one-filling-rule | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:planning-store-carries-workstation-paths | active | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:predicate-reference-page | implemented | stale-state candidate | Body cites #92; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:primitive-canonical-serialization | active | partial parent; full acceptance remains | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:reader-conformance-over-refusals | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#191 |
| story:reader-true-refused-for-closed-readers | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:related-guard-behaviour | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#319 |
| story:related-guard-vocabulary-aligns | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:related-record-effects | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#229 |
| story:related-via-optional-input | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#304 |
| story:related-via-stored-reference | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#304 |
| story:release-status-publication-state | implemented | stale-state candidate; acceptance remains to verify | crates/edge/ess-xtask/src/main.rs:491,530,1497 requests isDraft, excludes drafts and tests malformed/missing/draft cases. |
| story:report-carries-passed-failed-skipped | active | stale-state candidate | Body cites #110; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:review-stale-lines-corrected | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:rust-recorder-does-not-lose-a-creation | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:scrub-the-planning-store-or-say-why-not | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:served-committed-command-answers-its-outcome | draft | consumer gap with compatibility/design work | crates/edge/ess-cli/src/web/bridge.rs:346 converts pump failure after a command outcome to undelivered; exact path to reverify before implementation. |
| story:served-store-and-entry | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#318 |
| story:served-view-params | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#311 |
| story:shared-public-gates | active | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:source-pinned-data-normalization | active | partial parent; full acceptance remains | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:specification-declares-its-ess-release | active | stale-state candidate | Body cites #106; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:specify-upgrade-command | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:stored-field-guards | implemented | stale-state candidate | Body cites #75; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:string-length-guards | active | delivered; documentation/evidence qualification | Chosen Unicode-scalar .count semantics:25 domain and11 conformance tests plus Go Unicode corpus passed at f863ee. Design header still proposed; historical red-first unlocated. |
| story:string-newtype-declares-its-alphabet | active | delivered; documentation/evidence qualification | Alphabet/example admission and sendable constrained witnesses passed25 domain/11 conformance cases at f863ee. Released explicit code/runtime limits retained; stale design header and historical red-first unlocated. |
| story:string-prefix-suffix-substring-operators | active | delivered; acceptance/documentation qualification | Rust/Go/TypeScript and Entity Runtime implementation located; actual text-match/invariant executions passed at f863ee. Manifest uses entity-core tag0.24.1 with exact lock4746bd7 rather than literal rev acceptance; design incorrectly defers TypeScript. Historical red-first unlocated. |
| story:suite-pins-transitions-updates-and-boundaries | active | delivered; acceptance/evidence reconciliation | Thirteen declared-behavior tests passed at f863ee including faulty-target kills. Every source is exercised on distinct instances inside one scenario, unlike literal111 cardinality wording; delivery commit283b8dcad4 records choice. Historical red-first unlocated. |
| story:the-browser-fixture-abandons-a-profile-per-start | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-design-page-is-held-to-the-fixture-it-describes | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-generated-go-runtime-is-gofmt-clean | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-interpreter-executes-stored-field-guards | draft | source-corroborated consumer defect/gap | crates/verify/ess-conformance/src/interpret/execute.rs:652 returns NotInterpreted for stored-field guards. |
| story:the-lane-does-not-pin-a-count-that-its-own-bookkeeping-moves | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-metadata-guard-rejects-every-build-but-one | draft | parked by AGENTS consumer-accounting decision | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-published-schema-admits-the-name-aliases-the-parser-reads | draft | source-corroborated consumer defect/gap | crates/specify/ess-domain/tests/adversary_charset_pass1.rs:186 explicitly asserts schema rejection of an accepted binding id alias. |
| story:the-startup-clamp-does-not-outlive-the-startup | implemented | stale-state candidate; acceptance remains to verify | crates/edge/ess-cli/tests/support/browser.rs:469 restores SESSION_TIMEOUT after upgrade. |
| story:the-startup-lock-does-not-cover-the-first-round-trip | active | internal tooling/test-quality report; verify current impact | crates/edge/ess-cli/tests/support/browser.rs:372 drops startup guard before session.new at :381; acceptance allows either lock correction or accurately documented bound. |
| story:the-unread-tree-bullet-is-read-whole | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:typed-literals-in-sets-and-unknown-instances | active | delivered; documentation/evidence qualification | Fourteen literal and three unknown-instance cases passed at f863ee; chosen not-found policy documented. Old Decimal-never-admitted design text is stale after later support; historical red-first unlocated. |
| story:types-only-realizations | active | partial parent; full acceptance remains | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:typescript-conformance-target | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:ui-react-live-binding | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#311 |
| story:ui-spec-style-tokens | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:ui-tui-app-generator | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#311 |
| story:ui-tui-live-binding | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#311 |
| story:union-tag-inline-with-fields | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:validate-sees-what-synthesize-refuses | active | stale-state candidate | Body cites #112; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:web-bridge-answers-like-http | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:wrong-state-witness-unknown-and-own-stored-guards | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |

## Verified AEP resolutions

- optional-guards-mean-what-they-say / issue93 moved to implemented after exact acceptance mapping, inspected actual historical red/green logs and verified containment in public0.51.0. Original intake row reflects prior active status; current artifact is authoritative.
- list-and-text-guards-are-synthesized / issue94 moved to implemented after the same independent acceptance/release audit. This covers the four reported guards, not arbitrary list predicates.
- go-numbers-compare-by-value /101 is released; original red-first chronology has not been located. No missing implementation inferred from missing historical evidence.
- report-carries-passed-failed-skipped /110 is delivered through the documented report/2 design, preserving report/1. Exact requested report/1 field names did not ship; original red-first chronology unlocated.
- specification-declares-its-ess-release /106 is released with versioned manifests/output state and later exact-pin delegation. Original warning/refusal behavior is superseded where delegation applies; original red-first chronology unlocated.

Each source-backed disposition is recorded on its owning story. The latter three remain active for their process-evidence qualification, not because their released behavior needs another implementation or gate run.

## Current verified delivery 2026-10-02

This section supersedes earlier intake-row snapshots where state changed.

- Security policy blocker cleared by delivery of the unchanged approved gates-policy main to the ESS secret. Failed security jobs reran successfully. No policy rules or exceptions weakened.
- PR386 merged as fa08de5bd816b3cc46694ec4d19d20d13e128764 by the bot App; merge tree exactly equals tested f0b220099. Behavior/query portion of #314 shipped to main, while #318 store/entry remains open.
- PR381 published at 55061600bd2be2d5be71daae40a637f8d00ddb74, including current main and all absorbed UI heads. Common gate passed; full remote CI36992931851 is running. Local affected tests, exact ci-lint and site-build passed. #323 fixed red-to-green; #324/#329 and both #353 regression sets retained. PR377/344/345/368/369 closed only after their exact heads were verified in the published candidate; earlier375/380 closed with preservation proof. PR381 is the only remaining original open PR. No UI merge/release claim yet.
- #316 identity-source correction committed936b119fcbfde45d7700bfc8915dcf267531784e after independent review;25 focused passes, required/Optional cases red-to-green in Rust and Go. Final server-group package/projection checks remain due.
- #317 initial independent review found missing external control before recreation. Measured regression reproduced it; corrected source passes98 focused cases. Original reviewer and a second independent reviewer found no remaining concrete issue. Full synthesis-group package checks remain due.
- #288 accepted and activated for the next synthesis fix. #308/#298 remain accepted scoped work waiting sequential implementation.
- AEP stories aggregate-views (#96), stored-field-guards (#75), predicate-reference-page (#92) reconciled to implemented using recovered historical evidence and released-source verification; #92 cross-repository reference follow-up verified at exact agentplugins remote.
- #101/#102/#105/#106/#110 delivered behavior is source/release corroborated, but historical red-first evidence is incomplete; explicit qualifications retained in each story. Do not reimplement these delivered behaviors merely to clear stale state.

Full intake remains82 issues plus161 initially nonterminal stories. Counts are intake, not a completion total. Remaining unresolved rows require ongoing fit/design/implementation or evidence-backed disposition.

## Additional source reconciliation 2026-10-02

- #304's archived intake story was deliberately superseded, not completed or lost: commit169415193 introduced active stories related-via-optional-input and related-via-stored-reference under epic:ui-live-apps. Their ess/21 design and prerequisites287/282 remain the delivery authority. Do not revive the archived duplicate; preserve full accepted Optional/stored-reference scope.
- #347: actual seven-case runtime audit at55061600 passed, including Rust/Go/TypeScript late-refusal and event-log corruption cases. Existing expect_not_granted already uses ObserveEvents before and after refusal. Consumer answer-only checking does not implement that expectation; private consumer correctness is unverified. Full reconciliation recorded in story:feature-request-347.
- #365: accepted existing read-filter design now has canonical active story and a dedicated managed worktree based on55061600. Complete model/checker/both-renderer/parity work is assigned as one future PR, separate from running381.
- #328: legacy same-name form selection is delivered, but arbitrary value/label selection and model-derived default identity are still required. #330: inline/local-enum drift checking is delivered; direct model-enum lookup still requires a model-aware loading contract and remains open. #354: node live ownership and header record scope need a design; existing Header.live means channel lifecycle indicators. None of these partial behaviors closes the original request.

## Integrated UI and refreshed intake

PR381 merged as b4da64e38b770fe74103409fe1fef7ae6ca214f4 by the bot App; merge tree4ec76c386613762c7157d3908deb3addb136620d exactly matches tested55061600. Every reported check passed. Refreshed GitHub intake:58open issues, no new reports,24closed since82issue intake:364,358,357,356,355,353,352,351,348,346,329,327,326,325,324,323,322,320,305,303,300,291,281,279. GitHub closure is recorded separately from any remaining individual AEP acceptance reconciliation. No release beyond0.51.0 yet.

Next server group316/379/385 plus347guidance now has full373pass package evidence, corrected generated fixtures and projection/site validation; synchronization with published UI main and one new remote gate remain. Synthesis288 committed63e64f6cd after measured red/treatment and independent review;308 now active, followed by accepted298.365complete read-filter work continues in its isolated tree.

## Follow-up checkpoint after UI integration

- Canonical planning commit577092dc2 records exact308 review/commit,318scope correction,298activation,284source-syntax mismatch and290full direction-aware design scope. The imported308 report was sanitized through the AEP CLI after common checks correctly refused workstation-specific paths; retry passed. No hook/policy was relaxed.
- Conformance source b05007e49 adds reviewed constrained-newtype structural replay observation; five focused binaries43pass3fail baseline to46pass0fail0ignored treatment. Boolean finite-domain work298 is active next in the same batch; no full group check or publication yet.
- Server candidate482609 includes mainb4da and passed common checks, ci-lint and projections; final site npm11 root-admission refusal is recorded on task:consumer-fixture-adoption-20261002. It remains unpublished pending the toolchain decision. Earlier full synthesis373pass0fail1existingignored and site evidence remain attributed to their exact source state.
- Full accepted318implementation started in isolated tree ess-backlog-served-entry-20261002 at482609. No partial acceptance, release or durability claim.
- UI365 full affected packages are running. Coordinator early source review identified two potential new shared-read cache defects: staggered local refetch counters can reuse a fulfilled stale request, and an authorization change is absent from the request key. Implementor is establishing decisive regressions; no final review verdict or green delivery claim yet.
- New story:feature-request-360 records the full consumer need and a bounded, shared arrangement approach with accepted307. It is planned, not implemented; removing the view parameter is explicitly not closure.

## Server follow-up published; npm dependency admission resolved

Bot PR https://github.com/beyond10x/ess/pull/387 has exact head 4826099161bec53d2da65996958b97cb0c96165a and groups issues316,379,385, plus fixture ownership and issue347 documentation. Its current-main parent is b4da64e. The signed common receipt passed and remote security/privacy is already green; full correctness gate remains running. No merge or release claimed.

Exact task site-build passed with Node24.15.0/npm12.2.0 and unchanged website allow-git=root. Official npm12 fixes propagation of actual root dependency admission to pacote; local probes preserved transitive and allow-none refusals. The earlier proposed Node22/npm10 exception was not used and is no longer needed. Server log: target/backlog-input/482609-site-build-npm12.log. The frozen UI candidate also passed task site-build (target/ui-reads-evidence/43-site-build.log), before the next review corrections.

UI review now has two further source counterexamples under focused reproduction: filtered live refetch bypasses only_if, and Rust number-to-text differs from JavaScript at exponent thresholds. The owner is correcting these before publication. Add workspace Cargo.toml and crates/ui/ess-ui/Cargo.toml to the issue365 scope for ryu-js ECMAScript formatting; Cargo.lock was already scoped. No unrelated numeric semantics change is authorized.

Issue318 reachability testing also exposed a pre-existing zero-input binding emitted unused event parameter under strict Rust warnings in rust/system.rs. The entry/store story remains bounded; the observation needs its own backlog disposition after an exact reproduction is retained. Non-scalar identity ordering remains under investigation, not silently declared supported.

## Additional AEP reconciliation, 2026-10-02

Read-only independent source audit by server_corrections; own new test executions: 0. References below identify retained current validation evidence, not reconstructed historical red runs.

- **validate-sees-what-synthesize-refuses:** both issue112 admission/refusal behaviors are delivered. `crates/edge/ess-cli/src/main.rs:2271-2304` and six CLI regressions passed in the full browser CLI log (lines1635-1646); source matches0.51.0. Remaining gap is historical red-first evidence, not an observed implementation defect.
- **create-only-command-cannot-refuse:** ess/16 supplied-identity `existing_instance` error with no effects is delivered. Domain upsert15 pass, synthesis adversary4 pass/2 pre-existing ignored, honest fresh-caller control passes. The original consumer distinction (four states, refuse in only two) has not been replayed. Do not claim that consumer replay or retain the obsolete assertion that lookup semantics remain undecided.
- **outcome-decided-by-environment:** genuine design work remains: pusher/CIDR example, costs of all three alternatives, chosen rule and independent review. The existing guarded-external-outcomes design settles InjectFault eligibility, not this question. Keep open.
- **typescript-conformance-target:** implementation is largely delivered. Retained full CLI evidence has five ESM tests passing (lines1594-1603); typecheck/runtime4, Rust-to-TypeScript per-scenario29 and adversary7 tests pass. Explicit same-suite Go-to-TypeScript comparison of all four report counts remains unverified, and the design still defers it. Record an evidence gap, not a demonstrated runtime defect.
- **shared-public-gates:** source adoption is delivered. Fresh GitHub audit found secret scanning and push protection enabled and active main ruleset22765952 requiring common security, Gate, both macOS ownership checks and planning validation with strict base updates. Workflow pin95d64e differs from the original acceptance's literal Gates0.1.0 tag4317ac; version qualification needs reconciliation. Historic producer-asset/receipt/task-check/site evidence was not re-executed. Historical SKIP_CONSUMER_CHECKS guidance is superseded by current opt-in CONSUMER_CHECKS.

These findings distinguish shipped behavior, missing evidence and actual unfinished design. None alone authorizes an unsupported implemented transition.

# Remaining UI and consumer-runner backlog audit

Source inspected: ess-backlog-next-20261002 at 7474bb5c5. Canonical decisions: ess-consumer-backlog-20261002/.engineering/planning/story. Issue bodies: cached issues-current-refresh.json, numbers 354, 330, 328, 311 and 347. No remote calls; current remote closure/release state was not independently refreshed. Own test/build executions: 0. No source, AEP or remote writes. This report contains no adopter files or credentials.

The existing tests and retained logs below are evidence produced by earlier runs, not executions by this audit. All paths below are repository-relative unless a log location is given. The UI source diff from aabc378c8 to 7474bb5c5 is empty (`git diff --name-only ... -- crates/ui`), so the retained final UI lane covers the same UI bytes. This does not replace combined-candidate integration gates.

## Recommended ledger dispositions

| Issue | Disposition at inspected source | Concrete next action |
|---|---|---|
| #354 | Both requested capabilities remain open; design required | Decide nested subscription ownership and header record source/title semantics, preserving existing Header.live status display |
| #330 | Inline/local enum workaround and bounded drift checking delivered; direct model enum resolution absent | Approve a model-aware loading/resolution contract, then implement across checker and both generation/run paths |
| #328 | Same-name form-field projection delivered; general value/label and model identity defaults absent | Approve projection/default contract, then implement complete choice behavior across React/TUI and standalone/filter-bar uses |
| #311 | Accepted redesign materially implemented; several replacement AEP states appear stale | Reconcile five replacement stories using existing runtime evidence; do not represent original channel streaming request as implemented |
| #347 | ESS capability already exists; accepted guidance clarification present | Reconcile documentation delivery; keep private consumer adapter correctness externally unverified |

## #354: nested live readers and record-derived headers

Canonical feature-request-354 revision 1 is draft, explicitly retaining BOTH halves and requiring a design. This is not a title-only task.

Current facts:
- `crates/ui/ess-ui/src/model.rs:685` NodeCommon has name/state/visible/degrades/unmapped, no live. Section owns `live: Option<Live>` at :713. Header title remains `Option<String>` at :1410. Header.live at :1423 is a list of channels whose lifecycle is displayed, not a read-update policy.
- TUI polling iterates page.sections in `crates/ui/ess-ui-tui/src/app.rs:141-188`; live event handling selects sections at :1393. React section handling at `crates/ui/ess-ui-react/src/emit.rs:1799-1817` applies usePoll/useLive to section reads. Shared/filtering read machinery delivered with #365 does not make an arbitrary nested node a live subscriber.
- TUI `view.rs:365-380` prints the literal header title. React `emit.rs:2024` quotes that title and `templates/runtime/core.tsx.tmpl:488` displays it directly. Existing header live display is visible at core.tsx.tmpl:490-492.

Remaining decisions: choose the source record when a page has multiple reads; define nested reader subscription lifetime while tabs are inactive/unmounted; specify which effects are meaningful for record, metric and list readers; preserve the existing Header.live meaning with an unambiguous additive design. Also specify loading/empty/error title behavior. These are concrete contract decisions, not an external infrastructure blocker.

Falsifiable future probes: a tab-held record/list changes after its channel event in both renderers; switch tabs repeatedly and prove one subscription/no duplicate event application; inactive-tab policy follows the chosen contract; two records on a page prove the title uses the explicitly selected read; title changes on refresh and handles absent/error states; old literal titles and channel lifecycle badges remain unchanged. No such new cases were run here.

## #330: model enums in options

Canonical feature-request-330 revision 1 is draft. Existing fit accepts the need for direct lookup and calls for a bounded loading design. It does not accept the inline workaround as closure.

Delivered: local enum and literal options expand, and ordinary unbound form choices are checked against their command input enum. `crates/ui/ess-ui-check/src/model.rs:399-467` checks these, including form groups/tabs; :439 deliberately excludes explicitly bound fields. `tests/checks.rs:1491` model_enum_values checks missing/unknown variants. The final UI log records this case passing at line 271.

Still absent: `crates/ui/ess-ui/src/expand.rs:616-632` resolves options only from the UI document's types and reports "not an enum type of this document" for a model enum. `ess-ui-check/src/lib.rs:387` calls model-free load_str before Model.check. CLI binding also loads the UI first (`crates/edge/ess-cli/src/ui.rs:153-157`), so supplying --model cannot repair this early refusal. Direct model enum references in standalone choices are equally blocked.

Smallest coherent design: an explicit model-aware resolution phase/API used consistently by check, React generation, TUI run and generated TUI production. Preserve existing model-free load APIs; do not resolve unavailable information into empty options. Decide local-versus-model name collisions, fully qualified names/aliases, unknown/non-enum types, missing --model and emitted variant order. Avoid assuming that adding a lookup inside Model.check is sufficient: the loader already refused by then.

Future red-capable tests: options naming a qualified model enum in form AND standalone choice; deterministic variant order; local options without a model; unknown/non-enum references and missing model have named diagnostics; conflicting local/model names follow the decision; inline variants still detect drift. A generation test must execute the same resolved options that check admitted.

## #328: dynamic choice values and labels

Canonical feature-request-328 revision 1 is draft, but its decision accepts the complete need and awaits a concrete projection/default contract, not another product approval. Scope includes model/binding/schema/checker, both renderers, choice_value tests and reference documentation.

Delivered partial correction (ba1a1d31d, source present): a form field named repository_id takes row.repository_id before row.id. Current TUI `app.rs:2476-2513` uses receiving field, then id, then entire row; labels use label/name/value. React `templates/runtime/composites/choice.tsx.tmpl:31-36` uses valueKey, then id/value/null; labels use label/name/value. The UI filter changes preserve this behavior; they add no projection contract.

Existing tests deliberately choose a receiving field ALSO named repository_id: `crates/ui/ess-ui-react/tests/choice_value.rs:25-40,133,157`, TUI `tests/choice_value.rs:81`. Their passing 2 React and 1 TUI cases substantiate this partial correction, not the broader request.

Remaining: a receiving field named selected_repository still cannot select repository_id and show location; neither dynamic fallback uses reads.key; Choice has no dynamic value/label fields (`ess-ui/src/model.rs:1301-1317`); ViewRoute carries only path/params (`binding.rs:45-50`). Model identity inference is not carried to either renderer. Standalone/filter-bar calls cannot rely on the form field's name.

Important design boundary: a ResolvedView has source entity and projected fields, plus possible aggregation (`ess-compiler/src/ir.rs:1444-1486`), not an unconditional unique row-identity member. Infer a model default only when the source identity is actually projected and the view shape justifies it; decide the named refusal/fallback for aggregate or identity-omitting views. Do not fabricate a first-field identity. Decide explicit value projection versus reads.key precedence, field path versus row expression, null/missing projection behavior and fixture-only legacy fallback. Preserve typed values through actual command submission; stringifying an option for display is not permission to stringify its value.

Future probes: selected_repository receives integer 42 from repository_id while displaying location; explicit reads.key; model identity with no explicit key; explicit value and label; standalone/filter-bar; unknown/missing/null projections; aggregate/identity-omitting view; legacy same-name field unchanged. Run both renderers through selection and inspect submitted command/state, not just generated JSX. Include filtered/polled read changes because #365 shares these seams.

## #311: live served-component binding

feature-request-311 is archived revision 2 and superseded. The replacement states are ui-binding-contract implemented rev10; ui-react-live-binding active rev6; ui-tui-live-binding active rev6; ui-tui-app-generator active rev8; served-view-params active rev7. Do not infer unresolved implementation solely from those active states.

The accepted redesign explicitly chose model-derived routes, --base-url rather than endpoint, explicit authorization, no server stream, and polling or named refusal for no_live. It excluded server paging; TUI's http://-only restriction is explicit. Therefore lack of SSE/WebSocket remains a difference from the original request, but is not missing acceptance under the recorded redesign.

Delivered source:
- CLI routes --model through binding for TUI run and React/TUI generation (`crates/edge/ess-cli/src/ui.rs:92-157`). Binding is generated from the served route table, with checked parameters (`ess-ui-check/tests/binding.rs:126` and following named controls).
- React bound httpAdapter, explicit setAuthorization and per-component baseUrl exist at `templates/runtime/data.ts.tmpl:271,283,336`; TUI HttpAdapter construction and post are in `ess-ui-tui/src/http.rs:175,241`.
- Actual served refusal and read/command cases: React `tests/live_binding.rs:351,430,622,796`; TUI `tests/http_adapter.rs:212,333,371,397`. Polling/refusal policies are tested at TUI :488 and React adversary_live_binding_pass1.rs. The final UI lane reports React live_binding 11/0/0 and TUI http_adapter 5/0/0, including actual synthesized gatepass server calls.
- Generated Rust TUI executable checks are `ess-cli/tests/generate_ui_tui.rs:269,323,351`; retained cli-ui-tests.log reports 3/0/0, including its built crate reading from the synthesized server. ui_run_live reports 3/0/0 for missing model, ambiguous base URL and named HTTPS refusal.
- View params reach both actual servers in `ess-synth/tests/view_params_served.rs:426,431,454,542,697`; server-group-lanes/view_params_served.log reports 8/0/0, including missing-param refusal and unchanged parameter-free control.

Disposition: reconcile the replacement stories' accepted cases/evidence as delivered in source; no specific new implementation defect was established by this audit. Final combined validation/publication is coordinator-owned. Neither old active states nor the absence of a streaming transport justify inventing new work in this batch.

## #347: independent unpublished-event observation

Canonical feature-request-347 is active rev6. The accepted scope is guidance, not a new suite step or runtime fix, and expressly excludes claiming the private runner corrected.

Existing runtime: `ess-conformance/src/runner.rs:438-445` looks ahead to ExpectNotGranted and records counts before ExecuteCommand; :1418-1475 counts after and fails growth, including duplicate occurrences. `src/target.rs:212` exposes independent observe_events. The whole log for the requested event/correlation is the authority, not direct answer events or view snapshots.

Delivered guidance: `website/docs/guides/verify/runners.md:68-78` now explicitly specifies before-send lookahead, same-correlation post-refusal count, duplicate occurrences and unsupported observation. `author-scenarios.md:148-155` repeats the ordering and states no extra authored step. These changes are present through commit 17bfc38c0 in the current source.

Retained grant-log-audit.log records adversary_265_pass2 7/0/0: late-refusing surface; repeated earlier occurrence; Go/TypeScript direct publication on refusal; TypeScript log publication before refusal. This is actual existing-runtime evidence, not a reproduction of the consumer's reported 91/131 survivors.

External evidence still needed to resolve the consumer failure: its adapter implementation/version and a sanitized run showing ObserveEvents reads the real implementation log under the scenario correlation both before and after a denied command. Require an event-only mutation that publishes before refusing while returning no direct events, plus duplicate-occurrence mutation; both must fail, and unsupported observation must be non-passing. Without that evidence, mark consumer integration unverified, not ESS capability absent. Do not send any message to the consumer without authorization.

## Smallest next implementation group

Recommend one choice-resolution group: design #330 and #328 together, then implement #330's explicit model-aware load path followed by #328's projection/default propagation in one carrier. They share loading, model resolution, form/standalone choice paths, CLI generation and React/TUI tests; one package validation and one remote gate are appropriate after both stabilize. Decisions listed above must be recorded before implementation. This is a proposed grouping, not permission to silently select new authored syntax.

Keep #354 separate: it changes nested subscription lifecycle and header read ownership, not merely choice resolution. Keep #347 outside the implementation group: shipped runtime plus delivered guidance needs consumer evidence, not duplicate semantics. Handle #311 as evidence/state reconciliation, with any actual newly demonstrated defect separately scoped.

## Retained evidence locations

- UI lane: ess-backlog-ui-reads-20261002/target/ui-reads-evidence/58-final-packages.log. Relevant ranges: binding 222-231; model_enum_values 271; React choice 508-514; React live_binding 552-567; TUI choice 920-925; TUI http_adapter 952-961. All relative tree paths are under the managed ESS trees directory.
- CLI lane: batch-0-51/target/ui-consolidation/cli-ui-tests.log:135-142 and :165-172.
- Server lane: ess-backlog-servers-20261002/target/backlog-input/server-group-lanes/view_params_served.log; final-results.tsv records exit 0.
- Grant lane: batch-0-51/target/ui-consolidation/grant-log-audit.log, seven passing cases, no ignored cases.

Own test/build executions remain 0. No completion inferred from issue title, existing test name alone, or stale AEP lifecycle state.

# Synthesis and semantics backlog audit

Source: `ess-backlog-next-20261002` at `7474bb5c5d9eda9642342ea968a8873103778751`. Canonical planning: `ess-consumer-backlog-20261002/.engineering/planning/`. GitHub issue bodies and OPEN state were refreshed read-only for all six issues on 2026-10-02; their contents match the retained intake. Own builds/test executions: **0**. This is source/design reconciliation, not fresh reproduction or completion evidence. No source or planning changes were made. During this read-only pass the coordinator advanced next-tree to `2291c5adfa61baad92b80efb752a841a11710723`; the compared commit changes only Cargo.lock and generated billing/gatepass artifacts, so every cited implementation/test/design source remains identical to the pinned audit head.

| Issue | Evidence-backed disposition | Smallest coherent next work |
|---|---|---|
| [#363](https://github.com/beyond10x/ess/issues/363) | Conditional per-measure aggregates are absent from the authored and resolved types; no accepted design/story was found. | Fit/design one per-measure predicate extension, then its complete type/IR/evaluation/synthesis/projection change. Keep separate from witness-only aggregate repairs. |
| [#362](https://github.com/beyond10x/ess/issues/362) | State is understood as a group key, but state alone cannot scope an exact count under the documented shared-target model. The refusal is intentional under the current contract. | Decide empty-target qualification versus grouped delta semantics; coordinate that decision with #312. Do not remove ESS-SYNTH-016 alone. |
| [#361](https://github.com/beyond10x/ess/issues/361) | Parameters on group keys are explicitly rejected. Some related copied keys already work; generated related identities and other unchosen source shapes still need separate reductions. | One bounded aggregate key/parameter witness group, with independent-key and source-decoy controls. First classify each reported shape rather than reimplement #257 or claim #360 fixes aggregates. |
| [#312](https://github.com/beyond10x/ess/issues/312) | General identity reuse/isolation disclosure and some cross-caller histories remain open; existing ignored regressions name them. Some cross-caller cases already exist. | One suite isolation/caller-history contract group; choose provenance/report compatibility before adding fields. Restore the named ignored regressions as decisive controls when fixed. |
| [#304](https://github.com/beyond10x/ess/issues/304) | Archived intake was superseded by two accepted active stories, neither implemented at this head. Actual present behavior is validation refusal, not absent-reference execution. | Deliver the accepted ess/21 Optional-input story, then stored-reference story after #282; coordinate the shared reference seam with #285. No new fit review or revival of the archived duplicate is needed. |
| [#299](https://github.com/beyond10x/ess/issues/299) | Filter-selected guard sets have an adopted family-F design; reading one selected row's value into sets has no complete accepted selection/cardinality design. Neither surface exists in current code. | Guard half with family F B; settle zero/one/many and pre-state selection for the value-read half before implementation. An arbitrary first-row rule would invent behavior. |

## #363 — conditional measures

Current `Aggregate` contains only function, input field and `skip_absent` (`crates/specify/ess-domain/src/view.rs:356`). `RawFunction` accepts `Count(Empty)` and `Sum(String)` rather than a measure-local condition (`view.rs:472`). `ResolvedAggregate` likewise has no predicate (`crates/specify/ess-compiler/src/ir.rs:1515`). The evaluator receives an already chosen vector of values, not a row predicate (`crates/verify/ess-conformance/src/aggregate.rs:92,154`). A view-wide filter therefore cannot express total/done/escalated counts in different fields of one row. Existing aggregate design describes a shared source/filter and functions (`docs/design/aggregate-views.md:108,164,443`), not conditional measures.

Canonical search found only the generic ledger entry, no accepted #363 artifact/design. This is an authored capability gap, not a current synthesis defect. Minimum complete design must decide: measure-local predicate vocabulary and type environment, its composition with the view-wide filter, empty conditional count/sum semantics, Optional/skip_absent interaction, and source/IR format gating with unchanged old bytes. Choosing the requester's example spelling is a proposal, not an approved decision.

The coherent implementation crosses domain raw/validated aggregate, compiler IR/resolution, shared aggregate evaluation and generated behavior/runtime consumers, conformance arrangement, docs/schema/diff projections. Decisive cases: one row reporting total and two different conditional counts; conditional sum with qualifying/nonqualifying/absent inputs; an empty qualifying subset; mutants ignoring or swapping a measure's condition. Existing `aggregate_views.rs`, `aggregate_optional_fields.rs` and `aggregate_optional_fields_adversary*.rs` are compatibility seams, not evidence that #363 shipped.

## #362 — state-only grouping

`aggregate.rs:936` already constructs `Key::State` from the lifecycle states. Refusal occurs later at `:971-982`: no scopable key, no parameter scope, and no allowed ungrouped additive delta. The delta path is explicitly restricted to `aggregation.is_ungrouped()`. This is not a missing lifecycle-state parser or a tuple-selection bug.

The design explains why an exact aggregate needs rows belonging to this scenario (`docs/design/aggregate-views.md:443-455`): generated IDs can isolate observations without clearing all pre-existing rows, so another row in state Done changes the exact count. Existing `aggregate_views.rs:321` deliberately refuses an enum-only group with ESS-SYNTH-016. State is the same finite, unscopable class. #309's tuple repair preserves a separate independently scoped key; it does not solve this shape.

Concrete design choice required:

1. Declare and qualify an empty logical target for the affected scenario, then assert absolute counts per reached state; this intersects #312's suite/provenance/runner contract. A comment or silent reset assumption is insufficient.
2. Add before/after grouped additive observations, with declared semantics for absent groups, state transitions moving rows between buckets and concurrent/external writers. Existing ungrouped count/sum deltas (`scenario.rs:2616`, `aggregate-views.md:722-771`) are a precedent, not authorization for a grouped step. This covers additive measures only unless the acceptance/design explicitly expands it.

A state-only count is the minimal issue reduction. Test against pre-existing rows in the same states, a transition between groups, a wrong grouping target and an off-by-one target. Preserve explicit refusal for shapes the chosen observation cannot justify. The current canonical ledger correctly calls this a design question; no dedicated accepted #362 story was found.

## #361 — parameters and copied group keys

There are at least two independent remaining shapes:

- A declared parameter can scope an aggregate only through one top-level equality over a field **not** in `group_by` (`aggregate.rs:877-921`, especially `:894`). Thus direct `group_by: [objective_id]` plus `objective_id == param.objective_id` is rejected even when the field is otherwise a valid scoped key. Existing positive parameter coverage is an ungrouped view, `aggregate_views.rs:449`, while disjunctive parameter use is intentionally refused at `:477`.
- Related copied keys are not wholly missing. `related_key` supports a reference read when some source creator writes the copied field from input (`aggregate.rs:291-333`), and planning records/scoping use that support (`:783-831`). `aggregate_related_key.rs:125` and `:378` cover honest copied keys and wrong-key mutants; `:398` covers an absent copied Optional key. `adversary_257_pass1.rs:382,431,468` cover owner-link, optional and mixed keys. But a related row's generated identity is not a field set from input, and `related_key` rejects that route at `:322-330`. Chained reads and copied aggregate inputs remain explicitly refused (`adversary_257_pass1.rs:450,501`).

The issue names both parameters and copied `goal` fields but does not include a full admitted model for each affected aggregate. The evidence blocker is therefore exact shape classification for that second half, not generic pending work. Required reductions: direct key+parameter; copied ordinary String key+parameter; captured/generated related identity key; a pre-existing supported copied scalar key without a parameter as a compatibility control. Preserve captured source identity instead of replacing it with a made-up literal or inferred ownership edge.

Recommended bounded group: key/parameter planning, typed binding and observation in `synthesize/aggregate.rs`, with the existing aggregate related-key and parameter fixtures. The ordinary row-view repair #360 uses a different planner and does not bypass the aggregate parameter restriction. Require a queried group with several rows, nonmatching groups, each key varied independently, source rows before/after the referenced row, and mutants ignoring the parameter, grouping by the reference rather than its copied value, or reading another source. #309's independent-tuple controls and #257/#272 controls remain required. No authored syntax change appears necessary for the direct-key shape; extend the aggregate design's current restriction explicitly before claiming the broader copied-identity shape is accepted.

## #312 — isolation disclosure and cross-caller histories

Canonical `story:feature-request-312` is draft revision 1 with fit review pending, not an accepted implementation design. The generic isolation contract exists: every scenario gets an isolated logical context and targets may achieve it through resets, namespaces or generated IDs (`docs/design/ess-closed-loop-execution-conformance-design-v0.1.md:383-406`; `target.rs:13,179`). That does **not** record that a particular suite needs an empty target. `SuiteProvenance` has format/system/version/digests/component only (`scenario.rs:367-401`); `ScenarioContext` carries scenario/correlation only (`target.rs:482-496`). No empty-target requirement was found in these carriers.

The code does not literally never vary callers: `synthesize/caller.rs:1-26` already runs assigned caller values and swapped histories, and `:171`/`:601` have a singleton-specific act-on-the-same-row path. `granted_actors` selects an authorized actor deterministically (`synthesize.rs:11606`). So the issue must not be described as an absent caller subsystem.

Concrete retained gaps:

- `adversary_287_pass1.rs:526-552`: ignored Uuid controls for a per-caller store and a correct shared target, both explicitly linked to #312.
- `adversary_287_pass1.rs:636-659`: ignored empty-target disclosure control.
- `adversary_287_pass2.rs:480-516`: ignored second-caller reinstall controls for singleton and Uuid cases. The comment identifies the command-level `creates_it` exclusion; `caller.rs:613-617` still takes that path.

These are existing claimed regressions, not fresh red observations in this audit. Smallest complete group must cover both requested axes: make caller-supplied identities fresh across scenarios where their domain permits it, and explicitly carry/qualify an isolation requirement where it does not; arrange and act on the **same** row as two distinguishable authorized callers, including existing-instance branches. Preserve grant enforcement and caller-dependent semantics. Decide how the runner reports unsupported isolation and how old suites retain meaning before adding provenance/report fields. Merely switching actors without distinguishable credentials, or swapping an entire scenario so every row is still created and read by the same caller, does not prove shared-state behavior.

This design can establish the isolation premise needed by #362. It should not automatically change all aggregate assertions or assert that existing shared-target semantics were already an empty-target requirement.

## #304 — accepted replacement work, not a lost archived issue

Canonical intake `feature-request-304` is archived revision 2. It is superseded by active `related-via-optional-input` revision 8 and `related-via-stored-reference` revision 8. Their fit review explicitly corrected the reported failure: Optional input via was rejected at validation on 0.48/0.49 rather than reaching a not-found branch. Current source agrees: `related_guard.rs:500-513` permits only primitive/named input types and asks for a required identity. Parsing accepts only `input.<field>` (`:100-125`), not a stored reference. These are supported-gap refusals, not hidden runtime acceptance.

Accepted design already decides both forms in ess/21: absent Optional reference reads no row and selects no related branch; remaining selection must reach exactly one branch. Present-reference behavior remains unchanged. The stored-reference variant follows addressed existence/held state and precedes accepting branches via #282's new precedence step. Creation/no-subject restrictions remain. Generated Rust/Go/Web/Clap behavior stays an obligation until the separate `related-guard-behaviour` story; Entity Runtime keeps its named refusal.

Neither accepted suite is implemented at the audited head: no `issue_304_*` or stored-reference acceptance tests were found under crates, and `interpret/execute.rs:524-535` still declines stored via and missing/non-text input. `RelatedVia` remains only Subject(String)/Input(String) with required-reference semantics (`related_value.rs:15-31,66-69`).

Next group should follow the existing accepted sequence, not invent another one: Optional input admission/compiler/synthesis/interpreter/docs first, stored reference after #282, shared with #285's reference type work where appropriate. Reconcile #287's delivered singleton dependency from source/evidence rather than treating its stale active state as a new code blocker. #282 is still a draft story with an accepted redesigned decision, and its precedence step remains real prerequisite work. The source format bundle policy is recorded in `.engineering/waves/downstream-gaps.md:40-46`; do not silently ship only half a coordinated ess/21 contract.

Exact accepted cases are already named in the two stories: absent/present/missing and dishonest-target mutants; old ess/20 IR bytes; absent-exhaustiveness refusal; stored blocker not Done/Done, wrong referenced-row/subject-row mutants, dangling reference witnessed or explicitly noted unreachable. No fresh design blocker is needed for these accepted semantics.

## #299 — accepted row-set guard idiom, unresolved single-row value selection

Current guard raw input is identity-via-only (`related_guard.rs:65,100-125,500-526`). Related values use a Subject/Input reference field, one hop, rather than a filter (`related_value.rs:1-31`). Neither current surface can select “the previous Session with the same agent and campaign” without an explicit identity link.

Canonical #299 is draft revision 1, fit review pending. However the guard half already has a coordinator-adopted design under family F B: `when_related: {entity, where, exists | count | forall}` over bounded row sets, reusing set-effect filter vocabulary and scoped arrangement (canonical `feature-request-237`, Fit questions 4/6 and final Decisions; related #228/#233). It is part of the ess/21 bundle. That is an accepted related design, not an implemented guard or a complete design for a value read.

The remaining value-side decision is concrete: a filter can match zero, one or many rows. The issue's “current/previous” is not defined by the Session model given—new generated identity per Login does not identify which of several matching sessions is current. Decide whether the model guarantees exactly one, declares deterministic ordering/selection, or explicitly names the superseded row; define the zero-match default, multiple-match refusal, and pre-command snapshot (so the newly created session cannot read itself). Copying one arbitrary/first row or assuming uniqueness from equal agent/campaign fields would invent semantics.

Smallest coherent delivery: guard half within family F B, then a separately accepted value-read extension reusing that typed selection after cardinality and snapshot rules are settled. #285's Optional/chained identity-reference design is adjacent but does not authorize filtered lookup. Tests need zero/one/two matching predecessors, an unrelated decoy, matching agent with different campaign, preservation of the old counter, and a first/last-row mutant. Until that value-side decision is recorded, #299 cannot be closed merely because exists/count guards arrive.

## Suggested sequencing

1. Prepare bounded #361 key/parameter reductions and fit scope; it does not need a new authored construct for its narrow parameter case.
2. Deliver already accepted #304 replacement stories in the coordinated reference/precedence bundle, preserving their documented target obligations.
3. Decide #312 isolation reporting and complete cross-caller histories; use that decision to resolve #362's exact-count premise, or explicitly choose grouped additive deltas for #362 instead.
4. Complete #363 measure-local predicate design and #299 single-row value selection design; retain family F B as the already selected guard idiom.

These are specific ready-to-scope, accepted-to-implement, and decision-required dispositions. None of these six issues is claimed complete by this audit. No ignored regression was run or changed.

## PR387 verified merge

Merged2026-10-02T13:27:46Z by app/b10x-bot as1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036. Tree0c8693cea107f7ffef46319f70d27e1eb87a2917 equals tested headad45061626; first parentb4da64e is its ancestor. All15 reported checks passed. Refreshed GitHub intake now has49 open issues; all ten closing references are closed. Three browser-startup stories are also implemented. The private consumer adapter for347 remains unverified. No new release is claimed.
