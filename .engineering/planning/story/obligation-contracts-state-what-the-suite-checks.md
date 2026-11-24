---
format: aep.planning-md/3
id: story:obligation-contracts-state-what-the-suite-checks
kind: story
status: active
title: Obligation contracts state what the suite checks
summary: Generated Rust obligation contracts omit view order_by, misprint an empty-string guard, omit refusal precedence and the ASCII-only trimmed rule (0.77.0)
tags:
- defect
- ess-batch-7
- ess-next
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/items.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:16Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:17Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Whoever implements an obligation from the contract `ess generate synthesize --target rust` writes
reads the same rules the conformance suite checks: the view's order, the guard as the IR holds it,
the refusal precedence and the trimming rule.

## Problem

Reported 2026-10-29 against ESS 0.77.0 by a downstream consumer; reproduced on 0.92.0.

Model (`spec.yaml`, `format: ess/27`): entity `notes.core.Note {owner, title}`; command
`notes.core.Create` with outcome `blank` `when: {input.title: {trimmed: {in: [""]}}}` (error
`notes.core.Blank`) before `created`; command `notes.core.Close` with `blank-reason` on
`input.reason` the same way before `closed` (a move); view `notes.core.OpenNotes` with
`params: [owner]`, `filter: owner == param.owner`, `order_by: [title asc, note_id asc]`.

```
$ ess generate synthesize --path spec.yaml --target rust --out rs
13 capabilities: 10 generated, 3 obligation(s), 0 refused
```

1. The view-query obligation drops `order_by`. PLAN.md, plan.json and the `OpenNotesQuery` trait
   doc read "…at `read_your_writes` consistency, containing instances where `owner == param.owner`."
   and say nothing of the order, while the suite checks it. Writers:
   `crates/generate/ess-synth/src/plan.rs:1105`, `rust/items.rs:406`, `go/items.rs:526`.
2. The empty-string guard prints as `trimmed(title in [])`, a guard that never holds, in PLAN.md,
   plan.json, the trait doc, and also the generated OpenAPI descriptions and docs.md.
   `Predicate::AnyOf`'s `Display` joins the values unquoted
   (`crates/specify/ess-primitives/src/predicate.rs:4423-4425`), so `""` disappears.
3. The command contract says "Declared outcomes (declaration order, not selection precedence)"
   and states the precedence only for related guards (`plan.rs:709-713`, `related_precedence`
   at `:720`); it does not say that input-guarded refusals answer before not-found and before
   wrong-state, though the suite pins that order.
4. `trimmed` strips ASCII whitespace only (`trim_ascii_whitespace`, `predicate.rs:552-562`), and the
   contract does not say so. An implementer using Rust `str::trim` treats a no-break space as
   empty where the suite does not. An authored scenario for the no-break-space case is refused
   with ESS-AUTHOR-041.

beyond10x/ess#798 (2026-11-21, ess 0.96.0) shows the related-guard sentence of `related_precedence` (`crates/generate/ess-synth/src/plan.rs:720`, text at `:789-796`) stating the reverse of what runs: an input-guarded refusal answers before `existing_instance` and `exists: false`, and an `external:` refusal with `before_row: true` answers before both; the contract states both.

## Acceptance

- Over the model above, the `OpenNotes` view-query contract in PLAN.md, plan.json and the
  generated trait doc (Rust and Go) states `ordered by title asc, note_id asc`, the keys and
  directions exactly as the IR holds them.
- The `blank` guard is printed `trimmed(title in [""])` everywhere `Predicate`'s `Display` reaches
  (PLAN.md, plan.json, trait doc, OpenAPI description, docs); a text value is quoted.
- The `Create` and `Close` contracts state that an input-guarded refusal answers before not-found
  and before wrong-state.
- A contract holding a `trimmed` guard states that `trimmed` strips ASCII whitespace only (space,
  `\t`, `\n`, `\v`, `\f`, `\r`). Synthesized scenarios do not rely on a non-ASCII space.
- One test per item in `crates/generate/ess-synth/tests/` compares the printed contract against
  what the suite checks, so the two cannot drift apart again.

- Red today: over the reproduction model (`ess/37`), the `depot.racks.AddShelf` obligation
   contract names its branches in the positions the phase order gives them: the input refusal
   (`no-label`) and the `before_row: true` external (`not-cleared`) before existence
   (`duplicate`), existence before the held state, the held state before `exists: false`
   (`unknown-rack`) and the present related refusal (`slot-taken`), and those before the accepting
   branch (`added`). The test asserts only these relative positions. It does not contain
   "`exists: false` before input-guarded refusals", and it does not contain "Related-presence
   predicates do not precede input-guarded refusals".
- The sentence is derived from the one phase order, `Phase::step_in(ir.format())` over the
   command's `PrecedencePlan` (`crates/specify/ess-domain/src/command/precedence.rs:102-117`, with
   `place` at 471-540 deciding each branch's phase), so it is whatever that order yields, including
   `input_absent:` first and the default last, and a branch `place` moves to another step on a
   command that reads a row first. The sentence and the plan the interpreter reads cannot name
   different orders for one format; no order is written by hand in `plan.rs`.
- Below `ess/27` the sentence is byte for byte today's: the three existing tests in
   `tests/declared_behaviour.rs` (lines 674-723, fixture at `ess/18`) stay green unchanged.
- The branch for a related row read through a stored field of the subject (`plan.rs:759-771`) is
   checked against the same rule and corrected if it differs (it already puts input refusals
   first; it names no `before_row` external).

## Scope

- cited: crates/generate/ess-synth/src/plan.rs
- cited: crates/generate/ess-synth/src/rust/items.rs
- cited: crates/generate/ess-synth/src/go/items.rs
- cited: crates/specify/ess-primitives/src/predicate.rs
- inferred: crates/generate/ess-synth/tests/declared_behaviour.rs
- inferred: crates/generate/ess-gen/tests/openapi.rs

confidence: medium

## Amendment of 2026-11-24: the related-precedence sentence (beyond10x/ess#798)

Folded, not a unit of its own: the story's item 3 ("the contract states the refusal precedence")
and this issue are the same function, `related_precedence` in
`crates/generate/ess-synth/src/plan.rs`, and the story already owns `plan.rs` and
`tests/declared_behaviour.rs`. The amendment adds one acceptance, one test and one changelog
reference to the story; nothing else in the story changes.

Size added to the story: S (about 80 changed lines: 35 source, 45 test).
Fixed suite bytes of committed models: none. No committed file carries the sentence
(`git grep 'Selection precedence'` finds only `plan.rs`); the existing tests pin it on an `ess/18`
fixture, whose order is the old rule and stays.
Patch-release candidate: yes (no committed byte, no diagnostic). The operator ordered #798 built
now in the hotfix wave on the released tag (2026-11-24, 03:10 CET); this amendment is the record
for the story, which keeps its other items for the later merge.

### Need

beyond10x/ess#798. For a command with `when_related:` in a source format from `ess/27` on, the
generated `PLAN.md` (and `plan.json`, the contract in every obligation that carries it) states a
selection precedence the interpreted target and the conformance suite do not follow: it puts
`existing_instance` and `exists: false` before the input-guarded refusals and says nothing of
`before_row: true` externals. An implementor who follows the sentence fails the suite.

### Reproduction

Reproduced on 0.98.0. Model `repro/798/` (the issue's model at `format: ess/37`, with
`unknown-rack` declared before `slot-taken`, which 0.98.0 requires: ESS-COMMAND-004 "`unknown-rack`
answers at step 4 before `slot-taken`").

```console
$ ess generate synthesize --path . --target rust --layout crate --out out/rust
17 capabilities: 16 generated, 1 obligation(s), 0 refused
$ grep -o 'Selection precedence: on commands with `when_related:`, check[^.]*\.' out/rust/PLAN.md
Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; …
```

The order that runs (the issue's four authored scenarios pass on the interpreted target; the two
stating the sentence's order are refused as ESS-AUTHOR-041), and the order the code holds:
`crates/specify/ess-domain/src/command/precedence.rs`, `ONE_RULE = V27` (line 32),
`Phase::step_in` (lines 102-117): input refusals and `before_row` externals (1), existence (2),
held state (3), every `exists: false` then every present-related refusal (4), accepting (5).

### Scope (added)

- `crates/generate/ess-synth/src/plan.rs`, `related_precedence` (lines 720-802): under
  `reads_one_rule(ir.format())`, write the sentence by walking `Phase::PRECEDENCE` in the order
  `Phase::step_in(ir.format())` gives and naming each phase's branches from the command's
  `PrecedencePlan` (acceptance 7); `before_row: true` externals appear where the plan puts them
  (with the input refusals). Below the rule, today's text stays (acceptance 8).

### Owned files (added)

- `crates/generate/ess-synth/src/plan.rs` (already the story's)
- `crates/generate/ess-synth/tests/declared_behaviour.rs` (already the story's; new `#[test]`s)

### Not yours

- The Rust and Go `behaviour` module comments (`rust/behaviour.rs:21-39`, `go/behaviour.rs:17`):
  they describe generated code that follows the plan and were not reported; the story's item 3
  decides whether they change.

### Tests first (added)

In `crates/generate/ess-synth/tests/declared_behaviour.rs`, a new
`plan_contract_states_the_one_rule_for_related_commands`: the reproduction model inline at
`ess/37`, `SynthesisPlan::of`, the `AddShelf` obligation's contract, assert acceptance 6 by the
positions of the phrases. A second `#[test]` builds the same command at `ess/26` and asserts the
old sentence (acceptance 8 from the other side). In-process, well under 5 s each (bound 60 s per
test, 120 s per step).

### Gate (added to the story's)

One at a time; no build-pool job over 10 minutes.

- `cargo clippy -p ess-synth --all-targets --locked -- -D warnings`
- `cargo test -p ess-synth --locked --test declared_behaviour`
- byte pins of `ess-synth`: `--test served_codec_names`, `--test go`
- `task test-xtask` (the unit edits `CHANGELOG.md`)
- No stamp: `ess-synth/src` is not among the sources `synthesis-sources.digest` hashes.

### Depends on

The story it amends (same function); nothing else.

### Changelog (added to the story's bullet, or its own)

Under `Fixed`:

- The obligation contract of a command with `when_related:` from `ess/27` states the precedence the
  interpreted target and the suite follow, written from the same phase order, where it put
  `existing_instance` and `exists: false` before the input refusals (#798).

### Fit review

1. Need: an implementor must be able to build from the contract and pass the suite. No syntax.
2. Class: defect: the generated contract contradicts the documented rule
   (`precedence.rs:22-32`, and `docs/design/generic-language-features.md` § G4 cited there).
3. Expressible: not applicable (generated text).
4. Fit: no surface; one sentence derived from the plan the interpreter already reads.
5. Second adopter: every command with `when_related:` at `ess/27` or later.
6. Cost: generated `PLAN.md`/`plan.json` text for such commands; no format.
7. Considered: a unit of its own (rejected: same function and test file as the active story, which
   would land them one after the other); deleting the sentence (rejected: the story asks for the
   precedence to be stated); fold (chosen).

### Decisions

- Decision: accept as proposed, folded into story:obligation-contracts-state-what-the-suite-checks.
