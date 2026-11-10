---
format: aep.planning-md/3
id: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie
kind: story
status: draft
title: A tie-breaking sort key is pinned by a synthesized tie
summary: 'order-flip on a view''s second order_by key always survives: synthesis never arranges a tie on the first key, and --emit takes no authored scenarios (#708)'
tags:
- ess-batch-7
- ess-next
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#708
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/ordered_head.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/paging.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/mutation-order-tie.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
- confidence: cited
  path: docs/design/mutation-audit-and-model-runner.md
- confidence: cited
  path: docs/design/read-api-view-idioms.md
- confidence: inferred
  path: website/docs/guides/verify/mutation-audit.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 12
---
## Need

A view ranked by more than one key has one `order-flip` mutant per key. Only the flip of the first
key is killed by the synthesized suite. Synthesis arranges two rows for the `ranked` check and gives
each its own value in every field, so the rows differ on the first key and no later key ever decides
their order. `ranked` reads the next key only where the earlier ones are equal
(`crates/verify/ess-conformance/src/runner.rs:3857-3861`), so the baseline's and the mutant's
expectation hold for the same target.

Minimal reproduction, `ess 0.92.0` (`format: ess/32`, one domain `example.order`):

```yaml
entities:
  - name: example.order.Order
    identity: {name: order_id, type: example.order.OrderId}
    fields:
      - {name: region, type: String}
      - {name: priority, type: Integer}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed], transitions: []}
commands:
  - name: example.order.PlaceOrder
    input:
      - {name: region, type: String}
      - {name: priority, type: Integer}
    outcomes:
      - name: placed
        creates: example.order.Order
        instance: order_id
        emits: [example.order.OrderPlaced]
        payload:
          example.order.OrderPlaced: {order_id: {generated: true}}
        sets: {region: input.region, priority: input.priority}
views:
  - name: example.order.OrderQueue
    source: example.order.Order
    consistency: read_your_writes
    order_by: [region asc, priority asc]
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: region, type: String}
      - {name: priority, type: Integer}
```

```
$ ess verify conform mutate --path spec --target interpreted --class order-flip
mutation audit of example v1 against interpreted: 2 mutant(s), 1 killed, 1 survived, 0 inconclusive, 0 stillborn, 0 unwitnessed, 0 equivalent (baseline: 1 scenario(s), 0 refusal(s)); 2 of 2 site(s) scored
survived order-flip/example.order.OrderQueue/priority: `priority asc` becomes `priority desc`
killed order-flip/example.order.OrderQueue/region: `region asc` becomes `region desc` — by example.order.PlaceOrder/outcome/placed (1 in total)
exit 1
```

The emitted baseline creates the pair with `{region: "region", priority: 1}` and
`{region: "region-1", priority: 2}` and asserts
`{"expect":"ranked","order_by":["region asc","priority asc"]}`.

Two variants of the same view show the same gap:

- `order_by: [region asc, priority asc, order_id asc]`: the flips of `priority` and of `order_id`
  both survive (3 mutants, 1 killed, 2 survived).
- `order_by: [order_id asc, priority asc]`: the flip of `priority` survives, and nothing can kill
  it, because two rows never tie on the identity.

An authored scenario with a tie on `region` kills the `priority` flip under
`--target interpreted --scenarios`, but `--emit` refuses `--scenarios`, so an implementation audited
through `--emit` and `--collect` keeps one survivor per later key and can never exit 0.

Wanted: for a ranked view with keys k1..kn, the scenario that asserts `ranked` holds, for each key
ki after the first, two rows that are equal on k1..k(i-1) and differ on ki. A chain of n+1 rows does
it: for three keys `(1,1,1)`, `(1,1,2)`, `(1,2,_)`, `(2,_,_)`. The further rows are arranged the way
the second row is today, through the declared creating outcome and the declared moves
(`crates/verify/ess-conformance/src/synthesize.rs:10700-10777`), each sent the earlier keys' values
of the row it ties. The key ki needs no steering where it is the identity: two rows always differ
on it, and `ranked` compares the values the view publishes.

Where a tie for ki cannot be arranged, the scenario stands and synthesis records a named gap for
that view and key beside it, as the head of an ordered selection does for a later key it cannot
steer (`docs/design/ordered-related-head.md:315-318`;
`crates/verify/ess-conformance/src/synthesize/row_set.rs:2206-2212`). The cases: an earlier key that
no declared creation writes from its input (`ordered_head::steerable`,
`crates/verify/ess-conformance/src/synthesize/ordered_head.rs:205-225`), an earlier key that is the
identity or is minted by the target, a key ki that neither is the identity nor is written from an
input, and a tie row the view's filter or owner scope does not show.

## Acceptance

The reproduction and its two variants are committed as fixtures and run in
`cargo test -p ess-conformance --locked --test mutation_audit`.

- On the reproduction above, `ess verify conform mutate --path spec --target interpreted --class
  order-flip`, with no `--scenarios`, prints `2 mutant(s), 2 killed, 0 survived` and exits 0.
- On the same specification, `--emit DIR`, then a run of every emitted suite against the interpreted
  target with its report written beside it, then `--collect DIR`, scores both mutants killed and
  exits 0.
- The emitted baseline scenario `example.order.PlaceOrder/outcome/placed` sends at least two
  `example.order.PlaceOrder` commands with the same `region` and different `priority`, before its
  `query_view`, and still asserts `{"expect":"ranked","order_by":["region asc","priority asc"]}`.
- With `order_by: [region asc, priority asc, order_id asc]` the same command prints `3 mutant(s), 3
  killed, 0 survived`.
- With `order_by: [order_id asc, priority asc]`, `ess verify conform synthesize` lists one refusal
  that stands beside `example.order.PlaceOrder/outcome/placed` and names the view
  `example.order.OrderQueue` and the key `priority`; the scenario is still emitted.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` and
  `--test synthesis_suite_bytes_table_cli_read` pass, and the change's diff of
  `tests/fixtures/synthesis-suite-bytes-table.tsv` touches only rows whose model holds a view with two or more `order_by` keys: a view ranked by one
  key, paged or not, keeps its suite bytes.
- `task test` passes, with the interpreted, Rust, Go and TypeScript conformance runs of the
  committed models it holds.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/ordered_head.rs
- cited: crates/verify/ess-conformance/src/synthesize/paging.rs
- cited: docs/design/mutation-audit-and-model-runner.md
- cited: docs/design/read-api-view-idioms.md
- cited: crates/verify/ess-conformance/tests/mutation_audit.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/mutation-order-tie.yaml
- inferred: website/docs/guides/verify/mutation-audit.md
- inferred: website/docs/reference/diagnostics.md
confidence: medium

Shared surfaces: re-pins the rows of `synthesis-suite-bytes-table.tsv` (read by both pin targets)
for models with a multi-key ranked view, and proposes its `CHANGELOG.md` sentence.

## Fit review

1. **The need, apart from the proposed syntax.** A declared order over several keys is a rule of the
   specification, and the synthesized suite checks only its first key. A target that reverses a
   tie-breaker passes every synthesized scenario. Reproduced on a new minimal specification with
   `ess 0.92.0`, output above. The requester proposes no syntax. Their proposal is a synthesis
   arrangement (a pair equal on the earlier keys, differing on the key) and, where it cannot be
   arranged, the verdict `unwitnessed` in place of `survived`.
2. **Class: defect.** The mutation audit's help says a survivor "is answered by declaring what makes
   the rule observable, or by filing a synthesis gap" (`ess verify conform mutate --help`, 0.92.0;
   `docs/design/mutation-audit-and-model-runner.md:106-109`). Here the view already publishes both
   keys, so nothing can be declared. The design already states the cost of an order as "an
   arrangement of rows that differ only in the order key, and one tie on it, so a target that orders
   the other way or breaks ties otherwise fails" (`docs/design/generic-language-features.md:291-295`),
   and `docs/design/read-api-view-idioms.md:159-161` records that synthesis "arranges no tie".
3. **Can it already be expressed?** Only in part. An authored `ess-scenario/1` with a tie kills the
   mutant under `--target interpreted --scenarios` (second command of the reproduction, 2 killed,
   exit 0). `--emit` refuses `--scenarios` (`error: the argument '--emit <EMIT>' cannot be used with
   '--scenarios <SCENARIOS>'`, exit 2), by design: an authored scenario in a mutant's suite runs
   identically against the unchanged target (`ess verify conform mutate --help`). So no idiom reaches
   an implementation of the adopter's own, and the idiom is not an answer.
4. **Fit.** No authored surface, so no cell of grid R or grid O opens or closes
   (`docs/design/language-grids.md`). The change is on the synthesis side of a construct that
   exists, `order_by:` on a view, and it follows the sibling: the head of an ordered selection
   already arranges, for each further key, a row that ties on the keys before it, and names a gap
   for a key it cannot steer (`docs/design/ordered-related-head.md:310-318`;
   `crates/verify/ess-conformance/src/synthesize/ordered_head.rs:1-19`). It reuses the `ranked`
   expectation unchanged (`crates/verify/ess-conformance/src/runner.rs:3833-3876`), so the Rust, Go
   and TypeScript runners, the interpreter, generated code, `ess verify diff` and the entity runtime
   read nothing new. Paged views keep their four rows (`synthesize/paging.rs:273-279`); the tie rows
   are further companions and count toward `counts: at_least`, which is a floor.
5. **A second adopter.** A support queue, `example.ticket.OpenTickets` with
   `order_by: [severity desc, opened_at asc]`: tickets of one severity are served oldest first. An
   implementation that serves the newest first within a severity passes today's suite.
6. **Cost.** No source format, no suite format (the scenario uses existing steps and the existing
   `ranked` expectation), no new keyword, no migration, no generated-API change, no diff
   classification. Suite bytes move for every model that holds a multi-key ranked view, and those
   rows of the byte tables are re-pinned. The named gap reuses an existing refusal cause where one
   fits (the witness gap the ordered head records, `synthesize/row_set.rs:283-289`); a new
   `ESS-SYNTH-` code is taken only if none does, with its row in
   `website/docs/reference/diagnostics.md`.
7. **Alternatives.** (a) Change nothing: every multi-key view keeps a permanent survivor and the
   emitted audit never exits 0. (b) Let `--emit` carry authored scenarios: refused by the audit's
   own design, since they cannot kill a specification mutant on the unchanged target. (c) Stop
   minting `order-flip` for later keys: the audit then says nothing about a rule the specification
   declares. (d) Arrange the ties in synthesis, and name the keys no tie reaches: chosen. It is the
   requester's proposal, taken; the one change is that the rule covers every key, the first
   included, and the identity as a key.

## Decisions

accept as proposed.

Synthesis arranges, per later key of a ranked view, rows that tie on the earlier keys and differ on
it, and records a standing, named gap per key where it cannot. The verdict of a mutant on such a
key is `story:an-order-flip-on-a-key-no-tie-reaches-is-unwitnessed`. Design page to extend first:
the `order-flip` row and "survivor" paragraph of `docs/design/mutation-audit-and-model-runner.md`,
and the sentence in `docs/design/read-api-view-idioms.md` that says synthesis arranges no tie.

## Source

github:beyond10x/ess#708
