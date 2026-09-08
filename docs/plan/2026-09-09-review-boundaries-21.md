# Review boundaries 21 — the last three remediation stories

## Selection and authority

Skill version 0.8.1 (`aep-drive` wave; `aep` protocol 0.54.0). N=3, all serving vision:O2 and
decomposing epic:review-boundary-remediation:

| unit | story | tags | scope confidence |
|---|---|---|---|
| U1 | story:review-primitive-semantics | P1 | high on the four packages and defect sites, medium on vector location — 17 cited / 4 inferred entries |
| U2 | story:review-typed-diagnostics | P2 | medium — compiler files cited, `ess-primitives/src/error.rs` cited from `resolve.rs:528`, 12 `ess-domain` files inferred — 4 cited / 13 inferred entries |
| U3 | story:review-execution-recovery-implementation | P1 | high on 21 existing paths and three byte pins, medium on 13 new-module paths — 21 cited / 13 inferred entries |

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

## Stage record

Updated by the coordinator as units change stage.

| unit | stage | head |
|---|---|---|
| U1 | proposed | — |
| U2 | proposed | — |
| U3 | proposed | — |
