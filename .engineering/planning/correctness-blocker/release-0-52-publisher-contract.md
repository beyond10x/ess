---
format: aep.planning-md/3
id: correctness-blocker:release-0-52-publisher-contract
kind: correctness-blocker
status: open
title: Publisher batch cap and closed-publisher behavior need PR404 before 0.52.0
relations:
- blocks: task:release-0-52-0-20261003
withholds: test_result
revision: 1
---
## Observed release hold

Existing PR398 merged at27b1ef5075666a52e256c06c2dee47a172c3d95f through the organization App after all15 checks succeeded. Its tree equals reviewed head5a5ac7f74d0ffdfcb9d91d395053620729b78254 exactly, b5a81028e22223633edf6aa44ac91867a298bede. PR403 closed as superseded after its source/evidence was preserved there. No0.52.0 tag was pushed.

The transport owner subsequently opened PR404, head1998a0a870240c613722bf9881f29da9cfe71b41, correcting two generated-publisher contract defects. The release coordinator read its exact five-file diff and regression tests: Rust can emit more than max_items when acknowledgements are slow; Go PublishNow can publish after Close. The owner's PR reports both tests fail before the fix and pass afterward, with all four generated-publisher tests and scoped Clippy passing. This is attributed owner evidence, not a claim of an independent local execution by the release integrator.

## Clears when

The owner brings the existing PR404 up to date with the release main, preserves the two regressions and correction, and lands the exact corrected candidate after required green checks. Then the release integrator runs final task check and site-lab/site-build on the delivered source and verifies the eventual exact tag, release checks, GitHub Release and four archives plus SHA256SUMS. No extra release PR is needed. Transport/publisher source remains third-session ownership.

## Coordination and partial local gate

A bot-authenticated coordination note was delivered to https://github.com/beyond10x/ess/pull/404 asking for current-main integration and existing-PR delivery before the tag. No direct acknowledgement by the live Claude session is inferred.

The prior local task check at5a5ac7f74 passed formatting/Clippy and completed198 test groups:1566 passed,0 failed,2 existing ignored. The tutorial_page group passed all8 cases with network steps enabled. This run paused at the10GiB free-space floor and was then deliberately stopped because required404 changes would alter the final release source; task exit201 is an interruption, not a completed gate. Assigned release-final evidence retains candidate.txt, check.log, check.exit and check-interruption.txt. Package-scoped Cargo cleanup removed21.8GiB of verified owned artifacts after all processes stopped; source, logs and review evidence remain. The final combined local gate and tag remain pending.
