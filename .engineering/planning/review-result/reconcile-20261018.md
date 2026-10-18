---
format: aep.planning-md/3
id: review-result:reconcile-20261018
kind: review-result
status: archived
title: 'Reconciliation of 2026-10-18: open issues, dangling branches, the store'
summary: 22 issues classified, 60-odd branches judged by content, store moves proposed
relations:
- reviews: epic:downstream-reported-gaps
- reviews: epic:generic-row-queries
- reviews: epic:generic-guard-precedence
- reviews: epic:generic-expression-language
- reviews: epic:generic-aggregates
- reviews: release-plan:ess-063
- reviews: release-plan:ess-060
prose_only: an audit of issues, branches and the store, not a critic verdict
revision: 2
transitions:
- {from: "active", to: "archived", at: "2026-10-18T21:33:58Z", actor: "human:timo", revision: 2}
---
Reconciliation of the 22 open GitHub issues, the local and remote branches not on `origin/main`,
and the planning store, read on 2026-10-18 against `origin/main` `a500f9f56` (0.62.0). Every
claim cites a command run on that day or says "I don't know". Branch classification is by content
(tests, CHANGELOG lines, ancestry with `git merge-base --is-ancestor`), never by patch-id.
Two local branch names carry a domain abbreviation and are given by head commit instead: the port branch (head `30de58532`) and the 2026-10-13 integration branch (head `786d571d6`); `git branch --contains <head>` names them. Companion artifacts: `release-plan:ess-063` (the waves), the stories and tasks it names,
`specification:one-owned-entity-per-state-axis` (#559).

## A. The 22 open issues

Class per `.agents/skills/assessing-external-requests/SKILL.md`; "wave" per `release-plan:ess-063`.

| issue | class | duplicate of / overlaps | verdict | salvageable branch | story | wave |
|---|---|---|---|---|---|---|
| #596 set move, per-row outcome | gap (one-call batch); convenience (answers per row) | #547 (set effects), G2 `each` (#564 family) | accept, redesigned: `each:` admits `moves:`/`deletes:` with an element `where:`; idiom today: one command per row (`.engineering/repro/596/per-row-settle.yaml`, valid, 12 scenarios) | — | `story:each-entry-moves-the-rows-its-elements-name` | 3 |
| #594 untagged union | gap (retrofit) | — | accept, redesigned (operator decision 2026-10-17, option c: `shorthand:` on a struct, `ess/27`) | `unit/ess-u594` (half-done, committed, uncompiled) | `story:json-type-discriminated-union` (on the branch) | 2 |
| #593 import refuses `x-ess-*` | defect | #590, #591 (one story) | accept, redesigned | `integrate/ess-wave-20261017` (finished, unshipped) | `story:schema-tools-accept-projection` (on the branch) | 1 |
| #592 alphabet not asserted | defect | #590–#593 second slice | accept as proposed | — (not on the branch) | `story:alphabet-projects-an-anchored-pattern` | 2 |
| #591 YAML instances refused | defect (two tools in one CLI) | #590, #593 | accept, redesigned (extension decides, no `--format`) | `integrate/ess-wave-20261017` | `story:schema-tools-accept-projection` | 1 |
| #590 no `$id` | defect | #591, #593 | accept, redesigned (`--schema-id-base`, opt-in) | `integrate/ess-wave-20261017` | `story:schema-tools-accept-projection` | 1 |
| #584 digest in guards; external cannot stand in | gap (option 1); gap (option 2) | option 1: G1 text/bytes family; option 2: #486 family, G4 | accept, redesigned: option 1 is the answer, option 2 its ordering half | — | `story:digest-and-encoding-functions-in-expressions`; `story:external-branch-narrowed-by-subject-guard` | G1; G4 |
| #572 compact, drift-checked, reviewable suite | defect (size) + gap (`--check`, suite diff) | closed #570, #571 (same story) | accept, redesigned (already recorded) | — | `story:a-committed-suite-is-compact-drift-checked-and-reviewable` | 4 |
| #564 queries over rows | gap | reopens closed #447, #492 gap 4; converges #543, #547, #549, #551, #557, #558 | accept, redesigned into G2 (already recorded) | the port carries #543/#547/#549 as narrower spellings G2 keeps | `epic:generic-row-queries` | G2 |
| #561 aggregates over expressions | gap | closed #441 decision 3, #562, #563, #565, #566 | accept, redesigned into G3 (already recorded) | — | `epic:generic-aggregates` | G3 |
| #560 expression language | gap | reopens closed #442, #439; #555, #556, #567; #545 (value roots) | accept, redesigned into G1 (already recorded) | — | `epic:generic-expression-language` | G1 |
| #559 two lifecycles on one entity | convenience | — | **decline, with the idiom**: one entity per axis, `when_related` over `state` (`.engineering/repro/559/one-entity-per-axis.yaml`, valid, 18 scenarios) | — | `specification:one-owned-entity-per-state-axis`; `story:an-entity-carries-a-second-state-axis` archived | — |
| #554 one precedence rule | gap | closed #473, #282, #530, #534, #535 (shipped 0.60.0 special cases) | accept, redesigned into G4 (already recorded) | — | `epic:generic-guard-precedence` (five stories) | G4 |
| #552 synthesis drops sibling branches | defect | #543 | fixed on the port | the port branch (head `30de58532`) | `story:minted-head-refuses-only-its-branch` (on the branch) | 1 |
| #551 selector tests related rows | gap | #543, #549; G2 | accept, redesigned (design page on the port) | tree `ess-u551-impl` (30 uncommitted files) | `story:related-rows-quantifier-in-a-selector` (on the branch) | 3 |
| #549 nested heads | gap | #543, #545; G2 | accept, redesigned; implemented on the port | the port branch (head `30de58532`) | `story:published-value-operand-in-a-selector` (on the branch) | 1 |
| #547 `affects:` by identity; generators refuse set effects | defect ×2 | #596, #542, #545; second adopter in its comments | accept, redesigned; implemented on the port | the port branch (head `30de58532`) | `story:set-effect-filters-read-the-identity` (on the branch) | 1 |
| #546 Optional inputs filled | defect | #544 | fixed on the port | the port branch (head `30de58532`) | `story:synthesis-leaves-unread-optional-inputs-absent` (on the branch) | 1 |
| #545 present mint, payload read | gap | #544, #560 (value roots) | accept, redesigned; implemented on the port | the port branch (head `30de58532`) | `story:minted-value-present-and-shared-with-its-payload` | 1 |
| #544 author accepts an exact value on a generated field | defect | #545, #546 | accept (refuse, not warn); implemented on the port as `ESS-AUTHOR-042` | the port branch (head `30de58532`) | `story:authored-exact-value-on-generated-field-is-refused` | 1 |
| #543 head of an ordered selection | gap | #549, #551, #552; G2 | accept as proposed; implemented on the port | the port branch (head `30de58532`) | `story:payload-source-reads-the-head-of-an-ordered-selection` | 1 |
| #542 system-issued commands | gap | #543, #547 | accept, redesigned (`system: true` actor, await steps); both stages on the port | the port branch (head `30de58532`) | `story:system-issued-command-is-awaited-not-driven`, `story:system-issued-command-await-in-go-and-typescript-runners` | 1 |

Bad-report verdicts: #559 is the one report answered by modelling it differently (idiom shown
above). #596 is half of one: the per-row command is the native shape when the answers arrive per
row; the one-call batch is a real gap. Every other report names a defect or a gap.

Overlaps among the 22: #543/#549/#551/#552 are one family (ordered heads) and converge in G2 with
#564; #547 and #596 are both set-effect rows of G2; #545 is a value-root cell of G1 (#560);
#590–#593 are one story; #584's two halves are a G1 cell and a G4 cell. No open issue duplicates
another open issue outright.

## B. Dangling work

Ancestry: `sh` loop over `git merge-base --is-ancestor <branch> <ref>` for the refs
`integrate/ess-issues-20261013`, the 2026-10-13 integration branch (head `786d571d6`), the port branch (head `30de58532`),
`integrate/ess-wave-20261017`. `integrate/ess-issues-20261013` differs from tag `0.60.0` by ten
files whose added lines are all on `origin/main` today (`git grep` of `refused_literals`,
`AlphabetUnsupported`, `n-forbidden` on `origin/main`), so everything that is an ancestor of it
shipped in 0.60.0.

| branch / stash / artifact | serves | status | evidence | action |
|---|---|---|---|---|
| `impl/ess-542`, `-stage2`, `-u1u3`, `-u4`, `-u5u7` | #542 | superseded | ancestors of the port branch (head `30de58532`) | delete after confirmation (local only) |
| `impl/ess-543`, `-u1u3`, `-u4u5`, `-u7` | #543 | superseded | ancestors of the port | delete after confirmation |
| `impl/ess-544`, `impl/ess-545`, `impl/ess-546` | #544, #545, #546 | superseded | ancestors of the port | delete after confirmation |
| `impl/ess-547-design`, `-u1u3`, `-u2u4u5` | #547 | superseded | ancestors of the port | delete after confirmation |
| `impl/ess-549-design`, `-u1u2`, `-u3u4` | #549 | superseded | ancestors of the port | delete after confirmation |
| `impl/ess-551-design` | #551 design page | superseded | ancestor of the port, which carries `docs/design/related-rows-quantifier.md` | delete after confirmation |
| `impl/ess-551-r1r2` + tree `ess-u551-impl` | #551 implementation | half-done, uncommitted | the branch has no commit of its own (head `16b0bdd9c` = the 2026-10-13 integration head); the tree holds 30 modified/added files, +2154/−113, `related_rows.rs` and three new test files among them | preserve (commit or export the diff), then `task:salvage-the-related-rows-quantifier-tree`; never discard |
| `impl/ess-fix-regressions` + tree `ess-fix-regress` | the 2026-10-14 integration head | superseded | same head as the 2026-10-13 integration branch (head `786d571d6`) (`786d571d6`); one uncommitted cosmetic rename in the tree, identical to the port's | delete after confirmation; `worktree finish` the tree |
| the 2026-10-13 integration branch (head `786d571d6`) and its tree | #542–#552 on the 0.59.0 base | superseded | ancestor of the port | delete after confirmation; finish the tree |
| tree `ess-regress-base` | a regression bisect base | abandoned | detached at `107971db6` (2026-10-13), clean | finish the tree |
| the port branch (head `30de58532`) and its tree | #542–#549, #552 on the 0.62.0 base | unshipped, merge complete, never compiled green | its uncommitted `RESUME.md` (hand-back notes), last `cargo check` exit 101 before `30de58532`; 2 cosmetic uncommitted edits | `task:land-the-2026-10-13-port` (wave 1) |
| `integrate/ess-issues-20261013` + tree `ess-issues-20261013` | 0.60.0 | shipped | ten-file residual against tag `0.60.0`, every added line on `origin/main` | delete after confirmation; finish the tree |
| `unit/ess-issues-20261013-{k0,k1,k2-b,k5,k6,l1,l2,l3,l4,l5,m1-b,m2,m3,m4,m5,m5adv,n0,n1,p548,p550,r1}` | 0.60.0 units (#493, #525, #530–#539, #541, #548, #550) | shipped | ancestors of `integrate/ess-issues-20261013`; their issues closed 2026-10-14 | delete after confirmation |
| `plan/ess-intake-20261013`, `plan/ess-issue-waves-20261013` (also `origin/plan/ess-issue-waves-20261013`) | the 2026-10-13 intake and waves | shipped | ancestors of `integrate/ess-issues-20261013` | delete local and remote after confirmation |
| `queue/ess-0.60.0` | 0.60.0 release queue | shipped | ancestor of `integrate/ess-issues-20261013` | delete after confirmation |
| `queue/ess-0.57.0` | 0.57.0 release queue | abandoned | 3 commits of 2026-10-08 on `e6fc27b1c`; 0.57.0 shipped from `origin/queue/ess-0.57.0-main` | delete after confirmation |
| `integrate/ess-wave-20261017` + tree `ess-wave-20261017`, `unit/ess-schema-tools` | #590, #591, #593 + two fixes | finished, unshipped (gates unrecorded) | 6 commits on `a500f9f56`; the unit is its ancestor | `task:land-the-schema-tools-wave-branch` (wave 1) |
| `unit/ess-u594` + tree `ess-u594` | #594 | half-done, committed, partly gated | commit bodies: `ess-domain`, `ess-compiler`, `schema-contract`, `ess-diff` gates green; `RESUME.md`: `ess-gen`, `ess-synth`, `ess-conformance`, `ess-cli` never compiled | `task:land-the-struct-shorthand-unit` (wave 2) |
| `stash@{0}` on `unit/ess-issues-20261013-m2` (`m2fix-f1f2`) | #534 `before_row` witnesses (`InsideLaterRefusal`) | shipped | `git grep` on `origin/main`: `InsideLaterRefusal`, `reaches_before_row`, `arranged_inside_later` present (10 and 1 hits) | drop after confirmation |
| `origin/unit/optional-input-related-via-20261003` | `{related:}` through an Optional reference (#285) | abandoned (preserved tree state) | 2 commits "preserve the staged/unstaged state of an abandoned tree", 2026-10-07; `story:feature-request-285` is implemented and its fixture name is on `origin/main` | delete remote after confirmation |
| `origin/plan/story-576-concurrent-exploration`, `origin/plan/park-consumer-accounting`, `origin/integrate/ess-wave-*`, `origin/deliver/*`, `origin/queue/ess-0.5x`–`0.62.0` | earlier releases | shipped | 0 commits ahead of `origin/main` | delete remote after confirmation |
| `frag-057`, `main` (local), `plan/adopter-hardening-drafts`, `plan/ess-g4-precedence`, `plan/fit-upcast`, `plan/story-576-concurrent-exploration`, `integrate/ess-0.60.0`, `integrate/ess-wave-2026100{8e,8f}`, `integrate/ess-wave-2026101{4a,6c}`, `unit/ess-wave-*`, `unit/selection-plan-*`, `update/pr488` | earlier releases | shipped | 0 commits ahead of `origin/main` | delete after confirmation |

Other sessions: the trees `ess-1018-u{1,2,3,5,6}-*` and `ess-integrate-20261018` are the
in-flight units and were not read. The port's `RESUME.md` is dated 2026-10-17 05:40–06:10Z and
`unit/ess-u594`'s 2026-10-17 15:5xZ; neither tree has a commit after that, so I don't know whether
either session is still active. No local branch has a commit after 2026-10-17.

## C. Planning store

Artifacts whose work shipped and that were never moved (each move is one `aep plan artifact
move`; a bulk move is the operator's call, so none was made here):

| artifact | status | shipped as | evidence |
|---|---|---|---|
| `release-plan:ess-060` | draft → **moved to `implemented` by this reconciliation** | 0.60.0, 2026-10-14 | tag `0.60.0`; `CHANGELOG.md` `## [0.60.0]` |
| `story:http-stage-2-design-answers`, `http-binding-request-encoding`, `http-binding-response-body-and-status-per-outcome`, `http-binding-response-headers-challenges-and-redirects`, `http-openapi-stage-2`, `http-rust-server-serves-the-binding`, `http-go-server-serves-the-binding` | draft | 0.60.0 (#493) | issue #493 closed 2026-10-14; `release-plan:ess-060` delivers them |
| `story:a-component-plays-a-pinned-contract-role` | draft | 0.60.0 (#525) | issue closed 2026-10-14 |
| `story:go-explorer-draws-read-only-commands`, `explorer-model-applies-selected-row-sets-and-draws-decimals`, `explorer-excludes-a-refused-step-not-the-command`, `precedence-swap-overlap-taken-earlier-scored-equivalent` | draft | 0.60.0 (#536–#539) | issues closed 2026-10-14 |
| `story:text-guard-ignores-surrounding-ascii-whitespace`, `selected-rows-effect-emits-one-event-per-changed-row`, `view-declares-its-answer-for-an-unknown-identity` | draft | 0.60.0, `ess/25` (#532, #531, #533) | `ess specify formats` on 0.62.0 lists all three under `ess/25` |
| `story:yaml-parser-without-unsafe-libyaml` | draft | 0.60.0 (#541, with the `ess-yaml` facade) | issue closed 2026-10-14; `release-plan:ess-060` § Decisions |
| `story:accepted-input-arranges-referenced-list-rows`, `wrong-state-for-an-unknown-identity-states-its-rule-and-repair` | draft | 0.62.0 units l6, l7 | `release-plan:ess-062` (archived) delivers them |
| `story:a-command-binds-to-a-protocol-or-deployment-http-route` | active | 0.60.0 (#493) | as above |
| `story:a-mutant-whose-suite-equals-the-baseline-is-unwitnessed`, `a-suite-fixes-the-targets-clock` | active | 0.62.0 (#586) | `CHANGELOG.md` 0.62.0 |
| `story:generated-decoders-enforce-declared-value-constraints`, `no-emitted-document-carries-a-number-wrapper` | active | 0.62.0 (#579) | `CHANGELOG.md` 0.62.0 |
| `story:every-witness-reads-back-its-outcome-effect` | active | 0.62.0 (#587, `ess/26`) | `ess specify formats` |
| `story:synthesis-witnesses-stay-unique-per-arranged-instance`, `synthesis-survives-two-identity-guards-on-a-self-reference`, `synthesis-covers-input-guards-beside-an-existing-instance-branch`, `diff-classifies-an-added-domain-entry`, `cli-binding-globals-config-and-output-are-optional`, `interpreter-default-outcome-beside-a-conjunction-guard`, `response-string-newtype-constraints-checked-not-refused` | active | 0.57.0–0.58.0 (#480, #474, #479, #469, #481, #471, #499) | issues closed 2026-10-07/08 |
| `story:a-scalar-to-record-input-change-is-breaking-for-callers`, `browser-fixture-startup-deadline` | active | I don't know | no issue reference; `7c468d06f` on `integrate/ess-wave-20261017` may be the second's fix |
| `release-plan:ess-24-one-language`, `release-plan:interpreter-runner-parity` | draft, "unscheduled" | — | still unscheduled; nothing to move |

Blockers still open, each for a reason that stands: `decision-blocker:consumer-accounting-parked`
(parked by design), `decision-blocker:decimal-canonical-serialization-crosses-repositories`
(waits on `obligation:review-contract-rollout-coordination`),
`dependency-blocker:aggregate-binding-cut-authority` (waits on adapter proof). Review-results: 251
records, all `active`, none since 2026-10-01 (`aep plan artifact review-value --since 2026-10-01`);
nothing to act on. `approval-record:ess-evolution-20260915` is `draft`; I don't know whether it
was meant to reach `approved`.

## D. Proposed GitHub actions (none taken; neutral wording)

| issue | action | comment |
|---|---|---|
| #559 | close, with the idiom | "Declined with the idiom: one entity per state axis, each with its own `lifecycle:`; a command on one axis reads the other through `when_related: {via: <reference>, predicate: state == …}`. A neutral model written this way validates and synthesizes 18 scenarios on 0.62.0, including the cross-axis refusal, so the rule stays checked. Recorded as `specification:one-owned-entity-per-state-axis` in the planning store." |
| #596 | comment, keep open | "Accepted, redesigned: the one-call batch lands as an `affects:` entry over the input list that may `moves:` or `deletes:` the row each element names, with a per-element `where:` (`story:each-entry-moves-the-rows-its-elements-name`, source format `ess/27`). Today's spelling, when the answers arrive one row at a time, is one command per row with one `when:` branch per verdict; that model validates and is witnessed on 0.62.0 (12 scenarios, 0 refusals). `each:` with `moves:` stays refused until the story ships." |
| #594 | comment, keep open | "Accepted, redesigned: a `kind: struct` type may name one field as its `shorthand:`, and a value may then be written as that field's scalar alone (`issue.get: getIssue` reads as `{op: getIssue}`); `ess/27`. No general untagged union. In progress on a unit branch." |
| #590, #591, #593 | comment, keep open until the release | "Fixed on an integration branch: `--schema-id-base <URI>` on `ess generate --kind schema` (#590), YAML instance files read as the JSON they denote (#591), `x-ess-*` annotations retained on import (#593). Ships in the next release." |
| #592 | comment, keep open | "Accepted as proposed: the projection gains `pattern: ^<prefix>[<class>]*$` beside `x-ess-alphabet` for alphabets in the Basic Multilingual Plane; planned as `story:alphabet-projects-an-anchored-pattern`, after the three sibling fixes." |
| #584 | comment, keep open | "Accepted, redesigned: option 1 is the answer. `sha256`, `base64url` and `ascii` join the expression language as a text/bytes family in the G1 wave (`story:digest-and-encoding-functions-in-expressions`); the ordering half of option 2 is `story:external-branch-narrowed-by-subject-guard` under the G4 wave. Until then the plain method stays witnessed and S256 stays an `external:` cause." |
| #572 | comment, keep open | "Scheduled as wave 4 of the next release plan; the suite-level encoding rides the batch's suite format pair if it lands before the release, otherwise the next pair." |
| #542, #543, #544, #545, #546, #547, #549, #552 | comment, keep open until the release; close with `Fixes` lines in the release PR | "Implemented on the 2026-10-13 integration, now rebased onto 0.62.0 (`ess/27`, suite `/52`–`/53`, `ess-diff/20`); being gated for the next release." |
| #551 | comment, keep open | "Design accepted (`exists: {in: {entity: <E>}, as: <b>, that: <P>}` in value selectors and row-set guards, `ess/27`); implementation in progress on the next integration pass." |
| #554, #560, #561, #564 | no action | already the visibility issues of their epics; the comments of 2026-10-13 stand |

## E. What could not be determined

- Whether the gates of `integrate/ess-wave-20261017` (`f8a206904`) were run: no commit body says.
- Whether `unit/ess-u594` and the port compile on their heads: both hand-back notes say no build
  ran after the last edits.
- Whether any session still owns the port tree, `ess-u594` or `ess-u551-impl`: no commit
  after 2026-10-17 and no lease visible to `worktree status`.
- What `story:a-scalar-to-record-input-change-is-breaking-for-callers` and
  `story:browser-fixture-startup-deadline` shipped as (no issue reference).
- Whether the 44 residual lines of `integrate/ess-issues-20261013` were dropped on purpose at the
  0.60.0 cut: they are on `origin/main` now, so the question is moot for the branch's fate.
- Whether the nine `ess/27` claimants fit the one-format-per-decision policy
  (`docs/design/generic-language-features.md` § Policy 2): the batch shares one number as the
  operator asked; the policy's count line in the next CHANGELOG section will show the share.
