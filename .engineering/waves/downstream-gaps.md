# Downstream gaps — waves

Epic: `epic:downstream-reported-gaps` (serves `vision:O2`). Skill: aep:implementing 0.16.0, wave mode.
Waves from `aep plan artifact waves --kind story` on the typed scopes (2026-09-30).
Plan critics: rounds 1 and 2 recorded as `review-result:downstream-gaps-*-round-{1,2}`.

## Coordinator-owned files

- `CHANGELOG.md`, `changes/`, derived outputs (`generated/`, `suites/generated/`, `schemas/generated/`,
  `website/docs/reference/diagnostics.md`), `crates/edge/ess-cli/src/main.rs`, workspace `Cargo.toml`.

## Units

| Wave | Story | Branch | Tree id | Build dir | Scratch | Stage |
|---|---|---|---|---|---|---|
| 1 | feature-request-257 | impl/gaps-257 | gaps-257 | b10x-target/gaps-257 | ess-gaps/257 | merged 2a8ffd97b |
| 1 | feature-request-265 | impl/gaps-265 | gaps-265 | b10x-target/gaps-265 | ess-gaps/265 | merged |
| 1 | feature-request-271 | impl/gaps-271 | gaps-271 | b10x-target/gaps-271 | ess-gaps/271 | merged |
| 2 | feature-request-229 | impl/gaps-229 | gaps-229 | b10x-target/gaps-229 | ess-gaps/229 | merged |
| 2 | feature-request-275 | impl/gaps-275 | gaps-275 | b10x-target/gaps-275 | ess-gaps/275 | merged |
| 2 | feature-request-276 | impl/gaps-276 | gaps-276 | b10x-target/gaps-276 | ess-gaps/276 | merged |
| 3 | feature-request-270 | impl/gaps-270 | gaps-270 | b10x-target/gaps-270 | ess-gaps/270 | merged |
| 3 | feature-request-278 | impl/gaps-278 | gaps-278 | b10x-target/gaps-278 | ess-gaps/278 | merged |
| 4 | feature-request-266 (after 265 and 278 merge: synthesize.rs, subject_fact.rs) | | | | | planned |
| 4 | feature-request-267, -272 | | | | | planned |
| 5 | feature-request-268 | | | | | planned |
| 6 | feature-request-269 | | | | | planned |
| 7 | feature-request-273 | | | | | planned |
| 8 | feature-request-279 | | | | | planned |
| 9 | feature-request-280 | | | | | planned |

Recomputed 2026-10-01 over the proposed stories after #275, #276, #278, #279 and #280 were scoped; waves 3-9 serialise on `crates/verify/ess-conformance/src/synthesize.rs`.

## Release cut

Release 0.49.0 from `integrate/gaps-w1` once #265 and #278 merge: #229 (ess/20), #257, #265, #270, #271, #275, #276, #278. Wave 4 onward ships in the release after (operator, 2026-10-01: release soon when much is fixed).

## Backlog order (reconciled 2026-10-01, after the fit review of every open request)

Base spec before UI spec (operator, 2026-10-01). Within base spec: defects and regressions, then accepted gaps, then deferred. Formats: ess/20 shipped alone in 0.49.0. On 2026-10-02 the operator placed issue389 on the fast lane: its one-time response disclosure contract takes the next unshipped major, ess/21. Every previously accepted syntax change below remains one coordinated bundle and moves together to ess/22; its semantics and dependency ordering are unchanged.

| priority | stories |
|---|---|
| 1 base defects | 251 (regression since 0.43), 266, 267, 272, 273, 274, 279, 280, 288, 289 |
| 2 base gaps, synthesis and tooling | 287 (singleton, no new key), 222, 212, 221 with 223, 236, 231 (lowerable-subset table now) |
| fast lane, ess/21 syntax | 389 one-time response disclosure, with validation and executable non-disclosure checks |
| 3 base gaps, ess/22 syntax | 282, 283, 285, 286, 268 (closes 194), 269, 200, related-record-effects, family F: 225, 228, 233, 237, 244a |
| deferred (decision-blocker) | 197 (refusal-may-change-state), 244b (calendar-window-time-zone), 231 part 2 (entity-core-lowering-features) |
| 4 UI spec | 284, 281 item 1, ui-spec-style-tokens |

Closed or merged by the reconciliation:
- #220 was fixed by #188; the story is archived and the issue closed.
- #194 goes into #268.
- #225 duplicates #233 item 3.
- #233 item 5 and #244a are one need, solved by A3.
- #229 follow-up counts and #228 go into family F B.
- #281 item 2 is declined with the idiom `{name: board, remove: true}`.
- Filed from the review: #288 and #289.

## Wave w2 (integrate/gaps-w2, from 0.49.0 at f86180f30)

Release on merge (operator, 2026-10-01: release often). Units edit disjoint files.

| unit | branch | tree | build dir | status |
|---|---|---|---|---|
| feature-request-272 | impl/gaps-272 | gaps-272 | b10x-target/gaps-272 | merged |
| feature-request-287 | impl/gaps-287 | gaps-287 | b10x-target/gaps-287 | dispatched |
| feature-request-306 | impl/gaps-306 | gaps-306 | b10x-target/gaps-306 | merged |
