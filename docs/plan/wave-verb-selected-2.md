# Wave: verb-selected, turn 2

Proposed 2026-09-11 from `aep plan artifact waves`, not from an epic. Skill version **0.9.1**.
Selection path: **computed** — binary `protocol 0.55.0`, exit 0.

**Status: proposed, not approved, not started.** Wave mdi-1 is still in flight; this one cannot
start until it closes. See *Why it waits*.

## What the verb returned

Eight waves, 49 collisions, one unassessed.

| verb wave | n | stories |
|---|---|---|
| 1 | 11 | architecture-review-and-outlook, browser-fixture-startup-deadline, component-declares-its-settings, create-only-command-cannot-refuse, fixtures-carry-workstation-paths, integrate-source-driven-realizations, literal-representation-walk-exhaustion, native-realization-ci, normalization-equality-eligibility, primitive-canonical-serialization, release-status-publication-state |
| 2 | 3 | enum-variant-in-an-entity-invariant, interpreted-command-execution, schema-unique-items-signed-zero |
| 3 | 1 | interpreted-scenario-supplied-facts |
| 4 | 1 | java-conformance-target |
| 5 | 1 | the-generated-go-runtime-is-gofmt-clean |
| 6 | 1 | interpreted-bindings-and-unmet-obligations |
| 7 | 1 | interpreted-eventual-views |
| 8 | 1 | interpreted-trust-gate |

Unassessed: `story:outcome-decided-by-environment` — still unscoped, still not proposed.

## Where the verb cannot see, and I disagree with its silence

**The verb computes over `--status draft` only.** Wave mdi-1's two units are `active`, so they are
invisible to it, and it therefore reports **no collision** between its wave 1 and anything running.
That is a filter artefact, not disjointness. Read from the scope entries directly, five of wave 1's
eleven touch a surface a running unit owns:

| story | surface | against |
|---|---|---|
| `create-only-command-cannot-refuse` | `crates/verify/ess-conformance` | unit 1's crate — **collides** |
| `integrate-source-driven-realizations` | `crates/edge/ess-cli/src/main.rs` | unit 1's exact file — **collides** |
| `primitive-canonical-serialization` | `crates/verify/ess-conformance/src/{input,report,witness}.rs`, `src/go/runtime.go`, `assets/coverage-admission.js` | same crate, different files — **likely** |
| `browser-fixture-startup-deadline` | `crates/edge/ess-cli/tests/support/browser.rs` | same crate, different file — low |
| `fixtures-carry-workstation-paths` | two `ess-cli` test fixture JSONs | same crate, different files — low |

## The proposed set

The six of wave 1 that touch neither running unit, capped at **N = 4** by disk (below):

| # | story | scope | first surface |
|---|---|---|---|
| 1 | `component-declares-its-settings` | cited | `crates/specify/ess-domain/src/component.rs` |
| 2 | `literal-representation-walk-exhaustion` | cited | `crates/specify/ess-domain/src/binding.rs`, `command.rs` |
| 3 | `native-realization-ci` | cited | `.github/workflows/ci.yml`, `Taskfile.yml` |
| 4 | `release-status-publication-state` | cited | `crates/edge/ess-xtask/src/main.rs` |

Held back from the same clean six, and why:

- `architecture-review-and-outlook` — cited, docs and `website/`, safe; left out only by the N cap.
- `normalization-equality-eligibility` — **inferred** scope, one design-page path. Not unsafe,
  unestablished; it wants a scoper before it is dispatched.

`subagent_type`: **`aep-drive:implementor`**, then **`aep-drive:adversary`**.

## Why it waits, and the number that decides N

| measurement | value |
|---|---|
| one unit's build directory, measured in flight | **8.5 G** and still growing (`~/.cache/ess-wave-mdi1/unit1/target`) |
| a docs-only unit's build directory | 4 K |
| free disk now | **49 G**, down from 85 G when mdi-1 started |
| four more Rust units at the measured cost | ~34 G, against 49 G free that is still falling |

That is the refusal. Four concurrent Rust builds on top of mdi-1's two would land inside a few
gigabytes of full, and a wave that fills the disk at unit three has destroyed two agents' work.
This wave starts when mdi-1 closes and its build directories are reclaimed — which is also the
skill's own rule that the next turn replans rather than continues.

## Commits approval would authorise

Four unit commits, four merges into the integration branch, the closing store commit, and the merge
of the integration branch into the base. Nothing else — no push, no tag, no release.

## Per-unit record

| unit | branch | head | worktree | build dir | scratch | stage |
|---|---|---|---|---|---|---|
| 1 | — | — | — | — | — | not started |
| 2 | — | — | — | — | — | not started |
| 3 | — | — | — | — | — | not started |
| 4 | — | — | — | — | — | not started |
