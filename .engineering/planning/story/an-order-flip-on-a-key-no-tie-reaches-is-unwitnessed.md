---
format: aep.planning-md/3
id: story:an-order-flip-on-a-key-no-tie-reaches-is-unwitnessed
kind: story
status: active
title: An order-flip on a key no tie reaches is unwitnessed
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#708
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_gained_refusals.rs
- confidence: cited
  path: docs/design/mutation-audit-and-model-runner.md
- confidence: inferred
  path: website/docs/guides/verify/mutation-audit.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:34Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:36Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

Some sort keys of a ranked view can never decide an order between two rows a scenario arranges: a
key after the identity, or a key behind an earlier key that no creating command writes from its
input. The `order-flip` mutant on such a key is scored `survived`, exit 1, although no suite could
kill it and nothing in the specification answers it.

Minimal reproduction, `ess 0.92.0`: the view of
`story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie` over `example.order.Order`
(`region: String`, `priority: Integer`, both written from the input of `example.order.PlaceOrder`,
identity `order_id` minted by the target), ranked by the identity first:

```yaml
views:
  - name: example.order.OrderQueue
    source: example.order.Order
    consistency: read_your_writes
    order_by: [order_id asc, priority asc]
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: region, type: String}
      - {name: priority, type: Integer}
```

```
$ ess verify conform mutate --path spec-identity-first --target interpreted --class order-flip
mutation audit of example v1 against interpreted: 2 mutant(s), 1 killed, 1 survived, 0 inconclusive, 0 stillborn, 0 unwitnessed, 0 equivalent (baseline: 1 scenario(s), 0 refusal(s)); 2 of 2 site(s) scored
survived order-flip/example.order.OrderQueue/priority: `priority asc` becomes `priority desc`
killed order-flip/example.order.OrderQueue/order_id: `order_id asc` becomes `order_id desc` — by example.order.PlaceOrder/outcome/placed (1 in total)
exit 1
```

The mutant's suite differs from the baseline's (the `ranked` expectation carries the flipped key),
so the "suite does not change" rule does not apply (`crates/verify/ess-conformance/src/mutate.rs:5432-5437`),
and `unwitnessed_outcome` answers `None` for every `order-flip` mutant
(`crates/verify/ess-conformance/src/mutate.rs:3195-3199, 5291-5326`).

Once synthesis records a standing gap for a view's key it cannot tie
(`story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie`), the audit reads that gap: an
`order-flip` mutant on a (view, key) the baseline holds such a gap for, which no scenario killed, is
`unwitnessed` (`ESS-MUTATE-004`), and its entry lists the gap under `baseline_refusals`, as a mutant
on an outcome whose scenario the baseline refused does today. A kill still wins, so an authored
scenario under `--target interpreted --scenarios` that does kill it is reported as the killer.

## Acceptance

- On the reproduction above, `ess verify conform mutate --path spec-identity-first --target
  interpreted --class order-flip` prints `2 mutant(s), 1 killed, 0 survived, 0 inconclusive, 0
  stillborn, 1 unwitnessed`, exits 3, and the line for
  `order-flip/example.order.OrderQueue/priority` carries `ESS-MUTATE-004` and names the baseline's
  refusal by its code.
- `--report-out` on the same run writes that mutant with a non-empty `baseline_refusals`, whose
  entry names the scenario `example.order.PlaceOrder/outcome/placed` and a subject that holds the
  view and the key.
- `--emit DIR` on the same specification, reports written from the interpreted target, then
  `--collect DIR`, gives the same verdicts and exit status.
- On `order_by: [region asc, priority asc]`, where every tie is arranged, no mutant is unwitnessed
  and the audit exits 0.
- An `order-flip` mutant on a key with no such gap that every scenario passes is still `survived`,
  exit 1 (a test with the tie arrangement switched off for one fixture, or a hand-written manifest
  without the refusal key).

## Scope

- cited: crates/verify/ess-conformance/src/mutate.rs
- cited: docs/design/mutation-audit-and-model-runner.md
- inferred: crates/verify/ess-conformance/tests/mutation_audit.rs
- inferred: crates/verify/ess-conformance/tests/mutation_gained_refusals.rs
- cited: crates/edge/ess-cli/src/main.rs
- inferred: website/docs/guides/verify/mutation-audit.md
- cited: website/docs/reference/cli.md

confidence: medium

The `mutate` help text is the doc comment at `crates/edge/ess-cli/src/main.rs:913-923`, and the
command reference in `cli.md` (line 1590) is regenerated from it. Shared surfaces moved: proposes
the `CHANGELOG.md` sentence.

## Fit review

1. **The need, apart from the proposed syntax.** The audit calls a mutant a survivor where no
   scenario could have told it from the baseline and the specification cannot be changed to make it
   observable. Reproduced with `ess 0.92.0` on a new minimal specification, output above. The
   requester's proposal, labelled as theirs: "If such a pair cannot be arranged (for example, the
   leading key is not settable from input), the mutant would be better reported as unwitnessed
   (ESS-MUTATE-004) with that reason than as a survivor."
2. **Class: defect.** The help defines a survivor as "a declared rule no synthesized scenario pins
   down", answered "by declaring what makes the rule observable, or by filing a synthesis gap", and
   defines `unwitnessed` for a mutant "on an outcome whose scenario the baseline refused"
   (`ess verify conform mutate --help`, 0.92.0). A key whose tie the baseline could not arrange is
   the same situation on a view, and it is scored the other way.
3. **Can it already be expressed?** No. `--known-failing` excludes baseline scenarios, not mutants
   (`ess verify conform mutate --help`), and there is no accepted-survivors file by decision
   (`docs/design/mutation-audit-and-model-runner.md:900`). Dropping the dead key from `order_by:`
   answers the identity-first case only, and changes what the specification says.
4. **Fit.** No authored surface; no grid cell (`docs/design/language-grids.md`). It extends one
   existing rule to a sibling site: `unwitnessed_outcome` reads the baseline's refusal keys for
   outcome and transition sites (`crates/verify/ess-conformance/src/mutate.rs:5285-5326`), and a
   view site joins them, keyed by the `RefusalKey` subject, which already names the view for the
   view refusals (`mutate.rs:4137-4147`). The verdict, the code, the `baseline_refusals` field and
   exit 3 all exist. `--target`, `--emit`/`--collect` and `--component` share the scorer.
5. **A second adopter.** A ledger view `example.ledger.Entries` with
   `order_by: [posted_at desc, entry_id asc]`, where `posted_at` is stamped by the target and not
   sent by the caller: no scenario can make two entries tie on it, so the flip of `entry_id` cannot
   be witnessed by synthesis.
6. **Cost.** No source format and no suite format. The report and the manifest already carry
   `baseline_refusals` and refusal keys, so no bump of `ess-mutation-report` or
   `ess-mutation-manifest` is expected; if the view-and-key subject needs a new field in either,
   that is a bump and is decided in the change. An audit that exited 1 on such a mutant exits 3.
   Help text, guide and CHANGELOG change.
7. **Alternatives.** (a) Change nothing: a permanent survivor per unreachable key, which teaches
   the reader to ignore survivors. (b) List the site as `unmutated` with a reason: wrong register,
   since `unmutated` means the class found no edit a target could tell apart
   (`ess verify conform mutate --help`), and here a target can, on rows the suite cannot arrange.
   (c) Refuse a key after the identity in validation: a separate question about the source
   language, not needed to make the audit honest, and it does not cover an unsteered earlier key.
   (d) `unwitnessed`, citing the baseline's gap: chosen, as the requester proposed.

## Decisions

accept as proposed.

An `order-flip` mutant on a view key the baseline's synthesis holds a standing tie gap for is
`unwitnessed` (`ESS-MUTATE-004`) unless a scenario killed it. Design page to extend first: the
`order-flip` row of the class table in `docs/design/mutation-audit-and-model-runner.md`, which
today says only that the mutant survives where the order is refused.

## Source

github:beyond10x/ess#708
