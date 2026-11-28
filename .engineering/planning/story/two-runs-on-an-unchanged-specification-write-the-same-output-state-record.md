---
format: aep.planning-md/3
id: story:two-runs-on-an-unchanged-specification-write-the-same-output-state-record
kind: story
status: draft
title: Two runs on an unchanged specification write the same .ess-output/state.json
summary: 'A determinism defect: the anchor_id of a new output record is a fresh UUID per run, so a committed generated tree differs from a fresh generation in one file (TD-D12)'
tags:
- defect
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:an-idle-output-record-carries-no-machine-path
revision: 3
---
## Defect

Two runs of `ess generate synthesize` on one unchanged specification into two fresh output directories write identical generated
files and a different `.ess-output/state.json`: its `anchor_id` is a new UUID in each run and its `checksum` follows. A consumer
that commits a generated crate and compares it with a fresh generation in its gate has to leave that file out. Gap 6 of a
consumer's request of 2026-11-25.

## Reported case

From a consumer, 2026-11-25. Measured on a 0.99.1 build (0.99.0 plus three synthesis fixes and one UI check fix, none in the
generators), 2026-11-25 03:22 CET, on a copy of the neutral specification of
`story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table`:

```
$ ess generate synthesize --path spec --target rust --layout crate --out out1/crate
$ ess generate synthesize --path spec --target rust --layout crate --out out2/crate
$ diff -rq out1/crate out2/crate
Files out1/crate/.ess-output/state.json and out2/crate/.ess-output/state.json differ
```

`state.json` is format `ess-output-state/4`, producer `ess 0.99.1`; the first run's `anchor_id` is `7a67cce8-7681-4f8a-905f-2e8e7b0b9254`.
`generate synthesize --help` has no `--check` (its `generate types` sibling has one), so the consumer has no ess-side comparison to use
instead. The UUID is minted when an output directory is first initialized: `crates/edge/ess-cli/src/output_ownership/mod.rs:1002`
(`state::new_uuid()` in `initialize`); a transaction carries it (`output_ownership/state.rs:422`).

## Findings covered

- Gap 6 of one consumer's request of 2026-11-25.

## Fit review

1. **Need.** A generated tree that is committed is compared byte for byte with a fresh generation of the same specification; every
   other file of the output passes, one does not.
2. **Class.** A defect against what the sibling story promised: `story:an-idle-output-record-carries-no-machine-path`
   (implemented, beyond10x/ess#484) removed the machine path from an idle record because "adopters commit that file"; the per-run
   UUID is the one remaining per-run field of a committed record. `AGENTS.md` § Determinism and formats asks that generated data keep
   canonical bytes.
3. **Expressible today.** No: the consumer leaves the file out of its comparison, which is what its gate does.
4. **Fit.** `models/output-ownership` declares an Anchor with a UUID; a transaction names it (`docs/design/review-output-ownership.md:65`).
   What the UUID guards, and whether a derived one keeps that guard, is the design question of TD-D12.
5. **Second adopter.** Any repository that commits generated output and compares it with a fresh generation.
6. **Cost.** Option A below keeps the format version `ess-output-state/4` and changes only how a new anchor is minted; B and C add a
   flag or a document rule.
7. **Existing stories.** `story:an-idle-output-record-carries-no-machine-path` borders (implemented): it made the idle record
   machine-free and left the UUID; it does not cover per-run stability. Nothing else names `anchor_id`. Release train: "0.99.x
   patch" if TD-D12 A is taken with no change to `ess-output-state/4`; otherwise next minor.

## Decisions

- TD-D12, a stable record for an unchanged specification. A: a new anchor's UUID is derived from the output's family and the digest
  of its first ledger, so two runs with equal files write equal records; an existing record keeps its anchor. B: the record stays
  per-run and the documented rule is that `.ess-output/` is not part of what a consumer commits or compares, with a `--check` on
  `generate synthesize` as the comparison. C: both A and B. Recommended A. Touches the output-ownership model: architecture: the
  owner's.

- TD-D12: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET (he is away): a new anchor's UUID is derived from the output family and the digest of its first ledger; an existing record keeps its anchor; reversible until the release that carries it.

- TD-D12, option B (not taken) would need a documented rule that `.ess-output/` is not part of what a consumer commits or compares, and a `--check` on `generate synthesize` (which has none on 0.99.1) as the comparison.

## Acceptance

- two_runs_write_equal_records: two runs of the Reported case into two fresh directories leave `diff -rq` empty.
- an_existing_record_keeps_its_anchor: a second run into the first run's directory leaves `anchor_id` as it was.
- a_transaction_still_binds_its_anchor: an interrupted run is recovered only against the record whose `anchor_id` it carries, as
  `state.rs:441` checks today.
- the_state_format_is_unchanged: the record is `ess-output-state/4` before and after, and an old record is read as before.

## Scope

- cited: `crates/edge/ess-cli/src/output_ownership/mod.rs` (`initialize`, 993-1008), `crates/edge/ess-cli/src/output_ownership/state.rs`
  (`new_uuid`, 805; `Transaction`, 422-441)
- inferred: `models/output-ownership/domains/ownership.yaml`, `docs/design/review-output-ownership.md`, `CHANGELOG.md`
