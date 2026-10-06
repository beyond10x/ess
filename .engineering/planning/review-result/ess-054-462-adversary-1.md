---
format: aep.planning-md/3
id: review-result:ess-054-462-adversary-1
kind: review-result
status: active
title: 'Adversary pass 1, #462 row-set upsert answer'
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-462
revision: 1
---
unit: ess-054-462-adv, uncommitted working tree of `~/.local/state/worktree/trees/b10x/ess/ess-054-462-20261006` (base `09ec1dc33`)
verdict: CONFIRMED (1 finding, warning)
cases: executed 8→16, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: decide whether the synthesis defect in finding 1 goes back to this unit or gets its own story

**Summary:** The interpreter change held against every attack. One case is red: for an upsert whose creation is chosen by an input guard, ESS writes a suite that no honest target can pass. The cause is in synthesis, which this unit did not touch.

**1. Diff stat**
```
 .../ess-conformance/src/interpret/execute.rs       | 32 ++++++++++++++++++++--
 docs/design/filtered-related-reads.md              |  7 +++--
 2 files changed, 35 insertions(+), 4 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_row_set_upsert_462.rs   (mine, the only file I added)
```
Both tracked changes are the implementor's. I changed no production file.

**2. Cases added** in `crates/verify/ess-conformance/tests/adversary_row_set_upsert_462.rs`. Each one builds a variant of `row-set-upsert.yaml`.

| case | asserts | now |
|---|---|---|
| `adv_input_guarded_upsert_suite_passes_interpreter` | `Shelve` has an `unknown-book` branch and creates under `when: mode == "new"` instead of `external:`. The synthesized suite passes on the interpreter | **red** |
| `adv_input_guarded_upsert_suite_passes_interpreter_without_row_set` | the same model without the row set passes its own suite | green |
| `adv_unreachable_creation_does_not_claim_absent_row` | absent book, `mode: old`, decoy library only: `unknown-book` with and without the row set | green |
| `adv_creation_from_other_field_keeps_unknown_instance` | the creation takes `input.copy`, and the absent `book` still gets `unknown-book` | green |
| `adv_wrong_state_beside_upsert_absent_book` | `moves` default beside `wrong_state`: absent book gets `added` with a decoy, `refused` with a match, and the suite passes | green |
| `adv_empty_row_set_beside_upsert` | `exists: false` row set: `refused` with no matching library, `added` with one, and the suite passes | green |
| `adv_unforced_absent_book_matches_unguarded` | no forced external: guarded and unguarded give the same answer | green |
| `adv_go_parity_on_upsert_suite_with_faulty_targets` | Go runtime matches Rust on the honest target and both faulty targets; the faulty ones fail `refused` and `replaced` | green |

Red output, from the first run of the file on its own (`cargo test -p ess-conformance --test adversary_row_set_upsert_462`, exit 101; the line number moved after rustfmt):
```
panicked at .../adversary_row_set_upsert_462.rs:216:5:
assertion `left == right` failed: every scenario passes: {
    "demo.shelf.Shelve/outcome/added": Error,
    ... every other scenario Passed ...
}
  left: 1
 right: 0
```
Diagnostic from re-running the case alone (`adv-run3.log`):
```
observed: "invoking `demo.shelf.Shelve` failed: the request is not what the model declares: no branch the model allows is described by the given values — `demo.shelf.Shelve/added`: a supplied identity is already held by `demo.shelf.Book`, and creation never replaces it"
```
The steps of that scenario:
1. `Shelve{book, mode: new}` expects `added`.
2. `OpenLibrary` opens the decoy library.
3. `Shelve` with the same `book` and `mode: new` expects `added` again.

So the suite arranges the book first and then expects it to be created a second time.

**3. Suite run** (after the cases existed): `cargo test -p ess-conformance --no-fail-fast --test row_set_upsert --test adversary_row_set_upsert_462 --locked --offline`, exit 101.
- `adversary_row_set_upsert_462`: 7 passed, 1 failed.
- `row_set_upsert`: 8 passed, 0 failed.

The "before" count of 8 is `row_set_upsert` in that same run, with my file excluded as a separate binary.

**4. Findings** (they cover the working tree above)

| file:line | verdict | origin | finding |
|---|---|---|---|
| `crates/verify/ess-conformance/tests/adversary_row_set_upsert_462.rs:244` | CONFIRMED | undecided | For an `ess/23` upsert whose creation is chosen by an input guard beside a row set, the `added` scenario arranges the book and then expects it created again. The interpreter is right to refuse that. |

- **What was measured:** the assertion above fails with 1 scenario not passed (`Shelve/outcome/added`, Error). Without the row set, the same model passes its own suite.
- **What reaches it:** any authored `ess/23` model of this shape. It validates and synthesizes without a refusal. No model in this repository uses the shape; the only in-repo upsert with a row set is the unit's own fixture.
- **Where the defect is:** synthesis, not the unit's interpreter change.
  - I did not trace the mechanism. My guess is that the row-set witness arrangement creates the addressed row for every branch.
  - Nothing in `synthesize*` is in the diff, and synthesis does not import `interpret`. So the suite bytes should be the same at the base. I did not run it there, so the origin stays undecided.

**5. Attacked and could not break**
- Creation of a different entity, or from a different input field: still `unknown_instance`.
- An unreachable creation (input guard not met): still `unknown-book`, with or without the row set.
- `wrong_state` beside the upsert: an absent book never reaches the state refusal.
- `exists: false` row set and matching-library precedence: the row set answers first and creates no book.
- Unforced external: same verdict with and without the row set.
- Go runtime against the honest and faulty targets: matches Rust.
- `when_subject_state` beside a row set: validation refuses the model (`unsupported_construct`), so nobody can reach it. I deleted that case of mine.
- Not attacked:
  - the TypeScript runner (its harness is per-fixture, so there is no general parity helper);
  - `existing_instance:` beside the row set (the diff does not touch `existence::existing`);
  - `InputOrGenerated` identities;
  - mutating a copy of the code to probe the `selected_subject_refusal` branch (I ran out of time).

**6. Paths written outside the worktree**
- `~/.cache/ess-054-462/adv/`: `adv-run1.log` to `adv-run4.log`, `suite.log`, and an empty `tmp/` (48K in total, left for you).
- `/dev/shm/ess-054/ess-054-462-adv`: build dir, cleaned with `cargo clean` (434.5MiB removed).
- `~/.cache/b10x-go-cache/ess-054-462-adv`: Go cache, removed.

**7. Findings block**
```findings
- file: crates/verify/ess-conformance/tests/adversary_row_set_upsert_462.rs
  line: 244
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: an ess/23 upsert whose creation is input-guarded beside a row set synthesizes an `added` scenario that arranges the book and then expects its creation again, so no honest target passes it
```
