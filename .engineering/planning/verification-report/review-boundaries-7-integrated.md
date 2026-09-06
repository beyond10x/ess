---
format: aep.planning-md/1
id: verification-report:review-boundaries-7-integrated
kind: verification-report
status: draft
title: Boundary wave 7 integrated verification
relations:
- verifies: story:review-expression-typechecking
- verifies: story:review-openapi-semantic-accounting
revision: 2
---
## Subject and measured integration

Clean combined source `8271c3ec92adfe9e39d8fe877933e2a4199b2f46` contains expression unit `1b018bf38c49a644754f2f91be9fde2dc4c9f9df` and OpenAPI unit `7468e7a01b6eab0abe32cdb56e1718a9a0f67fa4`, preserving incoming normalization work. All eight ordered offline `task check` steps ran individually with their own observed exits, followed by site-build and planning. All ten returned zero. The workspace runner reports 1,873 passed, zero failed or ignored, across 145 summaries; the previous published combined baseline ran 1,776 cases.

## Unit results and limits

Expression implementation reached 683 cases, then first independent review added 14 cases and all 697 passed. The sealed Specification remains closed to public mutation; module-local revalidation, actual public compiler calls and the existing compiler-entry fence provide the relevant admission evidence. No unsafe testing aperture or fabricated external mutation is introduced. Type rules are shared while conformance projection and witness limitations remain separate. The complete immutable review reports no findings and retains its initial fixture setup failures.

OpenAPI implementation reached 150 cases. First review reached 163, with four failures caused by one introduced unit-variant unknown-field defect. The bounded correction preserved the legacy reader, fixed raw checked-interface admission and reached 165 passing cases. Second and final review added seven cases; all 172 passed, with no ignored cases. Both review reports are immutable. The CLI comparison records one resolved finding and no carried or new findings. Formatting and strict Clippy passed in both units. There was no third full review.

Checked import envelopes retain original-byte source identity and complete accounting. Gaps and unresolved sites refuse checked projection; known annotation omissions are separate. The low-level legacy structural reader/projector remains available with its historical limitations; native generation and incoming normalization behavior are separate. No full OpenAPI implementation or external consumer deployment is claimed.

## Individual gate evidence

```json
{
  "subject": "8271c3ec92adfe9e39d8fe877933e2a4199b2f46",
  "steps": [
    {
      "step": "fmt-check",
      "argv": [
        "task",
        "fmt-check"
      ],
      "started": "2026-09-06T07:57:24.695670+00:00",
      "finished": "2026-09-06T07:57:25.867562+00:00",
      "seconds": 1.171903467969969,
      "exit": 0
    },
    {
      "step": "clippy",
      "argv": [
        "task",
        "clippy"
      ],
      "started": "2026-09-06T07:57:25.867748+00:00",
      "finished": "2026-09-06T07:57:34.834178+00:00",
      "seconds": 8.96644417301286,
      "exit": 0
    },
    {
      "step": "test",
      "argv": [
        "task",
        "test"
      ],
      "started": "2026-09-06T07:57:34.834387+00:00",
      "finished": "2026-09-06T07:58:34.506978+00:00",
      "seconds": 59.67260195303243,
      "exit": 0,
      "counts": {
        "passed": 1873,
        "failed": 0,
        "ignored": 0,
        "summaries": 145
      }
    },
    {
      "step": "doc-check",
      "argv": [
        "task",
        "doc-check"
      ],
      "started": "2026-09-06T07:58:34.507780+00:00",
      "finished": "2026-09-06T07:58:42.843723+00:00",
      "seconds": 8.335954361013137,
      "exit": 0
    },
    {
      "step": "example-check",
      "argv": [
        "task",
        "example-check"
      ],
      "started": "2026-09-06T07:58:42.843947+00:00",
      "finished": "2026-09-06T07:58:49.647273+00:00",
      "seconds": 6.803342769970186,
      "exit": 0
    },
    {
      "step": "projection-check",
      "argv": [
        "task",
        "projection-check"
      ],
      "started": "2026-09-06T07:58:49.647510+00:00",
      "finished": "2026-09-06T07:59:06.710622+00:00",
      "seconds": 17.063133124029264,
      "exit": 0
    },
    {
      "step": "release-check",
      "argv": [
        "task",
        "release-check"
      ],
      "started": "2026-09-06T07:59:06.790650+00:00",
      "finished": "2026-09-06T07:59:07.210350+00:00",
      "seconds": 0.4204711669590324,
      "exit": 0
    },
    {
      "step": "action-check",
      "argv": [
        "task",
        "action-check"
      ],
      "started": "2026-09-06T07:59:07.210766+00:00",
      "finished": "2026-09-06T07:59:07.470639+00:00",
      "seconds": 0.2598937660222873,
      "exit": 0
    },
    {
      "step": "site-build",
      "argv": [
        "task",
        "site-build"
      ],
      "started": "2026-09-06T07:59:07.471105+00:00",
      "finished": "2026-09-06T07:59:30.020019+00:00",
      "seconds": 22.54892792296596,
      "exit": 0
    },
    {
      "step": "planning",
      "argv": [
        "aep",
        "plan",
        "artifact",
        "validate"
      ],
      "started": "2026-09-06T07:59:30.020247+00:00",
      "finished": "2026-09-06T07:59:31.581260+00:00",
      "seconds": 1.5610263909911737,
      "exit": 0
    }
  ]
}
```

The site lane actually executed its browser source/run checks, WASM build and Docusaurus build. npm printed its existing Git-dependency integrity/deprecation and install-script warnings; the build exited zero. Planning is valid and retains 16 findings-list advisories: the validator reports both absent blocks and valid empty lists through the same branch. Full output is retained alongside the gate results.

Builds used the coordinator's own target, worktree-local temporary/Go caches, four jobs, disabled incremental/debug output, offline Cargo and Node 24.20.0. The old shared sccache socket no longer listened, so direct rustc was used without starting a replacement daemon. Ordinary existing dependency caches remain incidental infrastructure effects. Every gate process was observed complete.

## Retained evidence and ownership

All exact commands, times, logs and exits are in `target/review-boundaries-7/final-gate`. The unit evidence archives are `/home/timo/.cache/ess-review/2026-09-06-resume/expression-wave7-evidence.tar.gz` (182 files; SHA-256 `8d0117c81655f6c0d976923e4e56e4bd2de8e3055b91c7e4d86e60eebef61bd3`) and `openapi-wave7-evidence.tar.gz` (3,131 files; SHA-256 `89635d7bf76da8e6bd751db40539d815aac52614b829c7580bdecec835052eda`). Adjacent JSON manifests bind every included file and enumerate excluded disposable compiler/dependency cache directories. No managed tree has yet been removed.

The resumed harness does not expose per-agent token or tool-use counters; those costs are unavailable rather than estimated. Actual command durations remain in each report and gate record. Independent agents own only their assigned source/tests/scratch; the coordinator owns all planning, merge, publication and lifecycle operations.

The AEP reader prerequisite remains separately active and under review. This gate closes only the two Wave 7 story implementations; source publication, remote CI, public delivery and managed cleanup are separately recorded outcomes. No release tag or version bump is implied.

## Final documentation correction

After the complete gate, source `5d5e87fc70e021ce38477ab71c842a2063a845a3` corrects the legacy reader's closed-DTO wording in the internal catalog and public reference and records confirmed story scope. Root verified the exact four changed paths: those two documents, the story and its CLI-written journal. No Rust, test fixture, manifest, lockfile, generated runtime or task changed. The complete site-build lane was repeated on that clean commit and exited zero; no second Rust suite is claimed. Publication metadata that follows changes only internal planning records.

```json
{
  "subject": "5d5e87fc70e021ce38477ab71c842a2063a845a3",
  "argv": [
    "task",
    "site-build"
  ],
  "started": "2026-09-06T08:01:36.528442+00:00",
  "finished": "2026-09-06T08:01:52.785015+00:00",
  "seconds": 16.25657180591952,
  "exit": 0,
  "log": "/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-7/final-gate/site-build-reference-correction.log"
}
```

## Combined gate after concurrent normalization publication

Combined source `d482457a12bb299d761bdfb957743d75e9d9f67c` includes incoming reviewed normalization `60ffcb2238ffef3a48d0db9555b6f2ca709ca2f7`. Every one of the ten ordered integration lanes exited zero. The workspace runner executed 1,880 cases, zero failed or ignored, across 146 summaries. Site-build executed its browser, WASM and Docusaurus steps. Complete command output and per-step exit records are in `target/review-boundaries-7/base64-combined-gate`; the earlier 1,873-case result remains attributed to its earlier subject.

The planning-only merge conflict was resolved by preserving incoming's exact 1,156-line canonical journal and replaying 23 local AEP operations. All 134 expected artifacts were byte-equal before recording this result. No review was discarded or edited, and the normalization session's new drafts remain separately owned. Both Wave 7 story implementations remain supported by this newer combined source.

```json
{
  "subject": "d482457a12bb299d761bdfb957743d75e9d9f67c",
  "steps": [
    {
      "step": "fmt-check",
      "argv": [
        "task",
        "fmt-check"
      ],
      "started": "2026-09-06T08:09:51.742486+00:00",
      "finished": "2026-09-06T08:09:52.979309+00:00",
      "seconds": 1.2368389279581606,
      "exit": 0
    },
    {
      "step": "clippy",
      "argv": [
        "task",
        "clippy"
      ],
      "started": "2026-09-06T08:09:52.979506+00:00",
      "finished": "2026-09-06T08:09:56.475015+00:00",
      "seconds": 3.495528078987263,
      "exit": 0
    },
    {
      "step": "test",
      "argv": [
        "task",
        "test"
      ],
      "started": "2026-09-06T08:09:56.475327+00:00",
      "finished": "2026-09-06T08:10:39.027567+00:00",
      "seconds": 42.55225454492029,
      "exit": 0,
      "counts": {
        "passed": 1880,
        "failed": 0,
        "ignored": 0,
        "summaries": 146
      }
    },
    {
      "step": "doc-check",
      "argv": [
        "task",
        "doc-check"
      ],
      "started": "2026-09-06T08:10:39.028378+00:00",
      "finished": "2026-09-06T08:10:41.301416+00:00",
      "seconds": 2.273051730939187,
      "exit": 0
    },
    {
      "step": "example-check",
      "argv": [
        "task",
        "example-check"
      ],
      "started": "2026-09-06T08:10:41.301613+00:00",
      "finished": "2026-09-06T08:10:46.734017+00:00",
      "seconds": 5.432418846059591,
      "exit": 0
    },
    {
      "step": "projection-check",
      "argv": [
        "task",
        "projection-check"
      ],
      "started": "2026-09-06T08:10:46.734230+00:00",
      "finished": "2026-09-06T08:10:47.335246+00:00",
      "seconds": 0.6010289900004864,
      "exit": 0
    },
    {
      "step": "release-check",
      "argv": [
        "task",
        "release-check"
      ],
      "started": "2026-09-06T08:10:47.335456+00:00",
      "finished": "2026-09-06T08:10:47.435928+00:00",
      "seconds": 0.1004854430211708,
      "exit": 0
    },
    {
      "step": "action-check",
      "argv": [
        "task",
        "action-check"
      ],
      "started": "2026-09-06T08:10:47.436153+00:00",
      "finished": "2026-09-06T08:10:47.481661+00:00",
      "seconds": 0.04552171495743096,
      "exit": 0
    },
    {
      "step": "site-build",
      "argv": [
        "task",
        "site-build"
      ],
      "started": "2026-09-06T08:10:47.481888+00:00",
      "finished": "2026-09-06T08:11:04.135840+00:00",
      "seconds": 16.653973231907003,
      "exit": 0
    },
    {
      "step": "planning",
      "argv": [
        "aep",
        "plan",
        "artifact",
        "validate"
      ],
      "started": "2026-09-06T08:11:04.136252+00:00",
      "finished": "2026-09-06T08:11:05.721155+00:00",
      "seconds": 1.5849185179686174,
      "exit": 0
    }
  ]
}
```
