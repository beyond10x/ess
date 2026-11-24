---
format: aep.planning-md/3
id: story:a-specification-may-state-one-run-of-a-process-with-a-command-once-per-member-of-a-set-and-a-command-once-per
kind: story
status: draft
title: A specification may state one run of a process, with a command once per member of a set and a command once per run
summary: Nothing states how often one run of a process issues a command or over which records, so no scenario observes a whole run; waits on decision-blocker:run-of-a-process (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-command-line-binding-declares-the-exit-status-of-each-outcome-of-the-command-it-forwards-and-a-suite-reads
- informed_by: story:a-system-issued-creating-command-is-witnessed
- informed_by: story:a-periodic-binding-may-run-on-a-wall-clock-schedule
- informed_by: story:periodic-binding-trigger-contract
- informed_by: story:runtime-realizes-a-scheduled-job
- informed_by: story:an-ordered-selection-may-be-counted-as-a-value-with-rows-before-this-one-and-its-head-may-be-the-subject-of-a
revision: 3
---
## Need

A downstream application reported on 2026-11-24 (twelfth gap report; reporter name kept out). A scheduled process reads a set (one row per member), issues command X once per member, continues with the other members after one is skipped, and issues command Y once per run. The specification says that a system actor may issue a command, and nothing says how many times one run issues it or over which records. Synthesis refuses the scenarios of a system-issued command that names no single record (`ESS-SYNTH-023`), and a synthesized scenario awaits one issuance on one record, so no scenario observes a whole run.

Status: architecture decision pending (RN-D1). The Acceptance below is written for option B and changes with the pick.

## Reproduction

Installed `ess` 0.98.0, generic model `run.yaml` (entity `Account` with `Pending -> Notified`; a system actor holding `Notify(account_id)`, which names its record, and `CloseReport()`, which creates a row):

| Probe | Result |
|---|---|
| `ess verify conform synthesize` | 6 scenarios; refusal `ESS-SYNTH-023` on `CloseReport/outcome/closed` ("creates the record it addresses, so no record exists before the step") |
| authored `ess-scenario/1`: two accounts arranged, await `Notify` on the first (accepted), on the second with the external refusal forced, a second await on the first (wrong state) | compiles (`/52`), 1 of 1 passes on the interpreted target |
| authored scenario awaiting `CloseReport` | `ESS-AUTHOR-045` |
| a binding invoking a system-issued command | `conflicting_declaration`: "a binding's invocation is already the system acting; two causes of one command are not admitted" (`ess-domain/src/system_issued.rs:105-125`) |

The authored scenario shows what exists: per-member outcomes for rows the author lists, with a refused member followed by a processed one. It states neither that the members are every row of a set, nor that each is issued once, nor that Y is issued once.

## Fit review

1. Need: a model states how often a process issues a command in one execution and over which rows (once per row of a selection, once overall), that a refused member does not stop the others, and a scenario observes the whole execution. The requester proposed `once_per: <view or set>` beside a run cause; their spelling is not adopted.
2. Class: gap, ARCHITECTURE: it introduces a run, a thing a specification states (what ESS owns; a new kind of declaration under option C).
3. Existing idiom: the authored multi-await scenario above, for rows an author lists. Policy 1 rules it out as the answer: "every row", "exactly once" and "Y once" are figures the reader sees and no suite checks. A set effect (`instances:`, `affects: each:`) is one call that changes many rows; it cannot give each member its own outcome or its own external answer.
4. Fit: grid R (`docs/design/language-grids.md:27`) selection cells; siblings that iterate or pick: `affects: each:` (one entry per element, one call), `instances:` (one call), `A4` one event several times (one call), `b2b3` head of an ordered selection (one pick; `story:an-ordered-selection-may-be-counted-as-a-value-with-rows-before-this-one-and-its-head-may-be-the-subject-of-a`, draft), periodic host reads (one call per tick, `periodic-binding-triggers.md`). A fan-out of calls reuses their selection vocabulary. It composes with system-issued commands (awaited per record, `system-issued-commands.md`), with bindings (causes; the refusal above), with the periodic observation records (one record per binding, lifetime, ordinal and invocation) and with the CLI binding for the exit status (`story:a-command-line-binding-declares-the-exit-status-of-each-outcome-of-the-command-it-forwards-and-a-suite-reads`). Every target refuses by name or carries it: Rust, Go, TypeScript, the interpreter, `ess verify diff`, generated docs, synthesis.
5. Second adopter: a billing run that charges each due subscription and posts one summary; a retention sweep that archives each expired document and ends with a report; a reconciliation that polls each device and closes a batch.
6. Cost: per RN-D1. Option B: binding keys and a source format (policy 2), one suite major and one step, a target capability in Rust, Go and TypeScript, diff classes, docs. Option C: a document kind beside `ess-cli` and `ess-transport`, an IR, diff, every target.
7. Alternatives: A (no run; handler fans out), B (run cause with fan-out), C (a run document), D (deployment only): `DECISIONS.md` RN-D1.

## Decisions

Deferred. `decision-blocker:run-of-a-process` blocks this story and this story waits on it.

RN-D1 (ARCHITECTURE): does a specification state a run of a process, and how. Open, with the owner; no default; with the owner since 2026-11-24, no answer yet. A: no run; ESS keeps one command per scenario, a scheduled process is a command-line process bound by `ess-cli`, the fan-out and the counts stay the handler's, and only the exit status per outcome of one forwarded command is added (RN-D2, RN-D3); need 2 is declined as out of the language. B: a run is a cause of bindings with a fan-out on the invocation (a run cause beside `event:` and `periodic:`; `invoke:` may repeat over a typed list; system-issued commands become admissible as invocations of that cause; one suite step observes the issuances of a run). C: a run is its own declaration (a document beside `ess-cli` and `ess-transport`, working name `ess-run/N`, with causes, ordered steps, a continuation policy and an exit table). D: a run stays a deployment fact (`ess-runtime` says when a process starts; ESS states nothing of what it issues or how it exits; both needs declined). The coordinator recommends B, taken in two steps so that nothing waits on it that need not: RN-D2 and RN-D3 first, which stand under A and B and give the exit status of a single-command process, then the run cause with fan-out. The Acceptance below is written for B and changes with the pick. Format follows the pick: A and D add nothing; B opens a source format by policy 2; C opens a document format.

Decided apart from it, taken on the default, 2026-11-24 16:29 CET (the coordinator, under the owner's delegation):

- RN-D4, the no-subject system-issued command. Decided: A, one acceptance sentence on `story:a-system-issued-creating-command-is-witnessed`, as SW-D6 did for set effects. B was a separate story.
- RN-D5, the one-shot scheduled process and the periodic cause. Decided: A, a process started per schedule by the deployment is not a periodic binding; the wall-clock story records that, and the cause of such a process is the run of RN-D1. B (widen the periodic cause to a one-shot process) changes a released contract.
- The exit status of a single-command process: `story:a-command-line-binding-declares-the-exit-status-of-each-outcome-of-the-command-it-forwards-and-a-suite-reads`.

## Acceptance

Written for option B.

- a_run_issues_a_command_once_per_member: a scenario arranges three rows in the selected set and takes the run; the run issues X exactly three times, once on each row, each with its own outcome; a target that issues X twice for one row, or for a row outside the set, fails.
- a_refused_member_does_not_stop_the_run: the second member's step is refused by its external system; X is still issued on the third; a target that stops at the first refusal fails.
- a_command_once_per_run_is_issued_once: Y is issued exactly once in the run, also with an empty set and with every member refused; a target that issues it twice or never fails.
- the_run_ends_with_the_declared_status: the run's exit status follows the outcomes' declared exits (`story:a-command-line-binding-declares-the-exit-status-of-each-outcome-of-the-command-it-forwards-and-a-suite-reads`): 0 when every member was processed or skipped, the declared non-zero status on an external refusal.
- a_binding_may_invoke_a_system_issued_command_under_the_run_cause: the refusal at `system_issued.rs:105-125` stays for every other cause and is lifted only for the run cause.
- unimplemented_targets_refuse_by_name: a target without the run capability answers unsupported by name; an older reader refuses the suite by version.

## Scope

- crates/specify/ess-domain/src/binding.rs, binding/periodic.rs, system_issued.rs  cited, the cause, the invocation, the refusal
- crates/specify/ess-compiler/src/ir.rs, resolve.rs  inferred
- crates/verify/ess-conformance/src/periodic.rs, scenario.rs, target.rs, runner.rs, synthesize/system_issued.rs  cited, the observation, the step, the await
- crates/verify/ess-conformance/src/go/runtime.go, ts/runtime.ts, interpret.rs  inferred
- crates/verify/ess-diff/src, crates/generate/ess-gen/src/docs.rs  inferred
- docs/design/system-issued-commands.md, periodic-binding-triggers.md, binding-list-selection.md  cited

Borders: `story:a-system-issued-creating-command-is-witnessed` (active; AM-1), `story:a-periodic-binding-may-run-on-a-wall-clock-schedule` (draft; AM-2), `story:periodic-binding-trigger-contract` (implemented), `story:runtime-realizes-a-scheduled-job` (implemented), `story:payload-source-reads-the-head-of-an-ordered-selection` (implemented), `story:a-set-effect-selection-reads-the-row-it-references-and-a-bulk-delete-removes-children-before-parents` (draft), `story:a-specification-may-declare-the-records-another-system-owns-without-claiming-ownership-and-check-a-job-over` (draft, blocked).
