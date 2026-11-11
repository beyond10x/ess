---
format: aep.planning-md/3
id: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for
kind: story
status: implemented
title: A presence child beside a read of the same optional is witnessed by the absent input and never refused
summary: ESS-SYNTH-022 asks for a witness of defined(x) beside a comparison on x that an earlier branch already masks, which no input can give (beyond10x/ess#729)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#729
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_t1_lane2_presence_child.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/mutation-manifest-4-scoped.json
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/mutation-report-4-scoped.json
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/presence_child_beside_read.rs
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:33:38Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:33:39Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-10T17:53:38Z", actor: "human:timo", revision: 18, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Need

A guard that tests the presence of an `Optional` input beside a comparison on the same input is the
narrowing form the language designs (`defined(x)` narrows `x`). Synthesis refuses its per-child
witness with `ESS-SYNTH-022` and asks the author for an input that cannot exist: the field absent
and, at once, a comparison on that field decided. A comparison that reads an absent `Optional` is
unknown, so it is neither true nor false for any input. The help text sends the author to an
authored scenario that nobody can write.

The cause is not the earlier branch. The refusal appears with and without a branch that takes the
absent row first, for `all` and for `any`, for an enum and for an `Integer`:

```yaml
format: ess/23
system: demo
version: v1
domain: demo.review
components:
  - component: review-server
    owns: {domains: [demo.review]}
    accepts: {commands: [demo.review.RecordDecision]}
    reached_by: network
types:
  - {name: demo.review.Kind, kind: enum, variants: [Note, Correction]}
  - {name: demo.review.CheckStatus, kind: enum, variants: [Reported, Verified]}
errors:
  - name: demo.review.CorrectionNotVerified
commands:
  - name: demo.review.RecordDecision
    input:
      - {name: kind, type: demo.review.Kind}
      - {name: check_status, type: Optional<demo.review.CheckStatus>}
    outcomes:
      - name: correction-not-verified
        when:
          all: [kind == Correction, defined(check_status), check_status != Verified]
        error: demo.review.CorrectionNotVerified
      - name: recorded
        accepts: nothing
```

`ess verify conform synthesize --path . --out suite.json` (0.92.0):

```text
refused: refusal[ESS-SYNTH-022]: outcome demo.review.RecordDecision/recorded has a scenario `demo.review.RecordDecision/outcome/recorded` that leaves a connective's child unwitnessed
  no solved candidate of the 6 tried satisfies `(kind == Correction and not (defined(check_status)) and check_status != Verified)`
  help: cover the child with an authored scenario (ess-scenario/1) that sends an input where it alone decides the connective, or write the guard so that such an input exists
2 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

The same refusal is printed with an earlier branch
`incomplete: all: [kind == Correction, not defined(check_status)]` (the reported shape), for
`all: [defined(minimum), defined(maximum), minimum > maximum]` over two `Optional<Integer>` (two
refusals), and for `any: [not defined(minimum), minimum > 10]`.

Behind the wrong refusal is a missing check. In the specification above the suite sends
`{Correction, Reported}`, `{Note, Reported}` and `{Correction, Verified}`, and never
`{kind: Correction}` with `check_status` absent. For `any: [not defined(minimum), minimum > 10]`
it sends `minimum: 101` and `minimum: 10`, and never the absent `minimum` that the first child
exists for. A target that drops the presence test and reads an absent value as unequal, or as
zero, passes the suite. So the presence child is both refused and unchecked.

## Acceptance

Tests in `crates/verify/ess-conformance/tests/`, each over an inline specification, against
`ess_conformance::synthesize::synthesize` and the interpreter:

1. For the specification above, synthesis returns no `ESS-SYNTH-022`, and the `recorded` scenario
   sends `{kind: Correction}` with `check_status` absent and expects `recorded`.
2. With the earlier branch `incomplete: all: [kind == Correction, not defined(check_status)]`
   added, synthesis returns no `ESS-SYNTH-022`; `{kind: Correction}` with `check_status` absent is
   sent once, by the `incomplete` scenario, and no other scenario sends it.
3. For `all: [defined(minimum), defined(maximum), minimum > maximum]` over two
   `Optional<Integer>`, synthesis returns no `ESS-SYNTH-022`, and the default branch's scenario
   sends one input with only `minimum` absent and one with only `maximum` absent.
4. For `any: [not defined(minimum), minimum > 10]`, synthesis returns no `ESS-SYNTH-022`, and the
   guarded branch's scenario sends an input with `minimum` absent and expects that branch.
5. A hand-written target for specification 1 that answers `correction-not-verified` for
   `{kind: Correction}` with `check_status` absent fails the `recorded` scenario; the interpreter
   passes every scenario of 1 to 4.
6. A child whose isolating goal is satisfiable and that the bounded search misses is still refused
   with `ESS-SYNTH-022`: the existing cases in `tests/string_alphabet_and_length.rs`,
   `tests/authored_child_witness.rs` and `witness.rs`
   `a_satisfiable_goal_is_never_read_as_unsatisfiable` stay green unchanged.
7. `website/docs/reference/predicates.md` no longer says that `defined(x)` beside a comparison on
   `x` cannot be witnessed; "What synthesis can witness" states the rule of this story.

## Scope

Delivered in turn 1 of the zero-open-issues plan (lane story 2). The paths below are the ones
its commits changed, read from the diff, so every entry is cited.

- cited, confirmed: `crates/verify/ess-conformance/src/synthesize.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/presence_child_beside_read.rs`
- cited, confirmed: `website/docs/reference/predicates.md`
- cited, added: `crates/verify/ess-conformance/tests/adversary_t1_lane2_presence_child.rs`
- cited, added: `crates/verify/ess-conformance/tests/fixtures/mutation-manifest-4-scoped.json`
- cited, added: `crates/verify/ess-conformance/tests/fixtures/mutation-report-4-scoped.json`
- cited, added: `crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv`
- cited, added: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`

Scoped but not touched:

- `crates/verify/ess-conformance/src/witness.rs`

Shared surface written by the coordinator, not a scope entry: `CHANGELOG.md`.

## Fit review

1. **The need, apart from the proposed syntax.** A specification states "refuse a correction whose
   check status is present and not `Verified`" as `all: [kind == Correction,
   defined(check_status), check_status != Verified]`. Synthesis refuses the child `defined(…)`
   with `ESS-SYNTH-022` and never sends the absent input. Reproduction: the specification in Need,
   `ess verify conform synthesize` output quoted there (0.92.0). The requester's proposal, theirs:
   count the child as witnessed, or as shadowed by the earlier branch that takes the absent input.
2. **Class: defect.** `website/docs/reference/predicates.md:46-47` says a predicate is
   three-valued and a fact not observed is unknown; `:450` says a comparison that reads an absent
   `Optional` is unknown. The goal the refusal prints therefore holds for no input.
   `predicates.md:1823-1826` says a child with no input at all is not refused and `ESS-SYNTH-022`
   is for a search that "did not try every value of every field"; `diagnostics.md:215` tells the
   author to write a scenario with such an input. The refusal contradicts both. The reason in the
   code: `exactly_one` returns false for any `Decision::Unevaluable` child
   (`crates/verify/ess-conformance/src/synthesize.rs`, `exactly_one`), `witness::unsatisfiable`
   reads text literals only (`witness.rs:443`), and `witness::exhausts` answers false for an
   optional path (`witness.rs`, `if builder.optionals.contains(&path) { return false; }`).
3. **Can it already be expressed?** Partly, and not for every type.
   `"coalesce(minimum, 0) > coalesce(maximum, 0)"` beside the two `defined` checks synthesizes
   with 0 refusals (`ess verify conform synthesize` on that guard: `2 scenario(s) (0 authored), 0
   refusal(s)`). Over an enum the same idiom is refused at validation:
   `[type_mismatch] … coalesce(check_status, Verified) is neither`, and
   `docs/design/language-grids.md` grid O marks `coalesce(x, d)` over an enum "no". The idiom also
   changes what the reader sees: a default that the guarded branch can never read, because
   `defined(x)` already holds there. So the idiom is no answer, and the design page spells the
   narrowing form as the language's own (`docs/design/generic-language-features.md:93`).
4. **Fit.** No grid cell opens or closes: `defined()` is admitted over an `Optional` of every type
   (grid O, row `defined()`), and comparison rows are unchanged. The change is in synthesis only,
   and it follows the row the design already has: "presence, `coalesce`, `if`: one witness per
   arm: the fact absent and present" (`generic-language-features.md:193`). It is general across
   siblings by rule, not by case: any connective (`all`, `any`, either under `not`, at any depth),
   any presence spelling (`defined`, `not defined`, `missing`, `exists:`), any leaf that reads the
   same path outside `coalesce` (comparison, membership, text operator, count). Targets: the suite
   gains ordinary command steps, which the interpreter, the generated Rust and Go runners and the
   web runner already execute; `ess verify diff`, generated documents, the entity runtime and
   authoring read no new construct. `ESS-SYNTH-022` keeps its meaning and its text.
5. **A second adopter.** A range with optional ends: `all: [defined(minimum), defined(maximum),
   minimum > maximum]` refuses an inverted range and accepts an open one. It prints two
   `ESS-SYNTH-022` refusals today (`OUT/repro/run.txt`, case `c`).
6. **Cost.** No source format bump, no keyword, no new diagnostic, no diff classification, no
   generated-API change. Suites of specifications with this guard shape gain steps (one absent
   input per presence child that reaches the connective's own branch or its default), so their
   bytes change as with every synthesis improvement; the suite format does not. One paragraph of
   the reference page is replaced.
7. **Alternatives.**
   - *Change nothing, answer with `coalesce`.* Fails question 3: no idiom over an enum, and the
     absent input stays unchecked.
   - *As requested: treat a child masked by an earlier branch as covered.* Fits only the reported
     shape. The refusal is the same with no earlier branch (case `e`), and there the absent input
     reaches the default and is never sent.
   - *Only drop the refusal* (extend `witness::unsatisfiable` to a presence test beside a decided
     read of the same path). Removes the message and leaves the presence child unchecked.
   - *Chosen: three-valued isolation.* A child is isolated at an input where it alone decides the
     connective and every sibling that can be decided there decides the opposite; a sibling that
     reads the path whose absence the child tests is unknown at that input and does not block the
     row. The row is sent to the branch the model selects for it. Where an earlier branch takes
     it, it is that branch's row, as the code already says ("an input that isolates the child but
     selects another branch is that branch's"), and nothing is refused. A goal that needs a path
     absent and a comparison on it decided is shown empty, never refused.

## Decisions

accept, redesigned.

The need is real and is a defect. The requested shape (a child masked by an earlier branch counts
as covered) is changed to a rule that does not depend on an earlier branch: a presence child is
witnessed by the absent input, with siblings that read the same path left unknown, and a goal no
three-valued input can satisfy is not a refusal. This also adds the missing check on the presence
child, which the request did not ask for and the reproduction shows is absent. `coalesce` over an
enum stays as grid O has it; this story removes the need for it as a workaround and does not open
that cell.

## Source

github:beyond10x/ess#729

