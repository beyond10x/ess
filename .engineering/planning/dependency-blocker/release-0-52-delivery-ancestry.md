---
format: aep.planning-md/3
id: dependency-blocker:release-0-52-delivery-ancestry
kind: dependency-blocker
status: cleared
title: Gates refuses GitHub update-branch ancestry in the release candidate
relations:
- blocks: task:release-0-52-0-20261003
withholds: verification
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-10-06T09:44:39Z", actor: "human:timo", revision: 4}
---
## Observed refusal

After all source gates passed, the coordinator attempted the repository-supported native prebuild by publishing exact main e68684efb6a4ac22052c77d3ed8292fd44f9ace5 to queue/0.52.0-e68684efb. The common scan succeeded and retained a signed receipt, but `b10x-gates publish` exited 1 with `merged pull request missing or ambiguous`. No queue branch or package workflow was created. The retained prebuild-publish.log is authoritative; a passing common scan is not publication authorization.

## Read-only provenance diagnosis

The coordinator earlier used the bot-authenticated GitHub update-branch endpoint for PR 404. It created commit 258594c8602fa9e6d372ed5a869ad478d7e0c382 with the exact bot author, GitHub committer, and parents 1998a0a870240c613722bf9881f29da9cfe71b41 and 27b1ef5075666a52e256c06c2dee47a172c3d95f. The final PR merge e68684ef includes that update commit in its ancestry. GitHub's associated-PR response for 258594c86 lists PR 404, whose actual merge_commit_sha is e68684ef, not 258594c86. The response for e68684ef identifies exactly one completed merge, PR 404. PR 405 remains closed and unmerged.

The inspected Gates published_merge verifier traverses the complete candidate DAG and requires each GitHub-committed exception to identify exactly one completed PR merge at that commit. Its missing-match error is the observed refusal. The inspected push hook uses the same ancestry verifier. This identifies the update-branch ancestor as the unsupported provenance; it is not a product test failure. Creating that ancestor was the coordinator's integration mistake. The source tree and all green test evidence remain valid, but do not discharge this delivery gate.

## Required resolution

The delivery-policy/tooling owner must supply an approved way to establish the recorded update commit's authority or explicitly authorize a suitable repair. Revalidate through the same Gates publication path after resolution. Do not route the rejected publication through another tool, change the trusted baseline, weaken verification, or rewrite main on this session's authority. No such action was attempted. The independent publication-identity decision also remains open. This blocker does not claim that a tag push was attempted; none was.

## Authorized resolution

The operator explicitly approved implementing and independently reviewing Gates support for bot-created GitHub update-branch provenance, preserving existing checks and main history. Gates story:github-update-branch-provenance owns the fix. The blocker remains open until the reviewed delivered tool verifies the original refused candidate ancestry through the same Gates path.

## Reviewed source repair delivered, tool release still held

Gates PR beyond10x/gates#27 merged through b10x-bot at 47c0686c6f635d4d2760bc3a36b3f7974797d323. Its tree matches reviewed candidate abe6b942d979dbed160111e2e3c3bc908b7f12f9. Final local gate passed 143 tests with seven existing ignored; required remote checks passed. Independent adversary found no issues; guard-removal mutation caught the removed check.

The 0.1.12 static binary is built but unpublished. A required privacy scan refused private-identifiers at line 4177, diagnosed as a three-byte match in executable .text. No tool adoption, policy exception, scanner bypass, tag or release followed. The exact held binary SHA256 is d5c1663a2962828b2ba77dcafaf74f965cf84aead1b95becfd19ed0b591f4bd6. Operator disposition of that refusal and verified tool delivery are still required before retrying the original ESS publication.
