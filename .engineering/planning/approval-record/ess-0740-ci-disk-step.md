---
format: aep.planning-md/3
id: approval-record:ess-0740-ci-disk-step
kind: approval-record
status: approved
title: Free runner disk before the 0.74.0 test archive and release on green
summary: 'D120: the build-tests job gets the Free runner disk step; the 846a40d replay stands; tag at green CI'
tags:
- ess-0.74.0
- non-interactive
relations:
- decides: story:a-binding-tries-every-publisher-of-its-trigger
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T04:25:57Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-28 (decision D120): "go. Push 45939b7b9 (ci.yml only:
"Free runner disk" step on build-tests) to queue/ess-0.74.0; the 846a40d replay stands (no source,
test or synthesized byte changed). Tag, release, verify-release at green". Approved under the
orchestrator's grant (class C: bot commits, push, release).

## Why

PR 698's "Build workspace test archive" ran out of runner disk twice while it wrote the nextest
archive (run 40627742331, attempts 1 and 2). That is a gate failing twice on one cause, so the
wave's stop rule halted it. The job had no disk-freeing step; the test shards already had one.

## Conditions

- If the job fails on disk a third time, stop again and report the archive size.
- The adopter replay of 846a40ddf stands. The commit after it changes `.github/workflows/ci.yml`
  and this record only.
