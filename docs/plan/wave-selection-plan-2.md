# Wave 2 of `epic:one-selection-plan` — the selection plan's design and type

> **Status: approved 2026-10-07 by the operator (option A).** Proposed against `origin/main` `985f58cc3a`; opened on `49f83bfc5a`, which contains PR #487. The
> wave starts only from a `main` that contains PR #487 (beyond10x/ess#486, the held-state order
> rule), as the epic requires. Skill: `aep:implementing` 0.20.1, wave mode. Close follows this
> repository's practice: the unit lands on `integrate/selection-plan`, which is delivered as one
> pull request against `main` and merged through the App when its `Gate` is green. No tag, no
> release.

Wave 1 of the epic was the #486 fix itself (PR #487). This wave delivers the plan the four later
waves read.

## 1. Pre-flight — measured 2026-10-07

| fact | evidence |
|---|---|
| root filesystem 22 G free of 848 G, 98 % | `df -h /` |
| the #487 tree's `target/` holds 19 G | `du -sh` on `ess-selection-precedence/target`; finished with `worktree finish --discard-cache --archive` once #487 merges, before this wave builds |
| `aep 0.68.0` | `aep --version` |
| ess linked trees: 5, of which 2 belong to the `ess` controller's wave `ess-wave-20261007b` | `git worktree list` |
| one measured build: `ess-domain` and `ess-compiler` test targets of the #486 work fit in under 2 G; a full `ess-conformance` candidate batch reached 19 G | the #487 tree's `target/` |

## 2. Units

| unit | story | objective | scope | depends on |
|---|---|---|---|---|
| U1 | `story:selection-plan-design-and-type` | `vision:O2` | cited: `docs/design/selection-plan.md`, `crates/specify/ess-compiler/src/ir.rs`, `crates/specify/ess-domain/src/command.rs`; inferred: `ess-compiler/src/ir/precedence.rs`, `ess-domain/src/command/precedence.rs`, `ess-compiler/tests/selection_precedence_table.rs` and its fixture | — |

N is one. The other six stories depend on this one (`depends_on`), so none is ready.

### `aep plan artifact waves --kind story --status draft`, restricted to this epic's stories

Selection path: the verb (`aep 0.68.0`). The store holds 138 draft-story collisions in total; the
lines below are every one that names a story of this epic, as the verb printed them.

Waves:

```text
wave 1: story:selection-plan-design-and-type
wave 2: story:entity-runtime-lowering-reads-selection-plan
wave 2: story:generated-behaviour-reads-selection-plan
wave 2: story:validation-reads-selection-plan
wave 3: story:synthesis-reads-selection-plan
wave 4: story:interpreter-reads-selection-plan
wave 4: story:overlap-witnesses-per-phase
```

Collisions:

```text
{"a":"story:feature-request-361","b":"story:interpreter-reads-selection-plan","path":"crates/verify/ess-conformance/src/interpret/execute.rs","confidence":"inferred"}
{"a":"story:feature-request-361","b":"story:overlap-witnesses-per-phase","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"cited"}
{"a":"story:feature-request-361","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"cited"}
{"a":"story:feature-request-362","b":"story:interpreter-reads-selection-plan","path":"crates/verify/ess-conformance/src/interpret/execute.rs","confidence":"inferred"}
{"a":"story:feature-request-362","b":"story:overlap-witnesses-per-phase","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"cited"}
{"a":"story:feature-request-362","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"cited"}
{"a":"story:generated-behaviour-reads-selection-plan","b":"story:rust-binding-unused-event","path":"crates/generate/ess-synth/tests","confidence":"inferred"}
{"a":"story:interpreted-eventual-views","b":"story:selection-plan-design-and-type","path":"crates/specify/ess-compiler/src/ir.rs","confidence":"cited"}
{"a":"story:interpreter-reads-selection-plan","b":"story:the-interpreter-executes-stored-field-guards","path":"crates/verify/ess-conformance/src/interpret/execute.rs","confidence":"cited"}
{"a":"story:optional-value-invariant-observation","b":"story:overlap-witnesses-per-phase","path":"crates/verify/ess-conformance/src/mutate.rs","confidence":"inferred"}
{"a":"story:optional-value-invariant-observation","b":"story:overlap-witnesses-per-phase","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"inferred"}
{"a":"story:optional-value-invariant-observation","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/authored.rs","confidence":"inferred"}
{"a":"story:optional-value-invariant-observation","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"inferred"}
{"a":"story:outcome-decided-by-environment","b":"story:selection-plan-design-and-type","path":"crates/specify/ess-domain/src/command.rs","confidence":"inferred"}
{"a":"story:outcome-decided-by-environment","b":"story:validation-reads-selection-plan","path":"crates/specify/ess-domain/src/command.rs","confidence":"inferred"}
{"a":"story:overlap-witnesses-per-phase","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/synthesize.rs","confidence":"cited"}
{"a":"story:overlap-witnesses-per-phase","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","confidence":"inferred"}
{"a":"story:overlap-witnesses-per-phase","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","confidence":"cited"}
{"a":"story:overlap-witnesses-per-phase","b":"story:synthesis-reads-selection-plan","path":"crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv","confidence":"cited"}
{"a":"story:overlap-witnesses-per-phase","b":"story:wrong-state-witness-unknown-and-own-stored-guards","path":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","confidence":"cited"}
```

Unassessed: none of this epic's stories.

The verb's later waves differ from the epic's table (it puts validation in its wave 2 and the
interpreter in its wave 4) because it also weighs collisions with stories outside the epic. The
verb wins for selection; the epic's table is the intended sequence and is re-checked against the
verb when each later wave is proposed.

## 3. Plan critics

Two rounds over the epic and its seven stories, four critics each (`aep:plan-critic-acceptance`,
`-design`, `-scope`, `-parallel-safety`), recorded as `review-result:one-selection-plan-<critic>-round-<n>`.
Round 1: 15 findings, round 2: 7; every one revised and recorded `fixed` (`review_outcome`
evidence). No finding is open. Round 2's revisions were not reviewed again (two-round limit).

## 4. Unit records

| unit | managed worktree id | branch | build directory | scratch root | stage |
|---|---|---|---|---|---|
| integration | `ess-selection-plan` | `integrate/selection-plan` | its own `target/` | — | open at `49f83bfc5a` |
| U1 | `ess-selection-plan-u1` | `unit/selection-plan-u1-design-and-type` | its own `target/` | `~/.cache/ess-selection-plan/u1-scratch` | dispatching; brief `u1-scratch/brief.md` |

Dispatch types: `aep:implementor` for U1, then `aep:adversary` against its tree.

## 5. Commits this wave makes

The opening store commit (the epic, its seven stories, the eight review results, their evidence and
this page) on `integrate/selection-plan`; one commit for U1 on its branch; the merge of U1 into
`integrate/selection-plan`; the closing store commit (U1's evidence, its scope rewrite and its
move); the push of `integrate/selection-plan` and one pull request against `main`, merged through
the App when `Gate` is green. Nothing else: no tag, no release, no later wave.
