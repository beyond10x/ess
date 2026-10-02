---
format: aep.planning-md/3
id: verification-report:consumer-backlog-20261002
kind: verification-report
status: draft
title: Consumer backlog reconciliation and delivery ledger
relations:
- verifies: task:consumer-backlog-20261002
revision: 10
---
## Intake

Observed 2026-10-02: 82 open issues; 161 nonterminal AEP stories, of which 87 have no structured GitHub reference. Inclusion in intake is not a completion claim. The source set must be refreshed before completion.

## GitHub issues

| Issue | Report | Current disposition and evidence |
|---|---|---|
| #385 | PLAN.md and a generated comment list input guards before when_related refusals, against the precedence the suite checks | Pending verification and fit review; no completion claim. |
| #379 | Go, web and clap targets still refuse existing_instance branches (follow-up to #310) | Pending verification and fit review; no completion claim. |
| #365 | ess-ui/1: no row filter on a read or collection (where: over row and page params) | Pending verification and fit review; no completion claim. |
| #364 | ess-ui/1: a column cannot show a field of a related view (label_from / lookup) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #363 | Aggregates: no conditional count or sum (count where state == Done) | Pending verification and fit review; no completion claim. |
| #362 | Synthesis: an aggregate grouped by lifecycle state alone is refused (ESS-SYNTH-016) | Open synthesis design question: state-only grouping cannot isolate rows under current shared-target assumption. Needs witnessed isolation or grouped-delta semantics; not fixed by #309. |
| #361 | Synthesis: a parameter or copied field on an aggregate group key is refused (ESS-SYNTH-017) | Pending verification and fit review; no completion claim. |
| #360 | Synthesis: a view parameter over a field copied from a related row is refused (ESS-SYNTH-005) | Pending verification and fit review; no completion claim. |
| #358 | ess-ui/1: metric cannot count or sum rows of its read | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #357 | ess-ui/1 React: charts have no series legend and x labels overflow | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #356 | ess-ui/1 TUI: record-tab forms and header actions are not rendered or drivable; ess ui test cannot address nested rows | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #355 | ess-ui/1: switch_to drops the current page params | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #354 | ess-ui/1: no live on composites in tabs or on header nodes, and a header title cannot read the record | Pending verification and fit review; no completion claim. |
| #353 | ess-ui/1: a literal string widget argument is evaluated as an expression (renders false) | PR369 and381 contain different fixes; preserve unique regression coverage and resolve literal semantics in consolidated UI candidate before closure. |
| #352 | ess-ui/1: graph_editor takes nodes and edges from one read; no separate node and edge views | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #351 | ess-ui/1: group_by orders groups by data order, not the enum or a declared order | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #348 | ess-ui/1: the TUI and React format integers differently (1840 vs 1,840), so a text assertion passes in only one | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #347 | Conformance suite: expect_not_granted cannot check unpublished events, since no step exposes the target's event log | Pending verification and fit review; no completion claim. |
| #346 | ess ui test: expect_command compares values as text, so 7500 and '7500' both pass | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #342 | synthesize: a when_subject refusal beside a deletes: branch makes the closed and invariant-after scenarios assert the deleted row | Locally fixed at 0bcabd5f1: three regressions red then green, 75 focused passes; independent source review found no concrete defect. Final grouped package verification/publication pending. |
| #330 | ess-ui/1: options cannot name an enum declared in the model | Pending verification and fit review; no completion claim. |
| #329 | ess-ui/1 TUI: dotted struct fields are sent flat and as text instead of nested and typed | PR345 source merged locally into carrier381 tree; combined verification/publication pending. OriginalPR remains open until preservation published. |
| #328 | ess-ui/1: a choice over a view takes its value from row.id; no way to name the value or label field | Partially delivered in 0.51.0 via PR #343; explicit value/label fields and standalone/filter choices remain open per PR body. |
| #327 | ess ui test: choose acts only on filter-bar choices, not on form choice fields | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #326 | ess-ui/1: an inline confirm has no testable node path, and a confirm overlay without does closes without running the action (React) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #325 | ess-ui/1 TUI: a burst of live refetch events re-reads once per event instead of once per batch | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #324 | ess-ui/1 React: overlay params are evaluated without the opener's row in scope | PR344 source merged locally into carrier381 tree; combined verification/publication pending. OriginalPR remains open until preservation published. |
| #323 | ess-ui/1 TUI: a form overlay submits only params.id, not its other params | Accepted and active in UI consolidation batch; source confirms id-only overlay parameter copy. Fresh red regression and fix pending source integration. |
| #322 | ess ui check does not resolve names inside pages (param types, form fields, bind keys, overlay params, columns) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #320 | ess-ui/1: rows are keyed only by live.match, so a section without a channel over a view keyed by another field gets empty row keys | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #319 | Code targets: generate behaviours for commands guarded by when_related | Pending verification and fit review; no completion claim. |
| #318 | Synthesized servers: generate an in-memory store and a server entry point for components reached by network | Pending verification and fit review; no completion claim. |
| #317 | No scenario re-creates an identity after deletes: removed it, so a lookup that finds removed rows passes | Accepted and active in synthesis batch after342; bounded create-delete-recreate witness implementation underway, no completion claim. |
| #316 | Generated Rust creation ignores an identity the payload takes from the input | Pending verification and fit review; no completion claim. |
| #314 | Go target: generate determined command behaviours, view queries, a store and a server main, as the Rust target does | Behavior/query/invariant portion queued in combined PR386; generated store/entry requirement remains in318. PR uses Refs, not automatic issue closure. |
| #312 | Suites assume an empty target per scenario without saying so, and never act on a row as a different caller than arranged it | Pending verification and fit review; no completion claim. |
| #311 | ess-ui/1 renderers cannot run against a live served component (fixtures only) | Queued in PRs #375/#377/#380; served view parameters in #386. Candidate failures tracked by task:consumer-server-gate-corrections-20261002. |
| #309 | Aggregate with two group keys filled from one input is refused for a move the source does not have | Locally fixed at c28e3bdae: five regressions red then green, 300 package lanes/2145 passes; independent read-only review found no concrete defect. Awaiting grouped publication. |
| #308 | A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer | Fit accepted with explicit subject-only structural observation design; retained-result constraint refusal stays separate. Scoped in AEP; implementation pending. |
| #307 | when_subject over a field copied from a related row at creation finds no candidate (ESS-SYNTH-003, then ESS-SYNTH-004) | Pending verification and fit review; no completion claim. |
| #305 | ess-ui/1: ess ui check accepts any Field.as value; the schema lists a closed set | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #304 | when_related through an Optional input: an absent reference takes the not-found branch instead of skipping the guard | Pending verification and fit review; no completion claim. |
| #303 | ess-ui/1: three document faults ess ui check reports nothing on (duplicate nav entry, unknown shell, shell without page outlet) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #300 | ess-ui/1: widget expansion is exponential in nesting depth; a valid document can hang ess ui check | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #299 | No read of a row selected by a filter (the instance with field == input.x) in a guard or in sets: | Pending verification and fit review; no completion claim. |
| #298 | A Boolean input is not treated as a closed domain | Fit accepted: bounded typed Boolean finite proof with default-semantics compatibility controls. Scoped in AEP; implementation pending. |
| #297 | Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected | Pending verification and fit review; no completion claim. |
| #296 | Retrofit: no way to declare intended behaviour that the implementation is known not to meet, and count it apart | Pending verification and fit review; no completion claim. |
| #295 | mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited | Pending verification and fit review; no completion claim. |
| #294 | mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001) | Pending verification and fit review; no completion claim. |
| #293 | Explorer excludes every command with an Optional input (and every command with an unknown_instance branch) | Pending verification and fit review; no completion claim. |
| #292 | check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2) | Pending verification and fit review; no completion claim. |
| #291 | conform run --target interpreted answers wrong_state for an unknown identity where synthesis expects the declared not-found refusal | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #290 | ess verify diff: no way to fail on a breaking change; a narrowing exits 0 | Pending verification and fit review; no completion claim. |
| #288 | An affects: filter over subject.<identity> validates but synthesis refuses it (ESS-SYNTH-001) | Fit accepted: preserve symbolic captured subject identity in affects witnesses. Scoped in AEP; implementation pending. |
| #286 | No read grant on a view: an actor's may: covers commands only | Pending verification and fit review; no completion claim. |
| #285 | {related:} follows only one required reference: no read through an Optional reference or across two references (ESS-COMMAND-002) | Pending verification and fit review; no completion claim. |
| #284 | ess ui check does not check that a page actor is granted the commands it binds | Pending verification and fit review; no completion claim. |
| #283 | A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004) | Pending verification and fit review; no completion claim. |
| #282 | ESS-COMMAND-004 refuses a when_related refusal beside a wrong_state outcome; no precedence is stated | Pending verification and fit review; no completion claim. |
| #281 | ess-ui/1: a section has no heading, and a page cannot leave out a section its kind contributes | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #279 | synthesize: a moving command with stored-field guards is taken to rewrite an aggregate group key (ESS-SYNTH-017), not reduced | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
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
| #212 | ess verify conform mutate does not emit sets-drop, outcome-order-flip or the ==/!= and ±1 guard-boundary arms | Pending verification and fit review; no completion claim. |
| #200 | Feature request: a search parameter on a view (a view parameter as the operand of contains / starts_with) | Pending verification and fit review; no completion claim. |
| #197 | Feature request: a refused command cannot declare the compensating change the service makes before answering | Pending verification and fit review; no completion claim. |
| #194 | Feature request: a binding cannot invoke only when an Optional path is present (ESS-BINDING-015 leaves no way to say 'skip') | Pending verification and fit review; no completion claim. |

## Nonterminal AEP stories

Source audit covers all 87 stories initially lacking structured issue references; 17 hidden references are now attached. Source corroboration is not test evidence. Stale candidates require full acceptance/release verification before closure; product work remains accounted for rather than silently excluded.

| Artifact | Current status | Disposition | Evidence / next verification |
|---|---|---|---|
| story:a-branch-may-clear-the-field-it-owns | draft | stale-state candidate; acceptance remains to verify | Story body:75 records grammar/validation/synthesis shipped in 0.26.0; public-guide acceptance remains open at :76-79. |
| story:a-browser-that-answered-http-once-is-still-a-slow-start | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-killed-childs-outcome-says-which-signal-ended-it | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-macro-invoked-twice-in-one-module-refuses-the-consumer-gate | active | parked by AGENTS consumer-accounting decision | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-marked-region-is-not-a-scan-of-what-runs | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-no-view-arranged-half-probes-the-row-through-the-command | draft | source-corroborated consumer defect/gap | crates/verify/ess-conformance/tests/adversary_arrangement_pass2.rs:551 explicitly asserts refusal mutation is not caught without a view. |
| story:a-refusal-records-the-document-it-was-read-from | draft | source-corroborated consumer defect/gap | crates/specify/ess-compiler/tests/locator_citations.rs:252 still ignores the duplicate-source provenance acceptance. |
| story:a-report-says-why-a-scenario-was-skipped | draft | consumer gap with compatibility/design work | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:a-skip-says-why-the-target-could-not-answer | active | stale-state candidate; acceptance remains to verify | CHANGELOG.md:2128-2141 records log reasons shipped but persisted report reasons absent; parent acceptance requires explicit split. |
| story:acceptance-runs-as-toolchain-scenarios | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:adopter-reviewed-delta | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:aggregate-views | active | stale-state candidate | Body cites #96; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:binding-delivery-at-most-once | proposed | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:browser-fixture-startup-deadline | active | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:change-fragment-upgrade-obligation | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:collection-quantifiers-witnessed-everywhere | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:collections-reach-their-upper-count-boundary | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#196 |
| story:concurrent-history-records-inputs | draft | consumer gap with compatibility/design work | crates/verify/ess-conformance/src/history.rs:195 Operation lacks input; story body:85 leaves wire compatibility unresolved. |
| story:consumer-accounting-baseline-never-extended | active | parked by AGENTS consumer-accounting decision | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:count-guards-above-one-are-synthesized | draft | stale-state candidate; acceptance remains to verify | crates/verify/ess-conformance/tests/stored_field_guards_adversary.rs:282,432,476 exercise threshold witnesses and bounds; witness.rs:1904 cites this story. |
| story:create-only-command-cannot-refuse | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:cross-runtime-verdict-equivalence | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:cross-system-relation-target | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:crosswalk-verb-external-names-held-to-declarations | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:deleting-a-scratch-tmpdir-breaks-sccache-for-every-other-agent | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:delivery-trust-fixture-race | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:diff-classifies-error-payload-sources | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#253 |
| story:empty-projection-is-refused-or-explained | active | stale-state candidate | Body cites #102; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
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
| story:feature-request-251 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#251 |
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
| story:feature-request-274 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#274 |
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
| story:feature-request-288 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#288 |
| story:feature-request-289 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#289 |
| story:feature-request-290 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#290 |
| story:feature-request-291 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#291 |
| story:feature-request-292 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#292 |
| story:feature-request-293 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#293 |
| story:feature-request-294 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#294 |
| story:feature-request-295 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#295 |
| story:feature-request-296 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#296 |
| story:feature-request-297 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#297 |
| story:feature-request-298 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#298 |
| story:feature-request-299 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#299 |
| story:feature-request-300 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#300 |
| story:feature-request-301 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#301 |
| story:feature-request-303 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#303 |
| story:feature-request-305 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#305 |
| story:feature-request-306 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#306 |
| story:feature-request-307 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#307 |
| story:feature-request-308 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#308 |
| story:feature-request-309 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#309 |
| story:feature-request-310 | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#310 |
| story:feature-request-312 | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#312 |
| story:field-sensitivity-class | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:format-rule-for-relaxations | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:go-and-typescript-read-current-suites | draft | source-corroborated consumer defect/gap | crates/verify/ess-conformance/src/go/mod.rs:362 caps generated Go and TypeScript suite admission at /27. |
| story:go-generated-behaviour | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#314 |
| story:go-normalization-pattern-semantics | active | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:go-numbers-compare-by-value | active | stale-state candidate | Body cites #101; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:held-state-has-one-operand | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:host-context-has-one-shape | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:integrate-source-driven-realizations | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-bindings-and-unmet-obligations | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-eventual-views | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-scenario-supplied-facts | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:interpreted-trust-gate | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:java-conformance-target | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:list-and-text-guards-are-synthesized | active | stale-state candidate | Body cites #94; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:mutation-audit-and-model-runner | active | stale-state candidate | Body cites #114; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:native-realization-ci | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:normalization-equality-eligibility | draft | product feature/design backlog; assess before scheduling | Story body:63-75 freezes formats 5/6; body:118-128 is design-only. Do not silently change current persisted equality. |
| story:optional-guards-mean-what-they-say | active | stale-state candidate | Body cites #93; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:outcome-decided-by-environment | draft | stale-state candidate; acceptance remains to verify | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:outcome-groups | active | stale-state candidate | Body cites #105; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:payload-fields-have-one-filling-rule | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:planning-store-carries-workstation-paths | active | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:predicate-reference-page | active | stale-state candidate | Body cites #92; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:primitive-canonical-serialization | active | partial parent; full acceptance remains | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:reader-conformance-over-refusals | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#191 |
| story:reader-true-refused-for-closed-readers | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:related-guard-behaviour | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#319 |
| story:related-guard-vocabulary-aligns | draft | product feature/design backlog; assess before scheduling | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:related-record-effects | draft | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#229 |
| story:related-via-optional-input | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#304 |
| story:related-via-stored-reference | active | Referenced issue / active candidate; verify against GitHub table | github:beyond10x/ess#304 |
| story:release-status-publication-state | draft | stale-state candidate; acceptance remains to verify | crates/edge/ess-xtask/src/main.rs:491,530,1497 requests isDraft, excludes drafts and tests malformed/missing/draft cases. |
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
| story:stored-field-guards | active | stale-state candidate | Body cites #75; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:string-length-guards | active | stale-state candidate | Body cites #104; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:string-newtype-declares-its-alphabet | active | stale-state candidate | Body cites #103; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:string-prefix-suffix-substring-operators | active | stale-state candidate | Body cites #95; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:suite-pins-transitions-updates-and-boundaries | active | stale-state candidate | Body cites #111; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
| story:the-browser-fixture-abandons-a-profile-per-start | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-design-page-is-held-to-the-fixture-it-describes | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-generated-go-runtime-is-gofmt-clean | draft | consumer report awaiting current reproduction | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-interpreter-executes-stored-field-guards | draft | source-corroborated consumer defect/gap | crates/verify/ess-conformance/src/interpret/execute.rs:652 returns NotInterpreted for stored-field guards. |
| story:the-lane-does-not-pin-a-count-that-its-own-bookkeeping-moves | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-metadata-guard-rejects-every-build-but-one | draft | parked by AGENTS consumer-accounting decision | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:the-published-schema-admits-the-name-aliases-the-parser-reads | draft | source-corroborated consumer defect/gap | crates/specify/ess-domain/tests/adversary_charset_pass1.rs:186 explicitly asserts schema rejection of an accepted binding id alias. |
| story:the-startup-clamp-does-not-outlive-the-startup | draft | stale-state candidate; acceptance remains to verify | crates/edge/ess-cli/tests/support/browser.rs:469 restores SESSION_TIMEOUT after upgrade. |
| story:the-startup-lock-does-not-cover-the-first-round-trip | draft | internal tooling/test-quality report; verify current impact | crates/edge/ess-cli/tests/support/browser.rs:372 drops startup guard before session.new at :381; acceptance allows either lock correction or accurately documented bound. |
| story:the-unread-tree-bullet-is-read-whole | draft | internal tooling/test-quality report; verify current impact | Existing story body is the source; read-only audit 2026-10-02. This classification is not completion or a new deferral. |
| story:typed-literals-in-sets-and-unknown-instances | active | stale-state candidate | Body cites #113; CHANGELOG.md:1408-1479 records 0.34.0 implementation. Verify whole acceptance and release before lifecycle closure. |
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
