---
format: aep.planning-md/1
id: decision-blocker:ess-gates-historical-exception-locations
kind: decision-blocker
status: cleared
title: Resolve exact historical journal exception locations before publication
relations:
- blocks: story:ess-evolution-preservation-gate
revision: 2
---
## Decision required
The newly installed b10x-gates commit hook refuses three personal-paths findings in unchanged historical planning journal records. The configured adoption baseline is published main 24d2fe714958c8cde63ea78122c31e28bcc682bc. Its journal lines 1980, 1981 and 1982 have byte-identical content hashes to already enrolled historical exceptions at lines 1985, 1986 and 1987. The scanner binds both content hash and line number. New personal evidence paths in this follow-up were removed by replaying unpublished planning changes through AEP with portable references; a subsequent commit attempt now reports only these three historical findings.

## Proposed resolution
The Gates policy owner may add the three exact historical locations while preserving the existing exceptions needed by the concurrent Gates candidate. Every proposed content hash was independently checked against the configured published baseline. The prepared proposal is local-evidence:ess-evolution-20260910/ess-gates-historical-exception-proposal.json. No protected policy, scanner rule, or hook has been changed or bypassed. Approval or action from the policy owner clears this publication blocker. Rewriting published planning history is not the resolution.

## Ready work
The preservation validator, its tests, documentation and governed story are ready in managed worktree wt-e0849b5270f6. task check and task site-build passed; the story is implemented, but its follow-up commit and push have not succeeded. After the exact policy discrepancy is resolved, retry the bot commit, verify author and committer, publish, update the clean primary, and finish/GC the worktree. Retained evidence includes the complete compressed consumer run and the local gate logs under local-evidence:ess-evolution-20260910/.

## Resolution

The operator approved finishing publication after reviewing the three exact historical exception additions. Added only those three personal-paths exceptions at their verified baseline line numbers, preserving the existing content hashes, rules and other exceptions. The baseline records were independently checked against published main 24d2fe714958c8cde63ea78122c31e28bcc682bc. No source gate is rerun: the operator explicitly instructed use of the existing passing verification. This clears the policy discrepancy; normal commit and push hooks remain enabled.
