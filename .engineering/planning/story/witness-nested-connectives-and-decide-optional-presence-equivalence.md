---
format: aep.planning-md/3
id: story:witness-nested-connectives-and-decide-optional-presence-equivalence
kind: story
status: draft
title: Witness nested any/all connectives per child and decide Optional presence in mutation equivalence
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Synthesis witnesses every `any:` and `all:` node of a guard, not only the top-level one, so the
`guard-connective` mutant of a conjunction nested in a disjunction (and of a disjunction nested in
a conjunction) is killed by rule rather than by luck of the candidate walk; and a `precedence-swap`
of two branches whose guards test `defined` of `Optional` scalar inputs, and that no input satisfies
together, is scored `equivalent` instead of `survived`. Requested in
https://github.com/beyond10x/ess/issues/501, the nested case the
https://github.com/beyond10x/ess/issues/155 fix (commit `dda521887a`, released in 0.37.0) left out.

## Fit review

1. **Need, apart from the proposed syntax.** No new syntax is asked for. Domain fact: "an order
   names its item in exactly one of three ways (code, SKU, URL)" is a disjunction of conjunctions,
   `any: [all: [a, b], all: [a, c], all: [b, c]]`. The mutation audit cannot confirm the synthesized
   suite pins the inner `and`. Minimal reproduction, `.engineering/repro/501/system.yaml`
   (validates: `catalog v1 — 1 file(s), valid`). `ess 0.56.0`,
   `ess verify conform mutate --path system.yaml --target interpreted`, output in `nested.out`:
   `10 mutant(s), 6 killed, 4 survived, … 0 equivalent`, with
   `survived guard-connective/catalog.orders.Place/too-many/1: (defined(item_code) and defined(item_sku)) becomes (defined(item_code) or defined(item_sku))`
   (`/2`, `/3` the same), plus
   `survived precedence-swap/catalog.orders.Place/too-many/none-given`. The same shape over
   `Boolean` inputs (`bools.yaml`, `bools.out`): `3 survived … 1 equivalent`, the same three
   `guard-connective` survivors. So the connective gap is not about `defined`, and the precedence
   swap is scored correctly once its leaves are decidable. A flat `all:` (`flat.yaml`, `flat.out`)
   is killed: `2 killed, 0 survived`. *Requester's proposal (theirs):* "Apply the #155 witness rule
   at every nesting level … and score a precedence swap of two guards no input satisfies together
   as `equivalent`."
2. **Class: defect** for the connective part. `docs/design/mutation-audit-and-model-runner.md:166`
   makes every `All`/`Any` node with two or more children, in pre-order, a `guard-connective` site.
   The 0.37.0 CHANGELOG entry (`CHANGELOG.md:1867`) states the #155 rule without any depth limit:
   "`any:` guards are witnessed once per disjunct and `all:` once per conjunct". The code applies it
   only at the top level. `conjuncts` takes "the leaves of its top-level `all`"
   (`crates/verify/ess-conformance/src/synthesize.rs:12828-12834`). The guarded branch isolates
   only an `any` that is a top-level conjunct (`synthesize.rs:13179-13188`), and the default
   isolates only top-level conjuncts (`synthesize.rs:13238-13244`). **Gap** for the
   precedence-swap part. The documented decidable fragment excludes it: `satisfiable` returns
   `None` for an optional leaf (`crates/verify/ess-conformance/src/mutate.rs:2129`), and
   `equality_tests` admits no `Predicate::Defined` (`mutate.rs:2215-2257`, falls to `_ => false`).
   `ess verify conform mutate --help` says the same: "decided only for equality, membership and
   truth tests of input fields against literals". Design line `mutation-audit-and-model-runner.md:173`
   defers the precedence-swap verdict to the `ESS-MUTATE-005` decision.
3. **Already expressible?** Not by the author. The mutate help says a survivor "is answered by
   declaring what makes the rule observable, or by filing a synthesis gap — not by authoring a
   scenario". Restating the guard as a flat `all:` changes the rule. A per-pair refusal branch
   (three outcomes) changes the declared error surface and loses the "exactly one" fact. No idiom
   exists.
4. **Fit.**
   - *Vocabulary:* there is no new authored surface. The fix generalises the existing
     `one_per_child` / `exactly_one` (`synthesize.rs:13074-13127`) from "children of the top-level
     connective" to "children of every connective reached with positive polarity". An `any` node
     calls `one_per_child(alone = true)` on the guarded branch. An `all` node calls it with
     `alone = false` on the default. Under `not:` the polarity flips. `keep` already re-decides
     every row with `selects_branch` (`synthesize.rs:13151-13158`), so a row that does not select
     the branch is never sent.
   - *Composes with:* `when:` input guards (this fix). `when_subject` stored-field guards were
     covered by #155 at top level (`dda521887a` touched `synthesize/subject_fact.rs`;
     `limit_goals` handles "a top-level conjunct, or a disjunct of a top-level `any:`",
     `subject_fact.rs:5973-5998`). Whether stored fields show the same nested gap is inferred, not
     reproduced. The story reproduces it first and either fixes it or records the refusal.
     Bindings, views and outcomes are untouched.
   - *Sibling generality:* the overlap witnesses (`overlap_inputs`) and boundary rows already
     search at any depth for windows (`synthesize.rs:12875`). Ordered bounds stay top-level, and
     that limit is out of scope here.
   - *Targets:* only synthesis and the mutation audit change. The interpreter, Rust/Go/TypeScript
     runners, `ess verify diff`, Entity Runtime and generated code are unaffected: no new
     construct reaches them, only additional `execute_command`/`expect_outcome` steps in existing
     scenario shapes.
5. **Second, unrelated adopter.** "A payment names exactly one instrument: card, bank account or
   wallet" has the same three-pair `any` of `all`. A shipping rule
   `any: [all: [quantity > 10, express], all: [quantity > 50, fragile]]` (`second.yaml`,
   `second.out`) is killed today (`3 killed, 0 survived`), but only because the walk happened to
   produce a disjunct-alone row. Nothing guarantees it.
6. **Cost.** No source format bump, no new keyword, no new diagnostic. No `ess-conformance/N`
   bump: the steps are existing kinds. No manifest/report format change: `equivalent` and
   `unsatisfiable_guard` already exist in `ess-mutation-manifest/4`. Synthesized suites for
   specifications with nested connectives gain rows. Committed generated suites with nested guards
   may need regeneration under `docs/design/synthesis-fixture-maintenance.md`; which ones is not
   known yet (inferred). Diff classification is unchanged.
7. **Alternatives.**
   - (a) *Change nothing:* the documented rule stays false at depth >= 2, and authors cannot
     answer the survivor. Rejected.
   - (b) *Special-case "a conjunction directly inside a disjunction"* (the literal reading of the
     proposal): leaves `all` in `any` in `all`, and `any` under `not:`. Rejected for the
     polarity-recursive rule, which is the same code path.
   - (c) *Score a precedence swap equivalent by a separate disjointness check:* duplicates
     `satisfiable`. Rejected. Teaching `satisfiable` a two-valued presence dimension for `Optional`
     scalar leaves (absent, or the existing literal/other domain) fixes the swap and, for free,
     dead-guard `ESS-MUTATE-005` over `defined`/`missing`. The proposal's intent is taken, its
     mechanism changed.

## Decisions

**Accept, redesigned.**

- **Witness rule at every depth.** Walk the guard and track polarity (positive at the root,
  flipped under `not:`). For every positive `any` node, and every negative `all` node, with two or
  more children: the guarded branch gets one row per child where that child holds and its siblings
  fail, chosen by `one_per_child(…, alone = true)` over that node's children. For every positive
  `all` node, and every negative `any` node: the default of each guarded sibling gets one row per
  child where exactly that child fails, by `one_per_child(…, alone = false)`. Every row still passes
  `keep`/`selects_branch`, so the enclosing structure is honoured without being encoded
  separately. A child already isolated by the plain witness or an earlier row adds nothing, as
  today, which keeps suites without nested connectives byte-identical.
- **Presence in the equivalence decision.** `equality_tests` admits `Predicate::Defined(path)`
  (and its negation through `not:`). `satisfiable` treats an `Optional` scalar leaf as
  `{absent} ∪ <its existing finite domain>` and counts the extra value inside the existing
  `MAX_CANDIDATES` product. Collections, text lengths, orderings and `Timestamp` stay undecided as
  documented. The precedence-swap verdict then follows unchanged from
  `mutation-audit-and-model-runner.md:173`.
- **Changed from the request:** polarity under `not:` is included; nesting is arbitrary, not one
  level; the precedence-swap fix goes through `satisfiable`, not a swap-specific rule.
- **Stored fields:** the implementor first reproduces a nested `when_subject` connective. If it
  survives, the same polarity walk is applied in `synthesize/subject_fact.rs`. If that cannot be
  done within this story, the refusal is recorded in a follow-up story named in the PR. Nothing
  is silently left out.

## Acceptance

- `ess verify conform mutate --target interpreted` over the issue-shaped specification (three
  optional inputs, `any:` of three pairwise `all:`, an all-absent refusal, `accepts: nothing`
  default) reports `0 survived`: the three nested `guard-connective` mutants are `killed` and the
  `precedence-swap` mutant is `equivalent`. Pinned by a new test in
  `crates/verify/ess-conformance/tests/` (e.g. `nested_connective_mutants.rs`).
- The `Boolean` variant of the same shape reports `0 survived` with the three nested connective
  mutants `killed`. Same test file.
- `any` nested in `all` nested in `any`, and an `all` under `not:`, are each witnessed per child.
  Every nested `guard-connective` mutant is killed. Same test file, one case each.
- An `Optional` input guard pair `defined(x)` / `not defined(x)` swapped by `precedence-swap` is
  `equivalent` with `unsatisfiable_guard` naming the overlap. A pair that can both hold is still
  scored by running. Test in `crates/verify/ess-conformance/tests/mutation_unkillable.rs` or a new
  file.
- A guard `defined(x) and not defined(x)` produced by a guard mutant is reported dead
  (`ESS-MUTATE-005`), and a satisfiable optional guard is never called dead. Same test file as the
  previous bullet.
- A suite synthesized for a specification with only top-level connectives is byte-identical to its
  0.56.0 synthesis: the existing `connective_and_source_mutants.rs` and
  `adversary*_connective_and_source_mutants.rs` tests stay green unchanged, and
  `cargo test -p ess-conformance --locked` passes.
- A nested connective in a `when_subject` stored-field guard is either killed (test added) or
  named in a follow-up story cited in the PR.
- `website/docs/reference/predicates.md` and the `mutate --help` text state the new decidable
  fragment (presence of `Optional` scalars) and the any-depth witness rule. CLI help drift is
  checked by `task check`.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`: **held file, needed.** `boundary_inputs`
  (cited, :13144), `one_per_child` (cited, :13097), `exactly_one` (cited, :13074), `conjuncts`
  (cited, :12829). A new polarity-aware node walk (inferred).
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`: **held (`src/synthesize/**`),
  conditionally needed** (inferred). Only if the stored-field reproduction shows the same gap
  (`limit_goals` :5986, `conjunct_goals` :6246, cited).
- `crates/verify/ess-conformance/src/mutate.rs`: not held, needed. `satisfiable` (:2111) and
  `equality_tests` (:2215), cited.
- `crates/verify/ess-conformance/src/witness.rs`: not held. Possibly touched if `first_where`
  needs the node's children as a goal (inferred).
- `crates/edge/ess-cli/src/main.rs`: not held, `mutate` help text (inferred).
- `website/docs/reference/predicates.md`, `docs/design/mutation-audit-and-model-runner.md`: docs
  (inferred).
- `crates/verify/ess-conformance/tests/`: new tests (inferred).
- **Not needed:** `src/interpret/execute.rs`, `interpret/execute/{related,existence}.rs`,
  `crates/specify/ess-domain/src/command.rs`, `command/**`,
  `crates/specify/ess-compiler/src/ir.rs`, `ir/**`. There is no change to the language, the IR or
  the interpreter. `Predicate::Defined` already exists in `ess-primitives`
  (`crates/specify/ess-primitives/src/predicate.rs:1304`, cited).
- Format/version: none (no `ess/N`, `ess-conformance/N` or manifest bump). Committed generated
  suites with nested connectives may change and need regeneration (inferred).
