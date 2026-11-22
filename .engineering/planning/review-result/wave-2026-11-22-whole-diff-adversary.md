---
format: aep.planning-md/3
id: review-result:wave-2026-11-22-whole-diff-adversary
kind: review-result
status: active
title: Adversary review of the whole wave of 2026-11-22
relations:
- reviews: story:each-code-target-admits-binary64-through-a-switch-of-its-own
- reviews: story:the-rust-target-carries-a-finite-binary64
- reviews: story:the-go-target-carries-a-finite-binary64
- reviews: story:the-web-target-carries-a-finite-binary64
- reviews: story:a-creating-branch-sends-every-row-it-creates-under-its-own-identity
- reviews: story:a-row-set-refusal-is-witnessed-beside-the-related-row-its-creator-needs
- reviews: story:a-creator-reading-several-related-rows-arranges-its-row-for-other-scenarios
- reviews: story:validation-refuses-an-ordering-comparison-over-an-enum
- reviews: story:validation-refuses-an-exists-false-branch-declared-below-a-refusal-it-answers-before
revision: 1
---
unit: wave 2026-11-22, leads 1–4; findings cover a3e75dfd8d plus my test commit a223907510
verdict: NEEDS-CHANGE (F1 S3 unit, introduced)
cases: executed 3→4 (enum_variant_refusals), red 1
origin: introduced 1 / pre-existing 3 / undecided 0
wrote-outside-worktree: 1 path, the scratch dir `wave-1122/close/adv/`
needs-coordinator: whether F2–F4 become backlog drafts

1. diff --stat a3e75dfd8d..HEAD: `crates/specify/ess-domain/tests/enum_variant_refusals.rs | 73 +` (test file only)

**Findings**
- **F1 S2 S3 (introduced)** `crates/verify/ess-conformance/src/synthesize/related_guard.rs:2911` `drive_several` → `as_own_scenario`: the cost grows exponentially with the depth of an ownership chain. The only bound is "this branch is not already open" (`own_opening`). The fallback's result is never memoised, and `open_key()` now includes the open own-scenarios, so the existing memos stop sharing across them.
  - Measured with `scale.sh` and `chain.sh`, the S3 fixture deepened to n levels, run on `ess-s3`. User CPU for depth 1 to 4: 0.035 / 0.236 / 2.09 / 33.2 s. At depth 5 it was killed at 540 s after 274 s of CPU, unfinished.
  - 0.97.4 takes under 0.08 s at every depth, but it refuses 1 / 6 / 11 / 16 / 21 branches.
  - Each level is about 15 times the one before, which is where the brief's "5 to 7 times" on one level comes from.
  - Not run as a test: a debug build would pass 60 s by depth 4. Fix direction: memoise `as_own_scenario` per (ir, command, outcome, distinction, bound) and keep the open-own stack out of the memo key.
- **F2 S2 VR-ENUM (pre-existing)** `crates/specify/ess-domain/src/expression/attributes.rs:225` (`atom_read`): `input.plan.level > low`, where `level` is an enum-typed attribute of an enum, validates. The guard compiles to `predicate: false`, so the branch is never taken. This breaks the changelog claim "every predicate site". 0.97.4 validates it too.
  - Red test: `an_ordering_over_an_enum_typed_attribute_is_refused_naming_the_enum`, output in `adv/t1.log`.
- **F3 S1 S (pre-existing)**, read and run, no test: `m/caller.yaml` is the S3 fixture plus an actor attribute and a first refusal `when: site_id != caller.site_id` on IssueGrant.
  - The synthesized scenario `IssueGrant/outcome/no-desk` fails against `--target interpreted`: observed `foreign`, expected `no-desk`.
  - The suite bytes are identical on 0.97.4.
- **F4 S2 S2+S3 (pre-existing)**, read and run: `m/combo.yaml` is the S3 fixture plus a row-set refusal `busy` over Grants (`desk_id == input.desk_id`), whose creator reads two related rows.
  - The branch is refused as ESS-SYNTH-001 on both binaries. The S3 fallback never reaches S2's `linked` path.
- **F5 S3 VR-BRANCH**, read, not run (lead 7): the code comment says the refusal applies to a stored-field via at any format, while `CHANGELOG.md` says "from `ess/27`".

**Per group**
- **B:** no finding. The Go and Rust `Token()`/`Decimal()` algorithms match, NaN and infinity are refused at the same parse, and without the switch nothing changes.
- **F:** not reviewed (lead 5–7 reviewer).
- **VR:** F2, F5. VR-BRANCH: no false positive with disjoint input guards; 0.97.4 already refuses `when:` beside `exists: false`.
- **S:** F1, F3, F4.
- **FX, the rest:** not reviewed.

**Could not break**
- Determinism: 3 runs of 7 models gave identical suite bytes.
- All S1, S2 and S3 fixtures and chain 3 pass against the interpreter.
- S1's retry is bounded at 4.

**Build:** one build, `cargo test -p ess-domain --test enum_variant_refusals`, 151 s wall; the test step itself took 0.06 s. The cache is discarded. Commit a223907510 is on review/w1122-adv.

**Not reached:** Go/Rust/web servers built and run on the edge values; the S1 `view_expectations` widening; F/FX/rest.

```findings
- file: crates/verify/ess-conformance/src/synthesize/related_guard.rs
  line: 2911
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the own-scenario fallback costs about 15 times more per level of ownership chain and does not finish depth 5 in 540 s"
- file: crates/specify/ess-domain/src/expression/attributes.rs
  line: 225
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "an ordering over an enum-typed enum attribute validates and lowers to false instead of being refused"
- file: crates/verify/ess-conformance/src/synthesize/related_guard.rs
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "with a caller-attribute refusal declared first, the synthesized no-desk scenario fails against the interpreter"
- file: crates/verify/ess-conformance/src/synthesize/row_set.rs
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "a row-set refusal over rows whose creator reads two related rows stays ESS-SYNTH-001"
- file: CHANGELOG.md
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: undecided
  message: "the changelog says the exists-false order refusal starts at ess/27 but the code applies it to stored-field vias at any format, not run"
```
