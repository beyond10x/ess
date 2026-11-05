---
format: aep.planning-md/3
id: approval-record:ess-0870-wave
kind: approval-record
status: approved
title: Wave W1 of the reconcile plan runs four units (synthesis refusals, OpenAPI publication text, value formats, choices/filters/forms) as one wave, one pull request and release 0.87.0
summary: Coordinator approval of 2026-11-04 under the operator's grant, with the W1 decision round's answers
tags:
- non-interactive
relations:
- decides: story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair
- decides: story:a-value-invariant-on-a-type-nothing-reaches-refuses-nothing
- decides: story:the-openapi-says-published-only-for-events-the-component-publishes
- decides: story:a-value-renders-in-the-form-its-field-declares
- decides: story:choices-filter-bars-and-forms-render-what-they-declare
revision: 20
transitions:
- {from: "draft", to: "approved", at: "2026-11-04T08:47:27Z", actor: "human:timo", revision: 2, executor: "agent:ess-w1-lead"}
---
## Decision

2026-11-04, the coordinator, under the operator's grant (class C: a wave, bot commits, a bot pull
request and its merge on green, a release through ess's own process), answered the wave proposal:

> Accept all five drafts.
> Q-W: a. The plan's collections and runner steps move to W2.
> Q-A: a. Q-C: a.
> G-3: a. G-7: a. G-8: a. G-12: a. G-13: a. G-14: a. G-15: a.
> adopter report: a (B/KB/MB/GB in 1024 steps; metric `bytes` scales the same way).
> Record this in an approval record (no private ids) and move the stories to active. Release 0.87.0.

The five stories below are moved draft -> proposed -> active by the wave lead on those words; the
lead accepted none on its own.

This run is non-interactive: the wave lead is a sub-agent with no operator turn. Skill: aep
`implementing` 0.26.0, wave mode. Agents: `aep:implementor`, `aep:adversary`.

## The questions and the answers

| question | answer |
|---|---|
| Q-W, the wave's units | a: synthesis (two stories), OpenAPI publication text, value formats, choices/filters/forms. The reconcile plan's collections (A10, A21-A27) and runner steps (B01-B11, B33-B35) move to W2. |
| Q-A, a not-found after a guarded move | a: the precedence order stands; one refusal whose help names `before_row: true` and declaring the not-found first; no ESS-SYNTH-019 |
| Q-C, an invariant on a type nothing reaches | a: a note, as an unreached `Binary64` position is; a reached type keeps ESS-SYNTH-013 |
| G-3, header badge | a: `header.badge: {text, tone}`, `text` an expression (built in W2) |
| G-7, action tone | a: `tone:` on an action, the button's words (primary, secondary, danger, ghost) (built in W2) |
| G-8, field tone | a: `tone` and `tone_by` on a field, badge and plain columns; `tone_by: <map>` reads the field's own value (built in W2) |
| G-12, lists in binds | a: a YAML sequence of expressions in a bind, sent as a JSON array (built in W2) |
| G-13, help | a: `help: {text, link}` on a form group and a field (a field covers its column header) |
| G-14, choice placeholder | a: `placeholder: <text>` on a choice, also the empty dropdown option |
| G-15, reset label | a: `reset: true` or `reset: {label}` |
| adopter report, byte size | a: `as: bytes` on a field, the metric's existing word; B/KB/MB/GB in 1024 steps; metric `format: bytes` scales the same way |

## Units

Base `origin/main` 8d26c81f74. Integration branch `integration/ess-0.87.0`, tree `ess-w087-int`.
Release 0.87.0.

| unit | tree | branch | stories |
|---|---|---|---|
| U1 | ess-w087-u1 | unit/ess-0.87.0-u1 | story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair, story:a-value-invariant-on-a-type-nothing-reaches-refuses-nothing (serial, one implementor: both land in synthesize.rs) |
| U2 | ess-w087-u2 | unit/ess-0.87.0-u2 | story:the-openapi-says-published-only-for-events-the-component-publishes |
| U3 | ess-w087-u3 | unit/ess-0.87.0-u3 | story:a-value-renders-in-the-form-its-field-declares |
| U4 | ess-w087-u4 | unit/ess-0.87.0-u4 | story:choices-filter-bars-and-forms-render-what-they-declare |

U3 and U4 share `crates/ui/ess-ui/src/model.rs`, `schemas/ui/ess-ui.schema.yaml`,
`crates/ui/ess-ui-tui/src/view.rs` and the generated `website/docs/reference/ess-ui.md`, split by
symbol: U3 owns `Field`'s `as:` vocabulary and the metric format; U4 owns `Choice`
(`placeholder`), `FilterBar` (`reset`), `FormGroup` and `Field` (`help`). The reference page is
regenerated at each merge.

## Commits this authorises

The unit commits; the merges into `integration/ess-0.87.0`; store commits (this record, scope,
evidence, close); the 0.87.0 release prepare; one bot pull request from a `queue/ess-0.87.0` head,
merged on green; tag 0.87.0 and the release through ess's own process. Nothing else.

## Go and build terms

The orchestrator relayed the host's build terms on 2026-11-04 at 08:5xZ, and the wave started on them:

| what | tell at | stop at |
|---|---|---|
| U1 | 8G target/ or 90 min | 10G or 150 min |
| U2 | 12G or 45 min | 15G or 75 min |
| U3, U4 | 4G or 120 min | 6G or 180 min |
| each gate stage | 14G | 18G |

- One ESS tree builds at a time. Every cargo call goes through the shared build slot; units never take a second slot.
- The gate runs 9 stages with the cache discarded between stages. S6 to S9 take the second slot beside one unit, and
  each starts only with at least 45G free on `/`.
- Every build uses line-tables-only debug information, no incremental compilation and 4 test threads.
- A unit's tree drops its build cache as soon as the unit merges into the integration branch.

## Ride-along: CI test shards and runner disk

The orchestrator added `story:ci-test-shards-leave-evidence-when-the-runner-runs-out-of-disk` to this wave as a
ride-along on the integration branch, not as a fifth unit: "measure where the shard's disk goes; cut it, or free
more before the build ...; add the crash-surviving sampler if it is cheap. Accept it into W1 as a ride-along; no
fifth unit." The story moves to active with this record.

## Ride-along: row toggles in ui journeys

The orchestrator added a second ride-along on 2026-11-04: an adopter reported that since 0.86.0 a ui journey step
that clicks a row toggle fails with "the row does not offer <label>" (18 of 573 journeys, green on 0.85.0),
because the terminal hint now carries the toggle state mark and the runner matches the plain label.
`story:a-journey-clicks-a-row-toggle-in-either-state` moves to active with this record. It is small and stays in
`ess-ui-test`, so it takes the next free sub-agent slot and adds no unit; the CHANGELOG names it as a fix of the
0.86.0 regression.

## Answer to G-1: duration form

The orchestrator answered reconcile question G-1 on 2026-11-04 (items A01 and the duration reports adopter report
adopter report):

- `as: duration` takes an optional `unit: ms | s`, default `ms`.
- It renders `h:mm:ss` with the hours unpadded: 73000 ms is `0:01:13`. Past 24 hours it stays in hours: 26 hours is
  `26:00:00`.

U3's scope is unchanged. The answer is built as a ride-along of this wave (see "Ride-along: duration form and
short columns").

## Ride-along: duration form and short columns

The orchestrator added two more ride-alongs on 2026-11-04. Each starts after the unit whose files it shares has
merged into the integration branch, takes the next free sub-agent slot after U2, and runs named test targets only:

- `story:a-short-text-column-is-not-truncated-while-the-table-has-room` (adopter report), after U4: the
  collection template and styles.
- `story:a-duration-renders-as-hours-minutes-seconds` (G-1 answered; adopter report), after U3: the field
  template and the field model.

Both stories move to active with this record. The lead tells the orchestrator when U3 and U4 merge.

## Accepted for the next wave: suffixed-instance synthesis

On 2026-11-05 the orchestrator accepted beyond10x/ess#736 as one story,
`story:a-suffixed-instance-is-created-with-the-inputs-the-specification-gives` (active). It goes first in the next
ESS wave after 0.87.0, because a downstream consumer has 30 of 48 red conformance scenarios behind it. It joins
this wave only if a unit slot is free and nothing collides. It lands in `synthesize.rs`, which U1 holds, so it
waits.

## Short columns in the browser

On 2026-11-05 the orchestrator relayed the adopter's answer to the short-column question: the report was about the
browser. The browser side was to ride along under the same story as a CSS-only change, unless 0.87.0 no longer
sets nowrap on text cells. The stock runtime sets no nowrap on any table cell, in 0.85.0 or now; the nowrap came
from the adopter's theme. That is recorded in the story, and no CSS change is made.

## Adversary findings on the UI work

The adversary pass over the integrated UI work (branch adv/ess-0.87.0-ui: red cases 961bb26837, pins 5ecdd17e46)
returned needs-change: 3 of 59 cases red, all introduced by this wave. The orchestrator decided on 2026-11-05:

| finding | severity | decision |
|---|---|---|
| F1: `form.tsx.tmpl` renders a form's parts outside the save-on-change scope layer, so a value changed in a part of a `save: on_change` form is lost | warning (U4) | fix in one correction unit: render the parts inside the layer |
| F3: the React `placeholder_file` (fixtures.rs) strips the declared dir as a prefix, so under `dir: ./data` the path `data/quarters.yaml` resolves to `./data/data/quarters.yaml` | note (U3) | fix in the same correction unit: normalise a leading `./`; the same fix and one case in the terminal's `data.rs` |
| F2: `read_takes` (emit.rs) ignores a read given the record that holds the search state | note (U4) | no document in the repository has that shape: a from-finding draft, no fix in this wave; its red case is ignored with a reason naming the draft |
| ride-along | | a field help link (`HelpNote`, fields.tsx.tmpl) goes through `safeUrl`, as the page header's link does |

The correction unit merges the adversary's pins and red cases with the fix, so every case passes or is ignored
with its reason.

## Adopter labels and the squash merge

The coordinator decided on 2026-11-05 that a downstream adopter's item labels stay out of every
committed text of this public repository, the planning store included.

- Mutable story bodies and summaries and this record had the labels replaced with `adopter report`
  through `aep plan artifact redact`, one literal per call; each call left its evidence record.
- The release changelog entries and three test comments dropped the labels in the release prepare.
- Commits already pushed to the integration branch keep their text: history is not rewritten.
  The pull request into `main` is merged as one squash commit with a title and body written for it,
  so neither those commit messages nor the intermediate diffs reach `main`; the integration branch
  is deleted after the merge.
- A squash merge has no second parent, so the release's `prior-gate` rule 2 cannot reuse the queue
  run. The tag goes on the squash commit after its `main` push run passes every required check
  (rule 1); otherwise the release runs the full gate.

## CI disk: the extraction footprint

The coordinator decided on 2026-11-05 that the queue run's disk stop is fixed in CI by
extracting the test archive in its own step and deleting it before the tests run, measured first:
the push goes only if the shard's peak fits the runner's free disk with 5G to spare. A larger
runner is spending and more shards do not shrink the one shared archive; neither is taken.

Measured on the second queue run (Test 4/20, a passing shard on a runner with 121G free):
`target/` held 17.1G of downloaded archives before extraction and 63.9G after it, so the
extracted workspace binaries take 46.8G, and the tests add 1.2G more (65.1G peak). The stopped
shard (Test 11/20) ran on a runner with 52G free after the cleanup step and no `/mnt`. Extracting
and then deleting the archive peaks at the archive plus 46.8G, about 64G, and the extracted
binaries and test output alone need 48G: neither leaves 5G on 52G. Nothing was pushed for it; a
variant that shrinks the extracted bytes is measured first.

## CI disk: the stripped-binary measurement

The coordinator then decided to measure the variant that shrinks the binaries: the test-archive
build without debug line tables (`-C strip=debuginfo`), for the CI archive jobs only, local profiles
unchanged. One test target, `ess-conformance --test suite`, is built with line tables and its size
compared with the same binary stripped of debug information; the 46.8G of extracted binaries is
scaled by that ratio. At 35G or less the archive jobs strip debug information and each shard
downloads, extracts, deletes the archive and then runs; above it, nothing is pushed. Backtraces in
CI test binaries lose line numbers; panic locations keep file and line.

Measured: the local line-table binary is 178.8 MB, 90.5 MB with its debug sections removed (0.51).
The ratio does not apply to the 46.8G: the CI archive jobs already build with `debug = 0` and
`strip = symbols` (`ci.yml`, held by `ci_lanes.rs`
`the_archive_jobs_build_the_test_binaries_without_debug_information`), so the extracted binaries
are already stripped and the estimate stays 46.8G, above 35G. Nothing was pushed. A shard that
extracts the archive into the checkout, deletes it and runs from `target/nextest/*-metadata.json`
was checked with a small archive (`--binaries-metadata`, `--cargo-metadata`, `--workspace-remap`,
`--target-dir-remap`): the tests run without a rebuild.

## CI disk: the release goes on, the split follows

The coordinator decided on 2026-11-05 how 0.87.0 goes on. The pull request takes only the clippy
fix (`let ... else` in the unreached-invariant note), checked locally with clippy at
`-D warnings` on `ess-conformance --lib` and the `ci_lanes` tests. A shard stopped by the disk
watch on a 52G runner gets one re-run of the failed jobs through the bot; a second disk stop
brings the two-archive split into this pull request, and any other red stops the release. The
split itself is `story:ci-test-shards-each-extract-half-the-test-binaries`, accepted as the first
unit of the next ESS wave (0.88.0).
