---
format: aep.planning-md/3
id: story:a-command-line-binding-declares-the-exit-status-of-each-outcome-of-the-command-it-forwards-and-a-suite-reads
kind: story
status: draft
title: A command-line binding declares the exit status of each outcome of the command it forwards, and a suite reads it
summary: An ess-cli binding fixes exit 0 for success and 1 or 2 by the kind of reply, chosen in handler code; no table says which outcome or refusal exits how and no scenario reads an exit status (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:ess-cli-flag-and-repeated-sources
- informed_by: story:feature-request-274
- informed_by: story:the-cli-projection-carries-timestamp-decimal-and-uuid
revision: 3
---
## Need

A downstream application reported on 2026-11-24 (twelfth gap report; reporter name kept out). A process a scheduler starts ends with an exit status the scheduler reads (a retry, an alert). The status should follow what the commands it issued ended in: success when a record was processed or skipped, non-zero when an external system refused. The specification cannot state which outcome or refusal of a command ends the process with which status, and no scenario can read a status.

Status: the declaration and the observation are defined here. The aggregate status of a process that issues several commands waits on RN-D1 (`story:a-specification-may-state-one-run-of-a-process-with-a-command-once-per-member-of-a-set-and-a-command-once-per`).

## Reproduction

Installed `ess` 0.98.0, generic model `run-c.yaml` (an `Account` entity, a command `Notify(account_id)` with an accepting outcome, an `external:` refusal and a wrong-state refusal, a component accepting it) and an `ess-cli/2` binding with `target: {kind: service_forward, owner: <component>, operation: <Notify>}` and `errors: {mail_refused: <type>}`:

| Probe | Result |
|---|---|
| `ess specify cli --path run-c.yaml --binding cli.yaml` | valid |
| the compiled plan's obligations | `handler:notify: implement the owner-qualified callable and its declared result/error contract`; no field ties `mail_refused` to the command's outcome `mail-refused`, and no field states an exit status |
| a `Uuid`-backed identity input | `unsupported CLI primitive Uuid` (story `the-cli-projection-carries-timestamp-decimal-and-uuid`, draft) |

What exists: success exits 0; the handler's `Error` reply exits 1 and `UsageError` exits 2, chosen in code; `invalid_input: <code>` answers a declared error with exit 2 (`docs/design/cli-presentation-binding.md:165-174, 279-287`). Nothing reads an exit status in a suite (`docs/design/conformance-target-process.md:160-162`: the target's exit status "is not part of the report").

## Fit review

1. Need: which outcome and refusal of a forwarded command ends a process with which status, and a check that the process does. The requester proposed "an exit status declared per outcome or per refusal"; the key's home is this story's design, not theirs.
2. Class: gap. The declaration is missing from a presentation binding; the observation is missing from the suite. `cli-presentation-binding.md:286-287` ("the adapter does not infer an exit status from payload fields") says the mapping is the application's, so ESS behaves as documented.
3. Existing idiom: the handler returns `Error` or a result. Policy 1 (`docs/design/generic-language-features.md` § Policy) rules it out: the figure a reader needs (which outcome exits how) sits in handler code and no suite checks it.
4. Fit: the HTTP binding declares a status per outcome and is checked against the command's outcomes (`docs/design/http-presentation-binding.md:23`, § Status per outcome); the CLI binding is the sibling cell. Reuse its shape: a table over the command's outcomes, refused when an outcome is unmapped. The observation follows the `await_outcome` pattern: a default-unsupported target capability and one suite step (`docs/design/system-issued-commands.md` § Target interface). The legacy `clap` target (`ess-synth/src/clap/tree.rs:355-385`) returns the handler's `ExitCode` and is covered by the same table through a refusal that names it until it carries it.
5. Second adopter: a retention sweep or a billing run started by a CronJob whose `retry_limit` (`docs/design/runtime-workload-kinds.md`) acts on the status; a deployment hook that must fail the rollout on a refused external call.
6. Cost: a new `ess-cli` format pair (`/1`, `/2` keep their bytes), a plan field and generated-runtime change, reference text; a suite major, one suite step and a target capability in Rust, Go and TypeScript; a diff class for a changed exit.
7. Alternatives: change nothing and document the handler's convention (fails policy 1); the status on the model's outcome (two spellings; RN-D2 B); a status per declared error only (RN-D2 C). Chosen: RN-D2 A and RN-D3 A.

## Decisions

Accept, redesigned. Taken on the default of each option list, 2026-11-24 16:29 CET (the coordinator, under the owner's delegation):

- RN-D2, where an exit status is declared. Decided: A, in the `ess-cli` binding, per callable: the status of success, a table from each outcome and refusal of the forwarded command to success or a declared error code, and an `exit:` on each declared error; the binding is refused when an outcome is mapped to nothing and no default is stated, or when a code names no declared error. Statuses admitted: 0, 1 and 3 to 125 (2 stays the usage class; 126 and above and 130 stay the adapter's). A new `ess-cli` format pair; `/1` and `/2` keep their bytes, and policy 2 places the number. B was `exit:` on the model's outcome (two spellings of one concept); C a status per declared error only (fails policy 1). The default holds while RN-D1 is not C.
- RN-D3, how a scenario reads an exit status. Decided: A, a default-unsupported target capability and one suite step (`expect_exit`) over a process target of the CLI binding, in Rust, Go and TypeScript, as `await_outcome` was added: a target that cannot report a status answers unsupported, never passes. B was no scenario observation (fails policy 1); C an exit field in the `ess-conformance-target/1` answer (changes a persisted protocol).
- Not decided here: the status of a process that issues several commands (RN-D1, open with the owner in `story:a-specification-may-state-one-run-of-a-process-with-a-command-once-per-member-of-a-set-and-a-command-once-per`).

## Acceptance

- an_outcome_table_is_checked_against_the_command: an `ess-cli` binding whose `service_forward` callable maps every outcome and refusal of its command validates; an unmapped outcome with no default, a code naming no declared error, and a status outside the admitted set are each refused by name.
- a_mapped_refusal_exits_with_its_status: the generated package, run with a handler that returns the command's `external:` refusal, exits with the declared status; the accepting outcome exits 0; the plan lists the table and carries no handler choice of status.
- a_suite_reads_the_exit_status: synthesis writes a scenario per mapped outcome that ends in `expect_exit` with the declared status; a target that exits with another status fails it, and a target that cannot report a status answers unsupported, never passes.
- an_old_binding_keeps_its_bytes: `ess-cli/1` and `/2` bindings compile to the plan bytes they did before; an older reader refuses the new format by version.
- the_legacy_clap_target_refuses_by_name: a model whose command-line component is bound to the table and emitted by `--target clap` is refused naming the table, until that target carries it.
- the_diff_names_a_changed_exit: `ess verify diff` reports a changed exit status of an outcome and classifies it.

## Scope

- crates/specify/ess-cli-contract/src/wire.rs, resolve.rs  cited, the binding and its checks against the command's outcomes
- crates/generate/ess-cli-project/src/runtime.rs  cited, the generated runtime's exit
- crates/generate/ess-synth/src/clap/tree.rs  cited, the legacy target's refusal
- crates/verify/ess-conformance/src/scenario.rs, target.rs, runner.rs, synthesize.rs  inferred, the step and the capability
- crates/verify/ess-conformance/src/go/runtime.go, ts/runtime.ts  inferred
- crates/verify/ess-diff/src  inferred
- docs/design/cli-presentation-binding.md, website/docs/reference/formats.md  cited

Borders: `story:ess-cli-flag-and-repeated-sources` (active, same `ess-cli-contract` files; order the merges), `story:feature-request-274` (implemented, `invalid_input`), `story:the-cli-projection-carries-timestamp-decimal-and-uuid` (draft).
