---
title: "0.31–0.33 — Entity Runtime lowering, sidecars a binding must name, and a Secret that is only present"
description: >
  0.31.0 lowers a service contract to Entity Runtime definitions. 0.32.0 imports OpenAPI 3.0 and
  makes an unbound sidecar in a bound workload a violation. 0.33.0 stops a Kubernetes scan from
  writing anything derived from a Secret value, and lets a binding acknowledge containers the
  service does not build.
slug: a-secret-is-only-present
tags: [release, ess]
date: 2026-09-26T12:00:00+02:00
release_tag: "0.33.0"
release_commit: 3ec2deb977bb19c292a611dcf92a3e676352864a
---

Three releases. One carries a specification into a runtime, and two tighten what an observed
cluster is allowed to say — about the containers a workload runs, and about the Secrets it reads.

{/* truncate */}

## A service contract becomes Entity Runtime definitions

0.31.0 adds `ess-entity-runtime`, which projects an admitted service contract into validated
Entity Runtime definitions and the typed host binding obligations they leave. Conditional outcome
selection, exact values and event order are preserved. What cannot be lowered faithfully is
refused by name — a cleared field, a silent preserve with no response — rather than approximated.
0.33.0 moves the target to Entity Runtime `0.24.1`.

## A bound workload runs nothing it does not declare

0.32.0 imports OpenAPI 3.0 by rewriting each schema to its 3.1 form, and reports a `null` it cannot
carry as a gap at its own pointer. `ess verify bindings` gains `OBS-BIND-008`: a container or native
sidecar in a bound workload that no binding names is a violation, so binding a workload now claims
it runs nothing else.

0.33.0 adds the honest exception. `ess-observed-bindings/2` lets a binding acknowledge a mesh proxy
or vendor agent by name and reason, and the report lists it as acknowledged, not bound.

## A Secret is only present

A full Kubernetes scan used to write each Secret value's unsalted SHA-256 and byte length, which
confirm a guessed low-entropy secret to anyone holding the file. 0.33.0 writes
`infra-observation/3` and `infra-ir/3`, where each Secret key is `{"present": true}` and nothing
more. Drift, in `infra-drift/3`, therefore reports added and removed keys and never a rotated value.
The new versions are described in [the version history](/ess/docs/reference/spec-versions).
