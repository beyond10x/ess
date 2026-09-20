---
title: "0.8–0.9 — a deployment described, and still nothing applied"
description: >
  A physical realization binds one exact specification digest to the artifacts that implement it,
  and seven typed formats lower a semantic system to independently released Helm deployments. ESS
  projects the inputs and verifies the evidence; it builds nothing, publishes nothing, applies
  nothing.
slug: a-deployment-described
tags: [release, ess, infrastructure]
date: 2026-09-21T09:00:00+02:00
release_tag: "0.9.0"
release_commit: 8e007b00b08cbe4d8e55ce4172d41b48f6e0ca29
---

The releases through 0.7.1 established that a specification can judge an implementation, and that
the same pattern points at a running cluster. What neither could say is **which artifact implements
which component**, and what it takes to run it. These two releases say it, as typed data, without
acquiring the ability to do anything about it.

{/* truncate */}

## A realization is authored, not inferred

`ess-realization/1` is an adopter-authored document that binds **one exact ESS system, version and
semantic digest** to resolved components and actors, immutable implementation artifacts, typed
runtime requirements, and local, loopback or network entrypoints.

The binding is to a digest, which is the whole point: a realization that named a system by name
would still look valid after the specification moved underneath it. `ess realization compile` emits
deterministic `ess-realization-ir/1` and rejects a stale lock, an unresolved or out-of-subset
reference, incomplete implementation coverage, a malformed placeholder, and **an inline secret
argument** — the last because a document that can carry a credential is a document somebody will
put one in.

`EssIr` did not change. A realization is a separate authority about the same system, which is why
it can be wrong about it and be told so.

## Seven formats, one lowering seam

0.9.0 adds `ess-build/1`, `ess-runtime/1`, `ess-release/1`, `ess-stack/1`, `ess-stack-lock/1`,
`ess-environment/1` and `ess-deployment/1`. Together they are the deterministic path from a
semantic system to independently released Helm deployments.

The CLI compiles build and runtime IR, verifies releases, resolves stacks from **an explicit offline
catalogue**, projects BuildKit and Helm files, compiles environment deployments, and reports the
exact component releases that changed between two deployment documents.

Every one of those verbs stops at a file. ESS refuses unresolved stage-owned obligations rather than
filling them, and it still never builds, publishes or applies anything. The executor boundary — the
place where a command may reach BuildKit, an OCI registry or Helm — does not exist yet; it arrives
in 0.13.0, and when it does it is explicit, named, and separate from every projection.

## Why this order

The temptation with deployment modelling is to start from the thing that applies and work backwards
to a description. That produces a description shaped by one applier. Starting from typed data and
refusing to apply anything means the first consumer of these formats is a diff, and a diff is the
artefact that survives a change of applier.
