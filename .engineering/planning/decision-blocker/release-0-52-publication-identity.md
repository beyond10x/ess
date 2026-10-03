---
format: aep.planning-md/3
id: decision-blocker:release-0-52-publication-identity
kind: decision-blocker
status: open
title: Reconcile existing release publisher with the required organization bot identity
relations:
- blocks: task:release-0-52-0-20261003
withholds: verification
revision: 3
---
## Observed policy mismatch

The workspace operator instructions require every GitHub write, including release creation and edits, to use b10x-bot[bot]. Direct source publication, PR updates, comments and the PR398 merge used the required App. The current repository release workflow has a different existing publication identity: .github/workflows/release.yml, Create or reconcile the GitHub Release, assigns GH_TOKEN from github.token and runs release create/edit/upload. Read-only GitHub inspection confirms release0.51.0 author is github-actions[bot]. This is an observed existing workflow behavior, not a hypothetical credential risk.

## Decision needed before the tag

Either the operator permits this existing automated publisher for0.52.0 while all direct agent writes stay with b10x-bot, or the repository owner supplies an approved bot-authenticated release publisher and its required credentials/verification. No exception is inferred, no App key is copied into CI, and no tag is pushed merely to test who publishes it. Prepare and verify the final source candidate first; any requested exception must identify this exact existing workflow and the workspace rule. Source integration and tests can proceed independently.

## Ready candidate at the unresolved decision

All source verification is complete on current main e68684efb6a4ac22052c77d3ed8292fd44f9ace5: local `task check` and `task site-build` (including `site-lab`) exited 0, and main CI/Gate run 37093196094 succeeded. The source tree is clean. No 0.52.0 tag or release exists. The operator has not answered the existing publication-identity question, so this blocker remains open. The next action is to tag this verified commit and verify its release checks and assets only after the existing publisher is explicitly permitted or an approved bot publisher is supplied and verified. No additional source PR is needed for the candidate as it stands.
