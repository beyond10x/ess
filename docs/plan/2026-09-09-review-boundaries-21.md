# Review boundaries 21 — the last three remediation stories

## Selection and authority

Skill version 0.8.1 (`aep-drive` wave; `aep` protocol 0.54.0). N=3, all serving vision:O2 and
decomposing epic:review-boundary-remediation:

| unit | story | tags | scope confidence |
|---|---|---|---|
| U1 | implementor d107b53; adversary pass 1 red 5; correction 1 33f4568; adversary pass 2 red 5 (1 pre-existing → story:primitive-canonical-serialization), ledger 0/5/5; correction 2 green (1094→1100) 12ec9f1, coordinator-verified, review_outcome fixed ×2; accounting decision A taken: consumer-coverage design clause + baseline re-freeze 70ba641, consumer-check exit 0 (BaselineUnknown 157677, Supported 54); **merged** into wave/review-boundaries-21 at b3427b1 | 70ba641 |
| U2 | implementor 36fa1df; adversary pass 1 red 6; correction 1 1b0368b; adversary pass 2 red 8, ledger 0/8/6; correction 2 green (520→531; 322,741 tokens, 175 tool uses, 23.0 min) ba6cb7b, coordinator-verified (no assert removed, one pin tightened, adversary files untouched), review_outcome fixed ×2; **merged** into wave/review-boundaries-21 at 0175113 | ba6cb7b |
| U3 | 67d9e95, 4f9dfba, 6a8aff7; adversary pass 1 red 4 (fixed), pass 2 red 4 (review-result:execution-recovery-adversary-wave21-pass2), ledger 0/4/4; correction 3 green (646→650; 742,098 tokens, 31 tool uses, 44.6 min) c2d8cc1, coordinator-verified (four adversary cases intact, removed assertions replaced or justified), review_outcome fixed ×2; **merged** into wave/review-boundaries-21 at d81245f | c2d8cc1 |

This is the whole remaining draft set of the epic: 28 stories are `implemented`,
story:fuzz-the-specification-surface closes in the commit preceding this wave's opening commit,
and nothing else under the epic is `draft` or `proposed` (`aep artifact list`, 2026-09-09).
Nothing is blocked (`aep artifact blocked`: "nothing is blocked"); the one decision-blocker in
the store is `cleared`. U1 depends on story:review-format-catalog (`implemented`); U3 depends on
story:review-execution-recovery-design (`implemented`). obligation:review-contract-rollout-coordination
is open and lists F08 (U1) as a migration candidate; U1 is therefore bounded to its byte-preserving
stage (story body, "Implementation pointers — 2026-09-09").

Base: ESS `main` at `9845922dbf2f8047d533eedf35fbe13d6d234079`, which holds PR #17 (`d50b1b4`, `954fb43`, merge `725549b`) and the
fuzz-story closure commit `9845922`. The wave-20 trees were retired on 2026-09-09 before this
commit (gc applied to five managed ids; two foreign trees left standing).

Approval of this page authorizes exactly these commits and nothing else: the closure commit for
story:fuzz-the-specification-surface (store evidence and move); the opening `chore(store)` commit
(this page, the three stories moved to `active`); one or more source commits per unit on its
`impl/` branch, including correction rounds; the merges of each green unit into
`wave/review-boundaries-21`; the closing `chore(store)` commit (evidence, moves, scope
write-back); the merge of the integration branch into `main` through a bot pull request. No
release tag, no version bump, no Website or Atlas work, no other repository.

## The verb's three lists

`aep artifact waves --format json` at the current store (all draft and proposed stories, 116
collision rows in total; the rows that name a unit of this wave are retained verbatim in
[the opening selection evidence](../reviews/2026-09-09-review-boundaries-21-opening-waves.md)):

- placement: `story:review-typed-diagnostics` wave 4 (inferred=true, 16 entries);
  `story:review-execution-recovery-implementation` wave 11 (inferred=true, 34 entries);
  `story:review-primitive-semantics` wave 13 (inferred=true, 21 entries). The verb orders the
  whole draft pool; the three land in different verb-waves only because of collisions with drafts
  outside this epic, which are not selected.
- collisions between any two of the three: exactly one —
  `{"a":"story:review-execution-recovery-implementation","b":"story:review-primitive-semantics","path":"Cargo.lock","confidence":"inferred"}`.
  Accepted: the lockfile is coordinator-owned at merge and re-resolved with `cargo update -w`
  semantics on the integration branch; each unit names any dependency it added in its report.
- unassessed (never placed, none selected): `story:authored-scenarios`, `story:own-planning-store`,
  `story:relations-design-page`, `story:relations-in-the-billing-example`,
  `story:relations-in-the-domain-model`, `story:relations-projected`,
  `story:unique-wire-field-identity`.
- cycles: `[]`.

Selection path: the verb, over typed entries the coordinator rewrote at file granularity for the
two units that share the `ess-primitives` and `ess-domain` crates. The verb and the coordinator's
reading agree; the crate-level entries the store carried until 2026-09-09 would have reported
U1/U2 as colliding, and the file-level split is what makes N=3 possible. Both stories' `## Scope`
sections name which files are the other unit's.

Deliberately left out: nothing from the epic. The seven unassessed stories and the other drafts
are outside this remediation wave.

## Accepted contract per unit

- **U1** — first commit is `docs/design/review-primitive-semantics.md` (the matrix per
  `Primitive` variant, the exact representation, the preserved spellings, the corpus location).
  Persisted bytes do not move in this wave: witness `1.0` spelling, report prose, schema
  `format`/`pattern` nodes, generated Rust/Go text. `cargo xtask generate --check`,
  `cargo xtask schema --check` and `task consumer-check` are the checks that notice. Not U1's:
  `ess-primitives/src/error.rs`, `ess-primitives/src/lib.rs`, every `ess-domain` file except
  `expression.rs` and `primitive_admission.rs`, `ess-compiler`.
- **U2** — first commit is `docs/design/review-typed-diagnostics.md` (the site type, the
  `location` rendering rule, the migration order, the inventory of paths left on the heuristic).
  `location: String` stays rendered so the 48 `.location` assertions and the guide sample keep
  passing; `bridge` reads the site when present and falls through otherwise. Not U2's:
  `ess-primitives/src/facts.rs`, `node.rs`, `predicate.rs`, `lib.rs`; `ess-domain/src/expression.rs`,
  `primitive_admission.rs`.
- **U3** — the accepted binding `docs/design/review-execution-recovery.md` (SHA256 9699f5ed…)
  and the R01–R29 matrix, unchanged; the layout decision (`[lib]` in `ess-cli` or a test-support
  driver binary) is the implementor's and is named in its report. This unit is the largest by an
  order of magnitude (34 paths, 29 fault families, 12 acceptance clauses); it may need more than
  one correction round, and if it is red after two full attacks it leaves the wave and keeps its
  branch. U1 and U2 do not depend on it.

Each unit writes source, tests and its own design page only; nothing under
`.engineering/planning/`; package-scoped gates; test-first with the red run in the report. The
adversary attacks each unit's design page and code together; no separate binding round is
scheduled. Model: Opus for implementors and adversaries (operator instruction, 2026-09-09).

## Managed work and resources

| unit | managed id | branch | build directory | scratch |
|---|---|---|---|---|
| coordinator | ess-review-boundaries-21 | wave/review-boundaries-21 | `<tree>/target` | `<tree>/target/review-boundaries-21` |
| U1 | ess-primitive-semantics-wave21 | impl/review-primitive-semantics | `<tree>/target` | `<tree>/target/review-boundaries-21/scratch` |
| U2 | ess-typed-diagnostics-wave21 | impl/review-typed-diagnostics | `<tree>/target` | `<tree>/target/review-boundaries-21/scratch` |
| U3 | ess-execution-recovery-wave21 | impl/review-execution-recovery-implementation | `<tree>/target` | `<tree>/target/review-boundaries-21/scratch` |

`<tree>` is the path `worktree repo list --repo <primary>` prints for the managed id; absolute
workstation paths are not recorded in tracked files (PR #16). Each unit's short `TMPDIR` is
`~/.cache/e21-tmp/<managed id>`, created empty by the coordinator (the authored-scenarios socket
fixture needs a short path on the checkout's filesystem; wave-20 record). Every tree builds into
its own `target`; `CARGO_TARGET_DIR` is never set; `sccache` is the configured `rustc-wrapper`
(`~/.cargo/config.toml:60`) and is left on for units, blank for the consumer-check lane as the
Taskfile already does.

Pre-flight measured 2026-09-09 (before wave-20 teardown): 35,502,329,856 bytes available on the
checkout filesystem (`df -B1`), 42,598,336 kB `MemAvailable`, 20 CPUs. Build output on disk:
wave-20 implementor tree 21,905,657,856 bytes (`worktree inspect`; its root `target` alone
19,918,102,528 bytes, which is the measured cost of one full `task check` including
consumer-coverage), primary checkout `target` 6,378,802,981 bytes, closure tree
1,662,532,207 bytes after `fuzz-check` and `consumer-check`. Floor: 8,589,934,592 bytes, as in
wave 20. N=3 fits only after the wave-20 trees are retired (about 57 GB free), with units gating
package-scoped and their build directories removed before the integration gate runs. Free space
is re-read when each unit returns.

Model budget: no number was given; N=3 is the operator's stated set. If HTTP 429 kills an
agent, its branch head and stage are written here and the unit is resumed, never re-dispatched.

## Disk during the wave

Re-read at each unit return. 2026-09-09 after both correction rounds: 11,086,360,576 bytes free against the 8,589,934,592 floor; unit build directories measured 15,410,388,430 (U1), 7,339,739,652 (U2), 11,525,699,721 (U3); `~/.cache/e21-tmp` 4,458,323,184; `~/.cache/sccache` at its 10 GB cap. Freed: the primary checkout's stale `target` (6,378,802,981 bytes, not a wave tree) and all but the three newest `target/consumer-coverage/run-*` directories in U1 (35 runs, 2,403,931,242 bytes) and U2 (5 runs). Unit build directories are removed after each merge and before the integration gate runs.

## Pre-flight refusals to clear before dispatch

1. PR #17 green and merged; story:fuzz-the-specification-surface `implemented` on the closure
   commit.
2. Wave-20 trees retired: ess-specification-fuzzing-wave20, ess-fuzz-source-audit,
   ess-review-boundaries-20, ess-review-boundaries-20-path-correction, ess-fuzz-story-closure.
   Three hold live session leases of the codex session that ran wave 20; the coordinator does not
   clear another session's lease, so that session must exit first. The two uncommitted edits to
   `docs/plan/2026-09-08-review-boundaries-20.md` in the coordinator trees are folded into the
   closure commit before the trees go.
3. `wt-aecaec01c0e9` (dag-source-gates-20260908) and `wt-c465ed5c6cd3`
   (eventlog-wave-published-gate-input) are other sessions' clean detached trees; they are left
   standing and named here.

## Integration checkpoints

After U2 (0175113) and U1 (b3427b1) merged, the six package lanes of both units ran together on the integration branch (`target/review-boundaries-21/integ/*.exit`): fmt 0, clippy 0, test 0 (1215 passed, 0 failed across ess-primitives, ess-domain, ess-compiler, ess-conformance, ess-gen, ess-synth), `cargo xtask generate --check` 0, `cargo xtask schema --check` 0, `task consumer-check` 0 (BaselineUnknown 157677, Supported 54, Refused 0). The whole gate runs once after U3.

## Whole gate, run 1 (integration branch at d81245f)

Per-lane exit codes (`target/review-boundaries-21/gate/*.exit`): fmt-check 0, clippy 0, **test 201**, **doc-check 201**, example-check 0, projection-check 0, support-check 0, **consumer-check 201**, fuzz-check 0 (25 replay cases), release-check 0, action-check 0. No lane skipped itself. `test` stopped at its first failing target (`cargo test --workspace --locked` has no `--no-fail-fast`): 22 of the workspace's lanes ran, 160 passed, 1 failed — `crates/edge/ess-cli/tests/coverage_browser.rs:336`, coverage case `predicate-equivalent-12` (the pair 2^53 / 2^53+1 listed as an equivalence at `tests/support/coverage_cases.rs:146-149`) now refused by the exact `Number`; routed to U1's implementor as correction 3. doc-check: two public doc links to private items (facts.rs:475, recovery/chart.rs:401), fixed by the coordinator. consumer-check: `ess_cli::bin(ess)::fn::parse_invocation` unclassified, classified by the coordinator. The gate runs again in full after the U1 correction merges.

## Integration corrections before whole-gate run 2

08ee01a (rustdoc private links; parse_invocation classified); 0bdb9f5 merges U1 correction 3 (154d269: the coverage lineage pinned the binary64 collapse in two places, `coverage_cases.rs:146` and the `finite-node-{owner}` cases; the browser adapter now compares integer tokens by digits; corpus 2^53 vectors); 4e63718 and 144aa95 reconcile `entry-classifications.json` against the extraction in both directions — 57 ids moved from `bin(ess)` to `lib(ess_cli)`, 413 new lib entries, 212 test-only bin entries (FixtureRealization), 25 kubernetes recovery entries (ForeignContext), 58 stale ids removed; 8,630 entries. 144aa95's message overstates the total by one and names the class loosely; a git note on the commit corrects it. Whole gate run 2 starts at 144aa95.

## Whole gate, run 2 (integration branch at 144aa95)

Per-lane exit codes (`target/review-boundaries-21/gate-run2/*.exit`): fmt-check 0, clippy 0, **test 201**, doc-check 0, example-check 0, projection-check 0, support-check 0, **consumer-check 201**, fuzz-check 0 (25 replay cases), release-check 0, action-check 0. `test` stopped at `crates/edge/ess-cli/tests/coverage_lineage.rs:162`: the generated Go runtime's `TestOriginalLineage` admits the five collapse cases round 3 added (`admitted=true want=false`), the third lane of the same class — routed to U1's implementor as correction 4. `consumer-check`: `missing bound entry cache-acquisition ess_cli::bin(ess)::oci_cache::fn::acquire` — the profile named the id U3 moved into the lib; fixed by the coordinator at d5f0848; `task consumer-check` at d5f0848 exit 0 (BaselineUnknown 157677, Supported 54, Refused 0; an intermediate attempt refused `complete source changed during extraction` because the coordinator edited this page in the tree mid-run — no writes to the integration tree during a run from here on). The earlier no-fail-fast run at 08ee01a (`gate2/test-nff.log`, 217 lanes, 2558 passed, 3 failed) had found the same Go lane plus the browser lineage case (fixed in correction 3) and a corpus self-check racing my own mid-run merge.

## Whole gate, run 3 (integration branch at f346f27)

Per-lane exit codes (`target/review-boundaries-21/gate-run3/*.exit`): fmt-check 0, clippy 0, **test 201**, doc-check 0, example-check 0, projection-check 0, support-check 0, consumer-check 0 (BaselineUnknown 157677, Supported 54), fuzz-check 0, release-check 0, action-check 0. The `--no-fail-fast` enumeration that followed (`gate-run3/test-nff.log`): 217 lanes, 2558 passed, 3 failed, all in `ess-cli` browser tests — `coverage_writer_adversary_pass2.rs:124`, `replay_fidelity_browser.rs:1197` and `:666` — one class: round 3's browser-adapter change reinterprets numeric-looking JSON *strings* as number tokens and projects them as `{raw}`. Routed to U1's implementor as correction 5 with the whole `ess-cli` package as its gate.

## Whole gate, run 4 (integration branch at e448671e) — the closing record

| lane | exit | evidence |
|---|---|---|
| fmt-check | 0 | `gate-run4/fmt-check.exit` |
| clippy | 0 | |
| test | 0 | 217 lanes, 2561 passed, 0 failed, 0 ignored (`cargo test --workspace --locked`) |
| doc-check | 0 | |
| example-check | 0 | |
| projection-check | 0 | `generate --check` and `schema --check` current |
| support-check | 0 | |
| consumer-check | 0 | BaselineUnknown 157677, Supported 54, Refused 0, 22 executed cases |
| fuzz-check | 0 | 25 replay cases |
| release-check | 0 | release 0.20.0: workspace version and changelog agree |
| action-check | 0 | |

No lane skipped itself (the word appears only in two test names). Runs 1–3 (d81245f, 144aa95, f346f27) are recorded above with their red lanes and the correction each produced. `task site-build` at e448671e: exit 0 (`gate-run4/site-build.exit`; site-lab WASM smoke and `_run.test.mjs` passed, Docusaurus generated static files).

## What the wave cost

Harness-reported figures per agent; a resumed agent's token figure is cumulative across its rounds (inferred from the monotone series), tool uses and wall time are per round and summed here.

| agent | rounds | tokens | tool uses | wall |
|---|---|---|---|---|
| U1 implementor (Opus) | 6 | 490,314 | 874 | 169.8 min |
| U1 adversary pass 1 / pass 2 | 2 | 117,737 / 136,921 | 42 / 55 | 12.3 / 13.4 min |
| U2 implementor (Opus) | 3 | 322,741 | 392 | 69.5 min |
| U2 adversary pass 1 / pass 2 | 2 | 135,367 / 160,136 | 47 / 52 | 23.3 / 14.0 min |
| U3 implementor (Opus) | 4 | 742,098 | 608 | 293.4 min (one HTTP 429 after round 1's report) |
| U3 adversary pass 1 / pass 2 | 2 | 152,100 / 135,464 | 64 / 58 | 33.0 / 37.6 min |
| story-scopers ×3 (stage 1) | 1 each | 59,268 / 44,968 / 52,034 | 27 / 26 / 8 | 1.0 / 0.8 / 2.2 min (one HTTP 429 each before resuming) |
| **total sub-agent tokens** | | **2,549,148** | | |

Executed-case counts per unit at merge: U1 1076 → 1100 (package), U2 502 → 531, U3 527 → 650; whole workspace 2561. Adversary findings per pass: U1 5 → 5, U2 6 → 8, U3 4 → 4; ledger between passes carried 0 for every unit.

## Commits the wave made

Opening 2900f62; units — U1 d107b53, 33f4568, 12ec9f1, 70ba641 (coordinator re-freeze), 154d269, cbce0c1, 42f1b3b; U2 36fa1df, 1b0368b, ba6cb7b; U3 67d9e95, 4f9dfba, 6a8aff7, c2d8cc1; merges 0175113 (U2), b3427b1 (U1), d81245f (U3), 0bdb9f5, f346f27, e448671e (U1 corrections 3–5); coordinator integration fixes 08ee01a, 4e63718, 144aa95 (+ git note), d5f0848; the closing store commit; the merge to main through a bot pull request. No tag, no release.

## Stage record

Final, 2026-09-09.

| unit | stage | head |
|---|---|---|
| U1 | implemented on the gate's record; merged (b3427b1, 0bdb9f5, f346f27, e448671e) | 42f1b3b |
| U2 | implemented on the gate's record; merged (0175113) | ba6cb7b |
| U3 | implemented on the gate's record; merged (d81245f); obligation:review-execution-recovery-implementation met | c2d8cc1 |

Filed on the way, outside the wave: story:browser-fixture-startup-deadline (Firefox 30 s deadline, pre-existing), story:fixtures-carry-workstation-paths (two coverage fixtures since 874962d, pre-existing), story:primitive-canonical-serialization (F08's decimal wire half, the epic's one remaining draft). The epic stays active for that draft.
