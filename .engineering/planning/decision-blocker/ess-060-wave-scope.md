---
format: aep.planning-md/3
id: decision-blocker:ess-060-wave-scope
kind: decision-blocker
status: cleared
title: Units of wave 2026-10-11j for 0.60.0
relations:
- blocks: release-plan:ess-060
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-11T11:03:10Z", actor: "human:timo", revision: 3}
---
## Question

Which units make wave 2026-10-11j, released as 0.60.0? Open issues on 2026-10-11: https://github.com/beyond10x/ess/issues/493 (older than 3 days), https://github.com/beyond10x/ess/issues/515, https://github.com/beyond10x/ess/issues/525. Builds run one at a time.

## Options

- **A.** Two tracks and one unit, run so only one build is live at a time.
  - Track 1, https://github.com/beyond10x/ess/issues/515 (`story:mutate-covers-subject-related-guards-sets-and-authored`), serial because all three share `mutate.rs`: j1 guard mutants for `when_subject` and `when_related`; j2 a `sets-drop` site per `sets:` entry, or the audit names its reason; j3 `mutate --scenarios <dir>`. Adversary on j3.
  - Track 2, `ess-http/1` stage 1 for https://github.com/beyond10x/ess/issues/493: j0 the design page `docs/design/http-presentation-binding.md` (branch `design/ess-http-binding`), with its six stage-1 open questions answered; j4 envelope, reader and `ess-http-ir/1` in a new crate under `crates/specify/`; j5 path templates, parameters and collisions; j6 `refines:`; j7 OpenAPI with a binding, the `ess specify http` verbs, schema and reference page. Adversary on j5.
  - j8 `story:go-synthesis-breaks-import-cycles-between-domains`.
  - Issues after release: 2 (#493 open until stage 2, #525).
- **B.** A plus stage 2 (request encoding, bodies, statuses, headers, redirects; about 7 units). Closes #493; issues after release: 1; the wave about doubles.
- **C.** Track 1 and j8 only. Issues after release: 2; #493 waits a wave more.

`story:stored-field-equals-returned-response-value` (https://github.com/beyond10x/ess/issues/498) is not in any option: its fit review rides `ess/24`, which shipped in 0.58.0, so it needs `ess/25` and a refreshed fit review first. It is proposed as the first unit of the next wave.

## Recommended answers to the design page's stage-1 questions

1. OpenAPI with a server-chosen route: refuse it by name; a marked path can be admitted later.
2. A binding that must route every operation: not in stage 1.
3. A literal segment beside a variable: refuse the overlap (`ESS-HTTP-011`).
4. `refines:` pins the refined file's bytes, as the specification source pin does.
5. `ess verify diff` over two bindings: one unit in stage 2.
6. A view on `post`: not in stage 1.

## Recommendation

A: it closes #515, starts #493 on a page whose stage-1 keys stage 2 does not have to change, and delivers the consumer's Go gap, with every build under the per-build limit.

## Decided

A, with the six stage-1 answers as recommended. Wave 2026-10-11j: j1-j3 `story:mutate-covers-subject-related-guards-sets-and-authored` (Fixes https://github.com/beyond10x/ess/issues/515), serial on `mutate.rs`, adversary on j3; j0 the design page with the answers; j4-j7 `ess-http/1` stage 1, adversary on j5; j8 `story:go-synthesis-breaks-import-cycles-between-domains`. One build at a time, at most 10G of build output per unit, the target discarded between units. One integration branch and one pull request, then 0.60.0 after a green full pre-release suite. `ess-http/1` stage 2 is the first work of the next wave, ahead of https://github.com/beyond10x/ess/issues/498, so https://github.com/beyond10x/ess/issues/493 closes then.
