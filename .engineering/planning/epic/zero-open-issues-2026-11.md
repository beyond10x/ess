---
format: aep.planning-md/3
id: epic:zero-open-issues-2026-11
kind: epic
status: draft
title: 'Zero open issues: resolve every open GitHub issue and every open defect in the store'
summary: Resolve the 48 GitHub issues open on 2026-11-09 and every non-terminal defect draft, in waves on one integration branch closed by one pull request
tags:
- integration
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Every GitHub issue of `beyond10x/ess` open on 2026-11-09 is resolved, and every open defect in this
planning store is fixed or recorded as no longer reproducing. One integration branch carries every
wave of this epic, and one pull request closes the issues it resolves.

Operator goal, verbatim (2026-11-09): "zero remaining issues on github, all bugs fixed from planning
store". Method, verbatim: "use /planning and /wave and /worktree:managing-worktrees - but in the end
provide 1 single PR to gh. make sure all are below 1 big epic for integration via bottoken from
./atlas".

## Scope

- The 48 issues open on 2026-11-09: #498 #554 #561 #564 #636 #637 #638 #640 #642 #650 #663 #674
  #678 #680 #681 #694 #705 #706 #708 #722 #723 #726 #727 #728 #729 #730 #731 #732 #733 #734 #735
  #736 #738 #739 #740 #750 #752 #753 #754 #755 #756 #757 #758 #759 #760 #764 #765 #766. Each is
  triaged with `.agents/skills/assessing-external-requests/SKILL.md`; its stories carry the issue
  as an external ref, a `## Fit review` and a `## Decisions`.
- Every non-terminal story or task whose body describes a defect in shipped behaviour (150 drafts
  classified on 2026-11-09). Each is reproduced on 0.92.0 before it enters a wave; a draft that does
  not reproduce gets its evidence in its body and leaves this epic.
- Order: bugs first (#765, #766), then the synthesis refusals (#705-#740), then the defect drafts,
  then the enhancements.

## Not in scope

- A release, a tag or a version bump. Releases follow `AGENTS.md` § Hotfix and the release gate.
- Feature drafts that cite no open issue (139 on 2026-11-09).

## Acceptance

`gh issue list --repo beyond10x/ess --state open` lists no issue that this epic's stories resolve
in code; any issue left open has a typed blocker artifact naming why it cannot be resolved in this
repository. Every story decomposing this epic is `implemented` or `archived` with its evidence.
