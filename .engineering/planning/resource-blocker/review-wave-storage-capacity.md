---
format: aep.planning-md/1
id: resource-blocker:review-wave-storage-capacity
kind: resource-blocker
status: cleared
title: Shared disk capacity prevents remaining wave verification
relations:
- blocks: story:review-secret-sanitization
- blocks: story:review-report-reader-validation
withholds: test_result
revision: 3
---
## Withheld verifier and observed condition

The remaining adversarial package checks and integrated `task check` / `task site-build` cannot start under the recorded disk reserve. This is a storage resource block, not a missing implementation approval; `approval-record:review-remediation-standing-implementation` remains in force.

On 2026-09-05, free bytes fell from 23,002,062,848 before unit builds to 11,868,954,624 while unit targets together used only 168 KiB. The coordinator reduced concurrency to one Cargo unit and changed its planning estimate to an 8,589,934,592-byte reserve, about four times its existing full integration target. Package implementation gates completed under that plan. Later storage fell to 3,398,778,880, then 910,450,688 bytes while our compilation was stopped. Reclaiming only the coordinator's separate baseline measurement target removed 470 files / 281.6 MiB; a following reading was already 288,198,656 bytes free because other activity continued.

Read-only `pidstat -d -h` observed active Cargo/rustc/sccache I/O; one rustc cwd was a separate managed Substrate checkout. No other process was killed, and no other worktree, cache or source was removed. Managed GC dry-run and finished-tree size inventory established no useful finished ESS reclamation. The coordinator's raw resource observations are in `target/review-boundaries-1/`.

## Clear when

At least 12 GiB free capacity is stable with competing builds paused or space reclaimed by their owner. Recheck actual capacity and task target sizes, record clearance, then resume the existing units from their current heads and test additions. This headroom includes the 8 GiB reserve and measured integration/build working space; do not rerun completed implementation steps or claim the unrun integration gate succeeded.

## Preserved state

Integration branch `wave/review-boundaries-1` holds opening commit `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b`. Unit code commits are `4851c9301a06a9557a36b23086aeee48ded26b03` and `ebe5e9dd7aeb1737fd7d5a2671d56824b5824eb5`; adversarial test additions remain in their managed unit trees and are copied as patches into the wave's engineering evidence. Neither unit merged and neither story is terminal. No commits were pushed and no release began.

## Clearance observation

After the storage incident, fresh readings were 54,895,435,776 and 53,885,259,776 available bytes, both well above the 12 GiB resumption threshold. The coordinator resumes the saved wave with one Cargo unit at a time and the recorded reserve. Implementation reruns are unnecessary; only pending adversarial routing and verification resume. The initial block remains historical evidence and is not a gate result.