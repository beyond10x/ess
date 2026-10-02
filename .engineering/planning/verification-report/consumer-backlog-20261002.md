---
format: aep.planning-md/3
id: verification-report:consumer-backlog-20261002
kind: verification-report
status: draft
title: Consumer backlog reconciliation and delivery ledger
relations:
- verifies: task:consumer-backlog-20261002
revision: 2
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
| #353 | ess-ui/1: a literal string widget argument is evaluated as an expression (renders false) | Queued candidate claims closure: PR #381, PR #369. Not shipped; preserve issue until accepted evidence. |
| #352 | ess-ui/1: graph_editor takes nodes and edges from one read; no separate node and edge views | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #351 | ess-ui/1: group_by orders groups by data order, not the enum or a declared order | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #348 | ess-ui/1: the TUI and React format integers differently (1840 vs 1,840), so a text assertion passes in only one | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #347 | Conformance suite: expect_not_granted cannot check unpublished events, since no step exposes the target's event log | Pending verification and fit review; no completion claim. |
| #346 | ess ui test: expect_command compares values as text, so 7500 and '7500' both pass | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #342 | synthesize: a when_subject refusal beside a deletes: branch makes the closed and invariant-after scenarios assert the deleted row | Pending verification and fit review; no completion claim. |
| #330 | ess-ui/1: options cannot name an enum declared in the model | Pending verification and fit review; no completion claim. |
| #329 | ess-ui/1 TUI: dotted struct fields are sent flat and as text instead of nested and typed | Queued candidate claims closure: PR #345. Not shipped; preserve issue until accepted evidence. |
| #328 | ess-ui/1: a choice over a view takes its value from row.id; no way to name the value or label field | Partially delivered in 0.51.0 via PR #343; explicit value/label fields and standalone/filter choices remain open per PR body. |
| #327 | ess ui test: choose acts only on filter-bar choices, not on form choice fields | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #326 | ess-ui/1: an inline confirm has no testable node path, and a confirm overlay without does closes without running the action (React) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #325 | ess-ui/1 TUI: a burst of live refetch events re-reads once per event instead of once per batch | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #324 | ess-ui/1 React: overlay params are evaluated without the opener's row in scope | Queued candidate claims closure: PR #344. Not shipped; preserve issue until accepted evidence. |
| #323 | ess-ui/1 TUI: a form overlay submits only params.id, not its other params | Pending verification and fit review; no completion claim. |
| #322 | ess ui check does not resolve names inside pages (param types, form fields, bind keys, overlay params, columns) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #320 | ess-ui/1: rows are keyed only by live.match, so a section without a channel over a view keyed by another field gets empty row keys | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #319 | Code targets: generate behaviours for commands guarded by when_related | Pending verification and fit review; no completion claim. |
| #318 | Synthesized servers: generate an in-memory store and a server entry point for components reached by network | Pending verification and fit review; no completion claim. |
| #317 | No scenario re-creates an identity after deletes: removed it, so a lookup that finds removed rows passes | Pending verification and fit review; no completion claim. |
| #316 | Generated Rust creation ignores an identity the payload takes from the input | Pending verification and fit review; no completion claim. |
| #314 | Go target: generate determined command behaviours, view queries, a store and a server main, as the Rust target does | Queued candidate claims closure: PR #384. Not shipped; preserve issue until accepted evidence. |
| #312 | Suites assume an empty target per scenario without saying so, and never act on a row as a different caller than arranged it | Pending verification and fit review; no completion claim. |
| #311 | ess-ui/1 renderers cannot run against a live served component (fixtures only) | Queued in PRs #375/#377/#380; served view parameters in #386. Candidate failures tracked by task:consumer-server-gate-corrections-20261002. |
| #309 | Aggregate with two group keys filled from one input is refused for a move the source does not have | Active: shared-input aggregate-key correction in grouped synthesis batch; story:feature-request-309. |
| #308 | A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer | Pending verification and fit review; no completion claim. |
| #307 | when_subject over a field copied from a related row at creation finds no candidate (ESS-SYNTH-003, then ESS-SYNTH-004) | Pending verification and fit review; no completion claim. |
| #305 | ess-ui/1: ess ui check accepts any Field.as value; the schema lists a closed set | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #304 | when_related through an Optional input: an absent reference takes the not-found branch instead of skipping the guard | Pending verification and fit review; no completion claim. |
| #303 | ess-ui/1: three document faults ess ui check reports nothing on (duplicate nav entry, unknown shell, shell without page outlet) | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #300 | ess-ui/1: widget expansion is exponential in nesting depth; a valid document can hang ess ui check | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #299 | No read of a row selected by a filter (the instance with field == input.x) in a guard or in sets: | Pending verification and fit review; no completion claim. |
| #298 | A Boolean input is not treated as a closed domain | Scoped gap: shared finite-domain proof excludes Boolean; implementation pending fit review and regression. |
| #297 | Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected | Pending verification and fit review; no completion claim. |
| #296 | Retrofit: no way to declare intended behaviour that the implementation is known not to meet, and count it apart | Pending verification and fit review; no completion claim. |
| #295 | mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited | Pending verification and fit review; no completion claim. |
| #294 | mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001) | Pending verification and fit review; no completion claim. |
| #293 | Explorer excludes every command with an Optional input (and every command with an unknown_instance branch) | Pending verification and fit review; no completion claim. |
| #292 | check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2) | Pending verification and fit review; no completion claim. |
| #291 | conform run --target interpreted answers wrong_state for an unknown identity where synthesis expects the declared not-found refusal | Queued candidate claims closure: PR #381. Not shipped; preserve issue until accepted evidence. |
| #290 | ess verify diff: no way to fail on a breaking change; a narrowing exits 0 | Pending verification and fit review; no completion claim. |
| #288 | An affects: filter over subject.<identity> validates but synthesis refuses it (ESS-SYNTH-001) | Pending verification and fit review; no completion claim. |
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

Includes possible stale records and product roadmap work; each requires a scope disposition before counting it as a consumer defect.

| Artifact | Status at intake | Outcome | Current disposition |
|---|---|---|---|
| story:a-branch-may-clear-the-field-it-owns | draft | A branch may clear the field it owns | pending source verification |
| story:a-browser-that-answered-http-once-is-still-a-slow-start | draft | A browser that answered HTTP once is still a slow start | pending source verification |
| story:a-killed-childs-outcome-says-which-signal-ended-it | draft | A killed child's outcome says which signal ended it | pending source verification |
| story:a-macro-invoked-twice-in-one-module-refuses-the-consumer-gate | active | A macro invoked twice in one module refuses the consumer gate | pending source verification |
| story:a-marked-region-is-not-a-scan-of-what-runs | draft | A marked region is not a scan of what runs | pending source verification |
| story:a-no-view-arranged-half-probes-the-row-through-the-command | draft | A no-view arranged half probes the row through the command | pending source verification |
| story:a-refusal-records-the-document-it-was-read-from | draft | A refusal records the document it was read from | pending source verification |
| story:a-report-says-why-a-scenario-was-skipped | draft | A report says why a scenario was skipped | pending source verification |
| story:a-skip-says-why-the-target-could-not-answer | active | A skip says why the target could not answer | pending source verification |
| story:acceptance-runs-as-toolchain-scenarios | draft | A toolchain story's acceptance is scenarios a conformance run decides | pending source verification |
| story:adopter-reviewed-delta | draft | An adopter's approval of a specification change is a committed delta | pending source verification |
| story:aggregate-views | active | A view can return aggregates over one entity's rows | pending source verification |
| story:binding-delivery-at-most-once | proposed | A binding can say it delivers at most once | pending source verification |
| story:browser-fixture-startup-deadline | active | The Firefox BiDi fixture assumes a 30-second startup on a shared runner | pending source verification |
| story:change-fragment-upgrade-obligation | draft | A change fragment states the upgrade a release asks of adopters | pending source verification |
| story:collection-quantifiers-witnessed-everywhere | draft | Every collection quantifier is witnessed with several elements, or says why not | pending source verification |
| story:collections-reach-their-upper-count-boundary | draft | An upper count bound on a collection is never sent at its accepting boundary | pending source verification |
| story:concurrent-history-records-inputs | draft | A concurrent history records each command's input | pending source verification |
| story:consumer-accounting-baseline-never-extended | active | Reconcile changed consumer obligations without extending initial eligibility | pending source verification |
| story:count-guards-above-one-are-synthesized | draft | A count guard above one gets synthesized scenarios | pending source verification |
| story:create-only-command-cannot-refuse | draft | A command that only creates cannot declare a refusal | pending source verification |
| story:cross-runtime-verdict-equivalence | draft | One corpus holds every emitted runtime to the same verdicts | pending source verification |
| story:cross-system-relation-target | draft | A relation may target an entity another system declares | pending source verification |
| story:crosswalk-verb-external-names-held-to-declarations | draft | ess verify crosswalk: hold external names (proto, REST, push) to the declarations that model them, refuse a gap | pending source verification |
| story:deleting-a-scratch-tmpdir-breaks-sccache-for-every-other-agent | draft | Deleting a scratch TMPDIR breaks sccache for every other agent | pending source verification |
| story:delivery-trust-fixture-race | draft | Delivery-trust tests never copy a fixture another test is rewriting | pending source verification |
| story:diff-classifies-error-payload-sources | active | The diff classifies error payload sources per outcome | pending source verification |
| story:empty-projection-is-refused-or-explained | active | A projection that writes nothing says why | pending source verification |
| story:ess-ui-type-grammar-aligns | draft | ess-ui types are spelled as ESS types | pending source verification |
| story:external-requests-are-assessed-before-adoption | active | An adopter request is assessed for fit before it is adopted | pending source verification |
| story:feature-request-194 | draft | a binding cannot invoke only when an Optional path is present (ESS-BINDING-015 leaves no way to say 'skip') | pending source verification |
| story:feature-request-197 | draft | a refused command cannot declare the compensating change the service makes before answering | pending source verification |
| story:feature-request-200 | draft | a search parameter on a view (a view parameter as the operand of contains / starts_with) | pending source verification |
| story:feature-request-212 | draft | ess verify conform mutate does not emit sets-drop, outcome-order-flip or the ==/!= and ±1 guard-boundary arms | pending source verification |
| story:feature-request-221 | draft | the explorer draws commands with existing_instance, subject_state or subject_predicate outcomes | pending source verification |
| story:feature-request-222 | draft | validate checks an authored scenario step's expected outcome against the command's guards | pending source verification |
| story:feature-request-223 | draft | explorer strings reach .count boundaries and example: values | pending source verification |
| story:feature-request-225 | draft | a guard comparing two identity-typed inputs (a self-edge check) | pending source verification |
| story:feature-request-228 | draft | No way to declare a multi-field key unique within a scope (equality, one arranged row) | pending source verification |
| story:feature-request-229 | active | No guard on another entity's state for non-creating commands, and no effect on related records | pending source verification |
| story:feature-request-231 | draft | Entity Runtime lowering refuses constructs ess/15-16 validate: unknown_instance, existing_instance, {related:}, {increment}, {cleared}, alphabet:, text .count, now | pending source verification |
| story:feature-request-233 | draft | Value expressions: dotted input paths in sets:/payload:, field arithmetic, sibling-field comparison, byte length | pending source verification |
| story:feature-request-236 | draft | mutate --emit has no --component: a repository implementing one component cannot score mutants | pending source verification |
| story:feature-request-237 | draft | Distinct list members and a count across records | pending source verification |
| story:feature-request-244 | draft | Guards over elapsed time since a stored instant and over calendar windows | pending source verification |
| story:feature-request-251 | active | A when_subject branch at an invariant upper bound is synthesized again | pending source verification |
| story:feature-request-257 | active | An aggregate group key copied from a related row is witnessed | pending source verification |
| story:feature-request-265 | active | An ungranted actor gets one declared refusal, witnessed per command | pending source verification |
| story:feature-request-266 | proposed | Synthesized scenarios account for bindings that move state | pending source verification |
| story:feature-request-267 | proposed | Binding flow, delivery and drop are synthesized into commands with wrong_state | pending source verification |
| story:feature-request-268 | proposed | A binding may react to one outcome of its source command | pending source verification |
| story:feature-request-269 | proposed | A binding failure policy may differ per refusal | pending source verification |
| story:feature-request-270 | active | A related sets value beside a when_related guard witnesses success | pending source verification |
| story:feature-request-271 | active | when_related over an owns via field is witnessed on both sides | pending source verification |
| story:feature-request-272 | active | A when_related guard on the creating command does not refuse the aggregate view | pending source verification |
| story:feature-request-273 | proposed | An event expectation checks identity fields against captured instances | pending source verification |
| story:feature-request-274 | active | A generated CLI hands unparsable dynamic input to the adopter validator | pending source verification |
| story:feature-request-275 | active | The caller-swapped run draws fresh identity inputs | pending source verification |
| story:feature-request-276 | active | Declarations added or removed leave no residual in the diff | pending source verification |
| story:feature-request-278 | active | An input guard beside a stored-field guard on one branch is synthesized | pending source verification |
| story:feature-request-279 | proposed | A stored-guarded moving command does not count as rewriting a group key | pending source verification |
| story:feature-request-280 | active | An enum-and-presence input guard is honoured by synthesis | pending source verification |
| story:feature-request-281 | draft | An ess-ui section has a heading, and a page can omit a section its kind contributes | pending source verification |
| story:feature-request-282 | draft | ESS-COMMAND-004 refuses a when_related refusal beside a wrong_state outcome; no precedence is stated | pending source verification |
| story:feature-request-283 | draft | A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004) | pending source verification |
| story:feature-request-284 | draft | ess ui check does not check that a page actor is granted the commands it binds | pending source verification |
| story:feature-request-285 | draft | {related:} reads through an Optional reference or across two references | pending source verification |
| story:feature-request-286 | draft | A view declares which actors may read it | pending source verification |
| story:feature-request-287 | active | A singleton entity can be declared and synthesized | pending source verification |
| story:feature-request-288 | draft | An affects: filter over subject identity is witnessed | pending source verification |
| story:feature-request-289 | active | A quantifier binder on the right of a comparison means the binder | pending source verification |
| story:feature-request-290 | draft | ess verify diff: no way to fail on a breaking change; a narrowing exits 0 | pending source verification |
| story:feature-request-291 | draft | conform run --target interpreted answers wrong_state for an unknown identity where synthesis expects the declared not-found refusal | pending source verification |
| story:feature-request-292 | draft | check-history: a generated Timestamp makes every history uncheckable (check.model-undetermined, exit 2) | pending source verification |
| story:feature-request-293 | draft | Explorer excludes every command with an Optional input (and every command with an unknown_instance branch) | pending source verification |
| story:feature-request-294 | draft | mutate: no way to declare a known-failing baseline scenario; one failure refuses the whole audit (ESS-MUTATE-001) | pending source verification |
| story:feature-request-295 | draft | mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited | pending source verification |
| story:feature-request-296 | draft | Retrofit: no way to declare intended behaviour that the implementation is known not to meet, and count it apart | pending source verification |
| story:feature-request-297 | draft | Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected | pending source verification |
| story:feature-request-298 | draft | A Boolean input is not treated as a closed domain | pending source verification |
| story:feature-request-299 | draft | A row selected by a filter can be read in a guard and in sets | pending source verification |
| story:feature-request-300 | draft | ess-ui/1: widget expansion is exponential in nesting depth; a valid document can hang ess ui check | pending source verification |
| story:feature-request-301 | active | synthesize is 20-30x slower since 0.40.0 on one specification (8 s to 4-7 min, scenarios +15%) | pending source verification |
| story:feature-request-303 | draft | ess-ui/1: three document faults ess ui check reports nothing on (duplicate nav entry, unknown shell, shell without page outlet) | pending source verification |
| story:feature-request-305 | draft | ess-ui/1: ess ui check accepts any Field.as value; the schema lists a closed set | pending source verification |
| story:feature-request-306 | active | Committed generated output regenerates in another checkout | pending source verification |
| story:feature-request-307 | draft | when_subject over a field copied from a related row is witnessed | pending source verification |
| story:feature-request-308 | draft | A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer | pending source verification |
| story:feature-request-309 | draft | Aggregate with two group keys filled from one input is refused for a move the source does not have | pending source verification |
| story:feature-request-310 | active | Generated code targets select a branch by whether the record exists | pending source verification |
| story:feature-request-312 | draft | Suites state their empty-target assumption and act across callers | pending source verification |
| story:field-sensitivity-class | draft | A field can carry a sensitivity class from a declared vocabulary | pending source verification |
| story:format-rule-for-relaxations | draft | A relaxation says which format admits it | pending source verification |
| story:go-and-typescript-read-current-suites | draft | Go and TypeScript packages read suite formats /28 to /33 | pending source verification |
| story:go-generated-behaviour | active | The Go target generates determined behaviours, view queries and invariant checks at parity with Rust | pending source verification |
| story:go-normalization-pattern-semantics | active | Qualify bounded ECMA-262 patterns in Go normalization | pending source verification |
| story:go-numbers-compare-by-value | active | Go conformance compares JSON numbers by value | pending source verification |
| story:held-state-has-one-operand | draft | Held state is selected by one operand | pending source verification |
| story:host-context-has-one-shape | draft | Host-bound context has one block and one prefix | pending source verification |
| story:integrate-source-driven-realizations | draft | Integrate source-driven realizations with the remediation baseline | pending source verification |
| story:interpreted-bindings-and-unmet-obligations | draft | A binding reacts under the interpreter, and an unmet obligation stays one | pending source verification |
| story:interpreted-eventual-views | draft | An eventual view is really eventual under the interpreter | pending source verification |
| story:interpreted-scenario-supplied-facts | draft | A scenario supplies what the model does not determine | pending source verification |
| story:interpreted-trust-gate | draft | The interpreter is checked against both hand-written targets | pending source verification |
| story:java-conformance-target | draft | A conformance suite can be emitted as a Java test package | pending source verification |
| story:list-and-text-guards-are-synthesized | active | List and text-ordering input guards get synthesized scenarios | pending source verification |
| story:mutation-audit-and-model-runner | active | ess audits a suite by mutation and explores sequences against the IR | pending source verification |
| story:native-realization-ci | draft | Run structural realization compiler checks in CI | pending source verification |
| story:normalization-equality-eligibility | draft | Specify numeric equality eligibility and its format compatibility boundary | pending source verification |
| story:optional-guards-mean-what-they-say | active | An Optional guard is refused or witnessed as its author meant | pending source verification |
| story:outcome-decided-by-environment | draft | An outcome a caller cannot see: refusals decided by neither input nor entity state | pending source verification |
| story:outcome-groups | active | One outcome can be declared for a group of commands | pending source verification |
| story:payload-fields-have-one-filling-rule | draft | Event and error payloads follow one filling rule | pending source verification |
| story:planning-store-carries-workstation-paths | active | 60 tracked files under .engineering/ carry home-directory paths the host-path lane does not scan | pending source verification |
| story:predicate-reference-page | active | Every predicate form ESS accepts is on one reference page | pending source verification |
| story:primitive-canonical-serialization | active | Canonical number serialization: the second stage of F08 | pending source verification |
| story:reader-conformance-over-refusals | draft | Reader-side conformance does not refuse what a reader accepts | pending source verification |
| story:reader-true-refused-for-closed-readers | draft | `reader: true` is refused where the reader is closed | pending source verification |
| story:related-guard-behaviour | active | Commands guarded by when_related are generated in the Rust and Go targets | pending source verification |
| story:related-guard-vocabulary-aligns | draft | `when_related` uses the vocabulary of its siblings | pending source verification |
| story:related-record-effects | draft | A non-creating command may declare an effect on related records | pending source verification |
| story:related-via-optional-input | active | when_related reads through an Optional input; an absent reference reads no row (ess/21) | pending source verification |
| story:related-via-stored-reference | active | when_related reads through a stored field of the subject, Optional included (ess/21) | pending source verification |
| story:release-status-publication-state | draft | Release status distinguishes drafts from public releases | pending source verification |
| story:report-carries-passed-failed-skipped | active | The conformance report carries passed, failed and skipped counts | pending source verification |
| story:review-stale-lines-corrected | draft | Stale lines found by the fit review are corrected | pending source verification |
| story:rust-recorder-does-not-lose-a-creation | draft | The Rust recorder does not lose a creation it observed | pending source verification |
| story:scrub-the-planning-store-or-say-why-not | draft | The reason for leaving the planning store unscanned was false; decide again on the real numbers | pending source verification |
| story:served-committed-command-answers-its-outcome | draft | A committed command is answered with its outcome, not 501 | pending source verification |
| story:served-store-and-entry | active | A served component gets a generated in-memory store and server entry point | pending source verification |
| story:served-view-params | active | Synthesized servers pass declared view parameters from the query string to the view port | pending source verification |
| story:shared-public-gates | active | Adopt independent common source gates | pending source verification |
| story:source-pinned-data-normalization | active | Source-pinned checked data normalization across Go Rust and TypeScript | pending source verification |
| story:specification-declares-its-ess-release | active | A specification declares the ess release it is maintained with | pending source verification |
| story:specify-upgrade-command | draft | ess specify upgrade moves a specification to the next source format and checks the delta | pending source verification |
| story:stored-field-guards | active | An outcome can be guarded by the addressed entity's stored fields | pending source verification |
| story:string-length-guards | active | A guard can test the length of a String | pending source verification |
| story:string-newtype-declares-its-alphabet | active | A String newtype can declare its character set | pending source verification |
| story:string-prefix-suffix-substring-operators | active | String guards can test a prefix, a suffix or a substring | pending source verification |
| story:suite-pins-transitions-updates-and-boundaries | active | Generated suites pin transition targets, update values, every source and guard boundaries | pending source verification |
| story:the-browser-fixture-abandons-a-profile-per-start | draft | The browser fixture abandons a profile per start | pending source verification |
| story:the-design-page-is-held-to-the-fixture-it-describes | draft | The design page is held to the fixture it describes | pending source verification |
| story:the-generated-go-runtime-is-gofmt-clean | draft | The emitted Go runtime is not gofmt-stable, so an adopter's formatter changes it | pending source verification |
| story:the-interpreter-executes-stored-field-guards | draft | The interpreter executes guards over the subject's stored fields | pending source verification |
| story:the-lane-does-not-pin-a-count-that-its-own-bookkeeping-moves | draft | The lane does not pin a count that its own bookkeeping moves | pending source verification |
| story:the-metadata-guard-rejects-every-build-but-one | draft | The metadata guard rejects every build but one | pending source verification |
| story:the-published-schema-admits-the-name-aliases-the-parser-reads | draft | The published schema admits the name aliases the parser reads | pending source verification |
| story:the-startup-clamp-does-not-outlive-the-startup | draft | The startup clamp does not outlive the startup | pending source verification |
| story:the-startup-lock-does-not-cover-the-first-round-trip | draft | The startup lock does not cover the first round trip | pending source verification |
| story:the-unread-tree-bullet-is-read-whole | draft | The unread-tree bullet is read whole | pending source verification |
| story:typed-literals-in-sets-and-unknown-instances | active | sets: accepts typed literals and an unknown instance has a declared answer | pending source verification |
| story:types-only-realizations | active | Consistent types-only realizations for Go Rust and TypeScript | pending source verification |
| story:typescript-conformance-target | draft | A conformance suite can be emitted as a TypeScript test package | pending source verification |
| story:ui-react-live-binding | active | The generated React app reads and commands a synthesized server; refusals show where the user acted | pending source verification |
| story:ui-spec-style-tokens | draft | An ess-ui document declares design tokens, themes and user preferences | pending source verification |
| story:ui-tui-app-generator | active | ess generate ui --target tui emits a Rust terminal app crate | pending source verification |
| story:ui-tui-live-binding | active | The TUI reads and commands a synthesized server (ess ui run --tui --model --base-url) | pending source verification |
| story:union-tag-inline-with-fields | draft | A union can carry its tag beside its variant's fields | pending source verification |
| story:validate-sees-what-synthesize-refuses | active | validate reports authored-scenario and unset-field problems synthesize would hit | pending source verification |
| story:web-bridge-answers-like-http | draft | The web bridge answers a command with the HTTP surface's shape | pending source verification |
| story:wrong-state-witness-unknown-and-own-stored-guards | draft | A wrong-state witness handles unknown sibling guards and the moving branch's own stored guard | pending source verification |
