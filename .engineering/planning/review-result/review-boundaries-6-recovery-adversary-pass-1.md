---
format: aep.planning-md/1
id: review-result:review-boundaries-6-recovery-adversary-pass-1
kind: review-result
status: active
title: Wave 6 recovery design adversary pass 1
relations:
- reviews: story:review-execution-recovery-design
revision: 1
---
unit: story:review-execution-recovery-design — committed 0a1be79f985b042b86c1eb8e76c07aecbe8bce57, base 009bf3cad2f01eaf1717ea737fa045ffb52d7f12
verdict: CONFIRMED
cases: executable cases not applicable (design-only); actual source inspection and document checks
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this pass and route the R25 wording correction; keep the implementation obligation open

1. git --no-pager diff --stat

```text
$ git --no-pager diff --stat
exit: 0
output: empty
```

The subject is committed. This attack changed no tracked or ordinary untracked file. Its only writes are the assigned ignored scratch report and inspection record. The full base-to-subject change is:

```text
$ git --no-pager diff --stat 009bf3cad2f01eaf1717ea737fa045ffb52d7f12 0a1be79f985b042b86c1eb8e76c07aecbe8bce57
 docs/design/review-execution-recovery.md | 144 +++++++++++++++++++++++++++++++
 1 file changed, 144 insertions(+)
exit: 0
```

2. Added cases and red output

Not applicable under the explicit prose-only adversary brief. No executable case, mutant, fake executor, production edit or live call was created or run. The finding below is a source-backed document inconsistency, not a failing executable recovery test. No executable before/after count or red result is claimed.

3. Actual verification

Read the exact 0.7.0 adversary charter, repository AGENTS.md, original unit brief, complete implementor report, story and implementation obligation, original F11, the complete committed design addition, and its cited executor, intent, strict-decoding, artifact-evidence and fake-executor sources.

The actual command outputs, UTC start/finish times and exits are retained in `target/review-boundaries-6/adversary-pass-1-inspection.txt`. All 13 recorded inspection/check commands exited 0. These are inspection commands, not 13 tests. Relevant exact results:

```text
$ git rev-parse HEAD
0a1be79f985b042b86c1eb8e76c07aecbe8bce57
exit: 0

$ git --no-pager diff --check 009bf3cad2f01eaf1717ea737fa045ffb52d7f12 0a1be79f985b042b86c1eb8e76c07aecbe8bce57
exit: 0
output: empty
```

A base-to-subject diff over all seven cited production/test files is empty and exits 0. The subject therefore introduces a document, while the inspected finite executor is unchanged from the unit base. The inspection record also retains SHA-256 hashes for the design and all seven cited source/test files.

No suite, formatter, Clippy, model validator or repository/site gate was run by this pass. Source/document review does not establish runtime recovery, model readiness or independent verification.

4. Judgement finding

| ID | What was measured | What reaches it | Verdict | Origin | Severity |
|---|---|---|---|---|---|
| R25-attribution | `docs/design/review-execution-recovery.md:94`: R25 says unknown removal attribution lasts until authorized absence is established, which conflicts with the design's rule that fresh observation cannot recover historical execution attribution. | The finite executor reaches `crates/edge/ess-cli/src/main.rs:1703–1713` for an authorized retirement and only receives the process result from `main.rs:2951–2959`. R25 explicitly governs the future success-before-durable-recording interruption and restart along that path. | CONFIRMED | introduced | warning |

The offending strategy text at line 94 is: “retain unknown attribution until authorized absence is established.” The design's claim definitions at lines 40–42, explanation at line 44, recovery requirement at line 55 and apply analogue R21 at line 90 all distinguish a fresh current-state observation from an earlier operation's attribution. R26 at line 95 also correctly forbids a fabricated removal effect.

An authoritative observation of absence can settle whether the release is absent at that observation point. It cannot establish that the earlier interrupted invocation caused the absence or identify its attempt. R25's “until” gives the observation a stopping condition for that historical uncertainty. This is a narrow wording inconsistency in a proposed acceptance strategy, not an allegation that an implemented receipt decoder or observer performed the unsafe inference. Neither facility exists in this subject.

Reachability is grounded in the existing `DeploymentCommand::Reconcile` retirement loop, plus the design's explicitly required future interruption boundary. The current caller can receive a successful uninstall acknowledgement; it stores no durable execution result. The future restart behavior is prescribed by R25, so this is a reachable design obligation rather than a fabricated executable configuration.

Proposed correction, confined to R25's strategy: retain unknown historical removal attribution even when a fresh authorized observation establishes absence; permit only an observed-absence claim unless separately admitted historical evidence establishes the earlier operation. Keep R26's zero-repeat-uninstall requirement. No receipt fields, implementation or broader design change is needed to resolve this wording.

The origin is introduced: the base-to-subject diff adds this entire document. The unchanged executor's absent durable history is the motivating pre-existing F11 weakness, already disclosed by the design; it is not counted as another finding here.

5. Attacks with no additional finding

- Desired/current equality: lines 8, 20, 44 and 52 reject applied-state/no-op inference from equal desired bytes; this agrees with the concrete weakness at `main.rs:1642–1652`.
- Target aliases, rebinding and baseline authority: R02 and the modeling table keep physical identity, credential scope, baseline ownership and service-to-release cardinality unresolved and require refusal instead of guessing.
- Failure before and after ORAS acquisition, chart selection and local writes: R05–R12 separate chart preparation from target mutation, preserve checksum refusal and include cold/warm restart strategies.
- Failed child versus unknown effect: R13–R17 distinguish failure to launch from a started process with an uncertain result; they preserve the completed prefix and forbid later mutation or guessed rollback.
- Missing, corrupt, unsupported, wrongly scoped and stale history: R04 and R18–R19 refuse unsupported claims; R16 and R21 require fresh matching observation without automatic replay.
- Equal desired after manual drift: R20 requires a fresh differing observation and a new repair decision; it does not authorize overwriting the manual change.
- Retirement sequencing and safety: R22–R28 retain the early removal flag guard, reverse baseline order, stop-on-failure, no unconditional repeat uninstall, replacement ownership checks and enforceable concurrency preconditions.
- Per-release versus global atomicity: the document limits `--atomic` to the single apply call at `main.rs:3036–3044`, accurately notes uninstall lacks those apply options, and does not claim coordinated rollback.
- Dry-run: requirement 2 and R03 permit only a local unverified preview with zero external calls and no cache/evidence writes; they do not require live observation.
- Typed-model boundary: every concrete identity, authority, ownership/cardinality, observation, concurrency, durability and result decision remains UNMAPPED; no receipt schema or implementation decomposition is introduced.
- Completion boundary: the design explicitly leaves F11 and the implementation obligation open, names future seams and matrix strategies, and makes no claim that the 29 matrix families are executed tests.

6. Paths written outside the worktree

None.

Assigned ignored scratch written by this pass:

- `target/review-boundaries-6/adversary-pass-1-inspection.txt`
- `target/review-boundaries-6/adversary-pass-1.md`

This completed report relinquishes writes. Root owns recording, correction routing, Git/AEP and worktree lifecycle operations.

```findings
- file: docs/design/review-execution-recovery.md
  line: 94
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: R25 says unknown removal attribution lasts until authorized absence is established, which conflicts with the design's rule that fresh observation cannot recover historical execution attribution.
```

