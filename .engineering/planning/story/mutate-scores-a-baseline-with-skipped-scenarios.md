---
format: aep.planning-md/3
id: story:mutate-scores-a-baseline-with-skipped-scenarios
kind: story
status: active
title: mutate --collect refuses to score when the baseline has skipped scenarios
refs:
- provider: github
  reference: beyond10x/ess#210
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:50Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:50Z", actor: "human:timo", revision: 3}
---
# Story: mutate --collect refuses to score when the baseline has skipped scenarios

## Why

beyond10x/ess#210. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #210 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

Issue #210 is open: `mutate --collect` refuses to score when the baseline has skipped scenarios. The defect is still in the 0.40.0 candidate tree (`e932781965`). I found it by reading the code; I did not build or run anything.

**Does it reproduce, and is it fixed already?**
- cited: `collect` in `crates/verify/ess-conformance/src/mutate.rs:1755`. At lines 1761-1766 it returns `AuditRefusal::BaselineFailed` (`ESS-MUTATE-001`) whenever `baseline.not_passed` is non-empty.
- cited: `score` at `mutate.rs:1646` fills `not_passed` with every status that is not a pass:
  - a standalone report maps `failed`, `unsupported` and anything else to `Error` (1690-1695);
  - a count report (`/2`) maps `skipped` to `Status::Error` (1709).
  So skipped, unsupported and error all count as a red baseline. The defect reproduces, and it is not fixed.
- cited: the in-process `audit` has the same refusal (`mutate.rs:1335-1347`, filter `!= Status::Passed`). The issue only names `--collect`, but `--target` refuses the same way.
- cited: the docs describe the current behaviour:
  - `website/docs/reference/cli.md:335`
  - `website/docs/reference/formats.md:319` ("`ESS-MUTATE-001`, the unmutated suite did not pass")
  - `website/docs/guides/verify-conformance.md:310`

**Where the fix lands**
- cited: in `collect` (1755-1845), the baseline gate becomes "refuse only on a baseline `Failed`". Each mutant's `scored.not_passed` is then filtered to the ids the baseline executed before `Verdict::classify` runs (1802-1806).
- cited: `Scored` and `score` (1638-1744) have to keep "skipped" separate from "error". Today a Go `skipped` becomes `Error`, and that information is lost.
- cited: `MutationReport` and `SuiteSize` (1044-1086) need somewhere to put the list of scenarios that were not scored, with a reason and a count. `render_text` (1098) has to print them.
- inferred: `audit` (1317-1375) gets the same change so that `--target` and `--collect` agree. `MutateCode` documentation (134-137) and the `--collect` help in `crates/edge/ess-cli/src/main.rs:602` need wording updates.
- inferred: new tests go in `crates/verify/ess-conformance/tests/mutation_external.rs` and `mutation_external_adversary.rs`. Adversary test 170 already pins the rule "skipped is inconclusive, not killed" for mutants.

**Collisions with the other issues**
- inferred: #209, #196, #198, #199, #201 and #195 are all about synthesis or the model. None of them should touch `mutate.rs`, so I expect no file collision.
- inferred: #209 has a runtime interaction. It turns skipped refusal scenarios into executed ones, so fixing it shrinks #210's trigger. Neither fix blocks the other.
- cited, outside the list: #203 also changes `collect`'s mutant scoring and `refusals` handling in `mutate.rs`. It is a direct collision, so the two should land in one unit or be serialized.

**Decisions the implementor has to make**
1. Which baseline statuses count as red.
   - Only `Failed`, or also `Error` (target could not answer)?
   - The issue says "skipped (unsupported capability or unreached refusal)". Today `Error` and a Go `skipped` share one variant.
2. Whether this is a report format change.
   - A new "not scored" field in `ess-mutation-report/1`: additive, or a bump to `/2`?
   - Does `--target` output have to stay byte-identical (formats.md:319 promises bytes for `--collect`-only fields)?
3. How to match scenarios across suites.
   - A mutant suite is re-synthesized, so its scenario ids can differ from the baseline's.
   - A mutant scenario that is absent from the baseline's executed set: excluded or scored?
   - This overlaps #203.
4. Whether `audit` (`--target`) gets the same relaxation. The issue names only `--collect`.
5. Exit status (docs line 310). What happens when every baseline scenario is skipped, so nothing is scored?

Confidence: high on the reproduction (read directly in the code), medium on where the fix lands, low on the collisions (from issue titles and bodies, not code).

Paths:
- crates/verify/ess-conformance/src/mutate.rs
- crates/edge/ess-cli/src/main.rs (inferred)
- crates/verify/ess-conformance/tests/mutation_external.rs (inferred)
- crates/verify/ess-conformance/tests/mutation_external_adversary.rs (inferred)
- website/docs/reference/formats.md
- website/docs/reference/cli.md
- website/docs/guides/verify-conformance.md

Verdict: needs-design. The defect is open and reproduces, but decisions 1 to 3 have to be settled first, and it should be sequenced with #203.
