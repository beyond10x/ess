---
format: aep.planning-md/1
id: story:deleting-a-scratch-tmpdir-breaks-sccache-for-every-other-agent
kind: story
status: draft
title: Deleting a scratch TMPDIR breaks sccache for every other agent
revision: 1
---
# Deleting a scratch TMPDIR breaks sccache for every other agent

A wave dispatches several agents at once, each with its own `TMPDIR` under
`/var/tmp/ess-wave-<n>/scratch/<unit>/tmp`, and each brief told the agent to delete that directory
before finishing — because ESS fixtures leave one directory per test run and 24,490 of them had
accumulated by 2026-09-12, filling the disk to 99% and failing a lane with `StorageFull`.

**`sccache` runs as a long-lived daemon, and it holds the `TMPDIR` of whichever agent first started
it.** When that agent finishes and deletes the directory, every later `cargo` invocation in every
worktree fails:

```
sccache: error: Failed to create temp dir ... at path
"/var/tmp/ess-wave-25/scratch/unit3-adversary2/tmp"
```

Exit 254, and it hits the *nested* cargo runs — `schema-contract` and `ess-cli-project` in this
case, five lanes red in a package that the agent seeing them had not touched.

Found by wave 25's unit-2 correction round, which attributed five red lanes rather than reporting
them, recreated the directory, watched the same five go green, and **left the directory in place
deliberately**, noting that deleting it re-breaks the machine.

Confirmed live afterwards: the running daemon's environment read
`TMPDIR=/var/tmp/ess-wave-25/scratch/unit2-correction1/tmp` and
`SCCACHE_STARTUP_NOTIFY=…/unit2-correction1/tmp/sccacheK2Yfat/sock`, pointing at a directory that
agent had just deleted — so at that moment every nested-cargo test on the machine was broken, in
every tree, including trees whose agents were still running.

## Two instructions in tension, both written by the coordinator

| instruction | reason it exists | what it breaks |
|---|---|---|
| "delete your `TMPDIR` before you finish" | 24,490 abandoned fixture directories filled the disk | sccache, for every concurrent agent |
| "`export RUSTC_WRAPPER=sccache` before building" | cold builds cost ~30 minutes each | — |

Neither is wrong on its own. Together they make the first agent to finish break the others, and the
breakage surfaces in a package the victim did not touch, which is the hardest kind to attribute.

## Acceptance

A wave can dispatch concurrent agents that each clean up after themselves without breaking any
other agent's build, and an agent that finishes does not have to know whether it was the one that
started the daemon.

The obvious candidates, none free:

- give sccache a stable `TMPDIR` of its own, set before any agent starts, so no agent's cleanup can
  reach it;
- have the coordinator reclaim scratch rather than each agent, which costs the coordinator a step
  per unit and delays reclamation until the wave closes;
- stop sharing one daemon, which costs the cache hits the wrapper exists for.

## Scope

- the wave coordinator's brief template — `inferred`; it is prose in this session, not a file
- `home-path:sha256:4727a71f2fe1dfaac07d79d5b2777a82a107e8522dd5cf9b60ea0a732f1ddf57` — `inferred`, if the stable-`TMPDIR` rule belongs beside the `/tmp` quota rule
