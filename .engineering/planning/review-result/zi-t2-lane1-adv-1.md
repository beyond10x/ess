---
format: aep.planning-md/3
id: review-result:zi-t2-lane1-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie
relations:
- reviews: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie
revision: 1
---
unit: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie, at commit 93ec319d3e (tree zi-t2-adv)
verdict: weak
new failing cases: 2 (adversary_t2_order_tie_pass1.rs::an_identity_last_key_of_a_view_hiding_the_subject_by_a_bound_is_pinned_or_named, ::an_identity_last_key_of_a_view_hiding_the_subject_by_an_enum_is_pinned_or_named)
mutants survived: 1 (order-flip/example.order.OrderQueue/order_id on a view whose filter hides the scenario's subject, with no gap named)

`git status`: two untracked test files and nothing else: `crates/verify/ess-conformance/tests/adversary_t2_order_tie_pass1.rs` and `crates/edge/ess-cli/tests/adversary_t2_order_tie_emit.rs`. No implementation file was touched.

1. **Direct CLI route: holds** (run5-cli.txt; `build.sh` running `cargo run -p ess-cli --bin ess`):
   - Reproduction: `2 mutant(s), 2 killed, 0 survived`, exit 0.
   - Identity-last variant: `3 mutant(s), 3 killed, 0 survived`, exit 0.
   - Identity-first variant: `synthesize` lists one refusal naming `example.order.OrderQueue` and `` `priority` ``, and the scenario is still emitted.
2. **`--emit` / `--collect` route:**
   - It cannot be done with the CLI alone. `ess verify conform run --target interpreted` refuses every mutant suite because the spec_digest does not match. It also refuses the baseline unless `--report-format 2` is given. So `--collect` found no reports (run5-cli.txt). This limit is in the CLI already, not in this commit.
   - Run in-process (the native Runner over the interpreter): `adversary_t2_order_tie_emit.rs` 2 passed, exit 0 (run2-emit.txt). It scores 2 killed and exits 0, scores 3/3 on the identity-last variant, and the baseline still asserts ranked `[region asc, priority asc]`.
   - I kept this file: the commit message says this route was run, but `mutation_audit` has no case for it.
3. **Finding (warning, beside the acceptance bullets, against the story's Decisions):** on a view whose filter does not show the subject (`priority >= 100` or `kind == Normal`) with `order_by: [region, priority, order_id]`, the `order_id` flip survives and no refusal is listed. The CLI shows the same: `1 survived`, and `synthesize` prints `0 refusal(s)`.
   - Cause: when the view hides the subject, the base row is the first shown companion (synthesize.rs:10939). For an identity key, `accept` (synthesize.rs:10982-10990) then matches that base row itself, so `companions.iter().any(accept)` skips the key. No tie row is arranged and no gap is recorded.
   - Fix to name, not applied: make `accept` reject the base row itself.
4. **Note:** the tie gap reuses the held-state heading "leaves a held state it claims unwitnessed" (synthesize.rs:21188), and its path nests backticks: `` `example.order.OrderQueue: `priority`` ``.
5. **What held:**
   - three keys mixing desc and asc, enum first and timestamp second (3/3 killed)
   - an optional earlier key and an optional later key (each killed or named)
   - paged views, 2/2 and 3/3 with the identity last, baseline still passing
   - a key the creator writes from the same input as the first key: named
   - the identity first: exactly 2 standing refusals beside `PlaceOrder/outcome/placed`
   - a view of a later state, identity last: killed
6. **Single-key models:** the bytes-table diff touches only `generated-views.yaml` (`[owner desc, kind]`), `read-api-view-idioms.example` (`[at desc, event_id asc]`) and the three new fixtures. Every other model in the table that lists more than one `order_by` key does so on an ordered related head, not on a view. Holds.
7. **Suite run** (run5-cli.txt, `--no-fail-fast`): `mutation_audit` 16 passed; my file 8 passed and 2 failed, exit 101. Cases executed went from 16 to 28.
8. **Budget:** 5 `build.sh` runs. Run 3 built nothing (`cargo run` needed `--bin`). Run 4 stopped at my red test binary before `mutation_audit` ran.
9. **Written outside the worktree:**
   - In `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie/adv/`: `go.sh`, `cli.sh`, `run1-cases.txt`, `run2-emit.txt`, `run3-cli-nobin.txt`, `run4-suite.txt`, `run5-cli.txt` (each with its `.touch`/`.wait` files where present), `emit-repro/`, `filtered/`, `tmp/`.
   - The emit test's directories under `CARGO_TARGET_TMPDIR`: `.../ess/targets/B/debug/...` (named `adversary-t2-order-tie/`).

```findings
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 10982
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: for an identity key on a view that hides the subject, accept matches the base companion itself, so no tie row is arranged, no gap is named, and the identity's order-flip survives silently
- file: crates/edge/ess-cli/src/main.rs
  line: 4364
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: the acceptance's emitted route cannot be run through the CLI because run --target interpreted refuses every mutant suite on spec_digest, so only an in-process runner can answer it
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 21188
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the tie gap is printed under the held-state heading and its path nests backticks
```
