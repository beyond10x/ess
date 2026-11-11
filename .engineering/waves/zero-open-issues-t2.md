# zero-open-issues: turn 2

Turn 2 of 32 of the wave plan [`zero-open-issues.md`](zero-open-issues.md) (§ The turns), for
`epic:zero-open-issues-2026-11`. Skill version 0.26.0 (`aep:implementing`, wave mode; `aep
0.79.0`). Run non-interactive: no operator turn exists until the goal is met, so the stage-1 stop is
`approval-record:zero-issues-turn-2-proposal` (tagged `non-interactive`). The turn runs under the
larger-turns rules of `approval-record:zero-issues-larger-turns`.

The turns are the operator-side planner's reordering of the 54-wave plan (2026-11-10): the stories
that edit synthesize.rs run serially in one lane, which goes against no printed collision, because
colliding stories stay serial; every other story keeps its collisions apart and its dependencies
earlier. The plan-level record is `approval-record:zero-issues-turn-plan`.

Status: delivered (2026-11-11)

## Units

Every unit serves `vision:O2`. Scope: every story already carried a typed `## Scope` when the turn
was drawn, so no `aep:story-scoper` ran; each `## Scope` is rewritten from the implementor's
confirmation table at close. "inferred" below means the verb printed `inferred: true`; "cited" means
every entry is cited.

| role | story | issue | scope |
|---|---|---|---|
| lane 1 | `story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie` | #708 | inferred |
| lane 2 | `story:an-absent-optional-under-false-and-unknown-is-arranged` | #728 | inferred |
| lane 3 | `story:an-arrangement-refusal-names-the-cause-of-every-creator-tried` | #728 | inferred |
| lane 4 | `story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state` | #739 | cited |
| lane 5 | `story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row` | #733 | inferred |
| unit 1 | `story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence` | #765 | inferred |
| unit 2 | `story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable` | #731 | inferred |
| unit 3 | `story:a-declared-field-format-renders-or-is-refused` | #723 | inferred |
| unit 4 | `story:a-set-guard-fed-by-a-subject-payload-is-arranged` | #705 | inferred |
| unit 5 | `story:the-react-renderer-draws-the-declared-tokens` | #723 | inferred |
| unit 6 | `story:a-ui-test-declares-how-each-read-and-command-is-answered` | #722 | inferred |
| unit 7 | `story:an-authored-scenario-retires-the-refusal-it-covers` | #735 | inferred |
| unit 8 | `story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation` | none (store defect) | inferred |
| unit 9 | `story:the-date-time-agreement-oracle-refuses-non-digits` | none (store defect) | inferred |
| unit 10 | `story:the-specification-checks-command-response-wire-names` | none (store defect) | inferred |

Scope decision the coordinator took to keep the turn disjoint: the `emit.rs` hunks for a declared
field format belong to unit 3, not unit 5, whose Scope named `emit.rs` for its tokens only.

## The verb's lines for the turn

Check path: `aep plan artifact waves --kind story --status active --format json` (the verb, not a
pairwise reading). Cycles: `[]`. Unassessed: 0. Placements of the turn's stories, verbatim:

```
wave 1: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation inferred=true
wave 1: story:a-ui-test-declares-how-each-read-and-command-is-answered inferred=true
wave 1: story:an-authored-scenario-retires-the-refusal-it-covers inferred=true
wave 1: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable inferred=true
wave 1: story:the-date-time-agreement-oracle-refuses-non-digits inferred=true
wave 1: story:the-specification-checks-command-response-wire-names inferred=true
wave 2: story:a-declared-field-format-renders-or-is-refused inferred=true
wave 2: story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence inferred=true
wave 2: story:the-react-renderer-draws-the-declared-tokens inferred=true
wave 3: story:a-set-guard-fed-by-a-subject-payload-is-arranged inferred=true
wave 12: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie inferred=true
wave 17: story:an-absent-optional-under-false-and-unknown-is-arranged inferred=true
wave 18: story:an-arrangement-refusal-names-the-cause-of-every-creator-tried inferred=true
wave 19: story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state inferred=false
wave 25: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row inferred=true
```

Every collision between two stories of the turn, verbatim; all eleven are inside the lane, and
colliding lane stories merge serially:

```
story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie <> story:an-absent-optional-under-false-and-unknown-is-arranged on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie <> story:an-arrangement-refusal-names-the-cause-of-every-creator-tried on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie <> story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie <> story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:an-absent-optional-under-false-and-unknown-is-arranged <> story:an-arrangement-refusal-names-the-cause-of-every-creator-tried on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:an-absent-optional-under-false-and-unknown-is-arranged <> story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:an-absent-optional-under-false-and-unknown-is-arranged <> story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state on crates/verify/ess-conformance/src/synthesize/subject_fact.rs (cited)
story:an-absent-optional-under-false-and-unknown-is-arranged <> story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:an-arrangement-refusal-names-the-cause-of-every-creator-tried <> story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:an-arrangement-refusal-names-the-cause-of-every-creator-tried <> story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state <> story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row on crates/verify/ess-conformance/src/synthesize.rs (cited)
```

No unit collides with another unit or with the lane. Every story of the turn is `active`, carries
no `depends_on` and no `blocked_by`.

## Stories that left the turn

None.

## On the way out

Each story moves `active -> implemented` (`aep plan artifact lifecycle story`) after a
`verification` record (baseline vs treatment) and a `test_result` record against its merge commit.

## Pre-flight (2026-11-10, 19:04 UTC+1)

- `df -h /`: 848G size, 810G used, 30G available (97%).
- `uptime`: load average 117.83, 92.12, 82.53. Every cargo command runs through the run's build
  script on two shared target slots with sccache; unit trees hold no `target/`.
- Integration head at stage 1: `d9eb1eecd7`, equal to `origin/integration/ess-zero-issues`.

## Agents

`aep:implementor` (sonnet), one for the lane, one per parallel lane story and one per unit;
`aep:adversary` (opus) once per lane story and per unit that touches guards, synthesis or emitted
bytes, a second pass only after a blocker; `aep:security-reviewer` (opus) where a unit touches input
parsing.

## Commits this turn authorises

One commit per lane story and per unit (plus correction commits), one `--no-ff` merge of each
parallel lane branch into `lane/t2-synthesize`, one `--no-ff` merge of `lane/t2-synthesize` and one
per unit into `integration/ess-zero-issues`, one reconcile commit per merge for the shared surfaces,
this page's opening store commit, one closing store commit, and pushes of
`integration/ess-zero-issues`. No tag, no release, no PR, no merge to `main`.

Amended 2026-11-11 by the operator ("could you cut a new ess release now"): turns 1 and 2 ship
now. One PR `integration/ess-zero-issues` -> `main` through the bot, whose CI is the full gate,
merged when green. Still no tag and no release commit: the operator cuts the release.

## Trees

Unit trees built through the run's build script on shared target slots; scratch is the turn's
scratch directory `t2/` under the run's scratch root.

| id | branch | story | stage | merge |
|---|---|---|---|---|
| `zi-t2-0` | `lane/t2-synthesize` | lane 1-5 | merged, finished, gc | `76bac8edf6` |
| `zi-t2-11` | `lane/t2-s3` | lane 3 (parallel) | merged into the lane, finished, gc | in `76bac8edf6` |
| `zi-t2-12` | `lane/t2-s5` | lane 5 (parallel) | merged into the lane, finished, gc | in `76bac8edf6` |
| `zi-t2-1` | `impl/the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence` | unit 1 | merged, finished, gc | `15647d1b24` |
| `zi-t2-2` | `impl/an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable` | unit 2 | merged, finished, gc | `12f434e632` |
| `zi-t2-3` | `impl/a-declared-field-format-renders-or-is-refused` | unit 3 | merged, finished, gc | `2aef845b57` |
| `zi-t2-4` | `impl/a-set-guard-fed-by-a-subject-payload-is-arranged` | unit 4 | merged, finished, gc | `2723ee70a8` |
| `zi-t2-5` | `impl/the-react-renderer-draws-the-declared-tokens` | unit 5 | merged, finished, gc | `bb691ed760` |
| `zi-t2-6` | `impl/a-ui-test-declares-how-each-read-and-command-is-answered` | unit 6 | merged, finished, gc | `61e4f7dce6` |
| `zi-t2-7` | `impl/an-authored-scenario-retires-the-refusal-it-covers` | unit 7 | merged, finished, gc | `dc5e6c9e6e` |
| `zi-t2-8` | `impl/a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation` | unit 8 | merged, finished, gc | `05e22e01fd` |
| `zi-t2-9` | `impl/the-date-time-agreement-oracle-refuses-non-digits` | unit 9 | merged, finished, gc | `ce1bbd0c0f` |
| `zi-t2-10` | `impl/the-specification-checks-command-response-wire-names` | unit 10 | merged, finished, gc | `6aa9e99a1d` |

## Gate

One chain on `fd62240ee0` through the turn's build script, one crate group at a time, each under
25 GB with a clean between groups; outputs in the turn scratch under `gate/`. The chain was stopped
during the ess-cli group when the operator chose to ship turns 1 and 2 with the PR's CI as the full
gate; every step a correction touched was re-run locally on the corrected head.

- `fmt-check`: exit 0 on `fd62240ee0` and on the corrected head.
- clippy `-D warnings`: ess-domain..ess-entity-runtime, the UI crates and the ess-conformance
  targets of the turn failed on seven test-only lints (fixed in `3e4f7a89c9` and `66371526dc`); ess-cli and ess-xtask
  exit 0. Re-run on the touched crates and targets after the corrections.
- `cargo xtask docs`: exit 0.
- ess-domain, ess-diff, ess-compiler: three failures, fixed. `selection_precedence_table` (the seven
  lane fixtures were not pinned; `7d2f833285`), `typed_diagnostics` (the head census missed the
  second `command` literal in `wire.rs` from unit 10; `3e4f7a89c9`, `66371526dc`), `binding_stored_read_idiom`
  (the section moved to its own page in turn 1's `045e12f293`; `3e4f7a89c9`).
- ess-synth, ess-gen, ess-entity-runtime: 1522 of 1529. `lowered_definitions_table` missed the same
  seven fixtures (re-pinned in `2b82d93f55`). The six `adversary_view_params_pass1/2` failures are
  the probe-directory race seen in turn 1: each process now builds in its own directory
  (`2b82d93f55`), and the two binaries pass in parallel (19 of 19 with the date-time target).
  Peak 26 GB in the integration tree's target, with 29 GB free on `/`.
- ess-cli, stopped at 786 of 1268:
  - `coverage_cli` (2 tests): lane 5 now witnesses the `ConfirmDecision/already-exists` refusal
    that unit 7's authored-covers fixture covered. The fixture now covers an `ESS-SYNTH-018`
    refusal (a note of more than 2000 tags) with a two-step authored scenario (`2b82d93f55`);
    `coverage_cli`, `authored_covers` and `adversary_t2_covers_readers` pass.
  - Three `observed_bindings` tests fail only because the integration-tree mode put TMPDIR inside
    the checkout, which the CLI refuses; not a regression.
  - The a16 input-discovery case fails with "path must be shorter than SUN_LEN" on `main` too
    (`story:the-a16-socket-case-binds-under-any-tmpdir`, filed in turn 1).
- Not run locally: the UI crate tests, the ess-conformance groups, `task test-xtask` and the
  projections. The PR's CI runs them.

## Backlog drafts filed

- `story:the-security-oracle-checks-date-time-as-ess-does`
- `story:the-terminal-shows-the-time-of-an-http-date`
- `story:a-partial-type-style-keeps-the-built-in-fields-of-its-name`
- `story:a-negated-list-literal-is-witnessed-beside-any-other-leaf`
- `story:an-interpreted-struct-member-follows-its-declared-presence`
- `story:a-binding-mapping-of-an-absent-optional-event-field-passes`
- `story:a-binding-flow-awaits-the-event-its-trigger-causes`
- `story:a-subject-route-refusal-names-every-creator-whatever-its-cause`
- `story:a-left-out-optional-stays-named-when-another-row-is-built`
