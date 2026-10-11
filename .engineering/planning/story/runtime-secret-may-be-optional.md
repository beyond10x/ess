---
format: aep.planning-md/3
id: story:runtime-secret-may-be-optional
kind: story
status: draft
title: A runtime container secret may be optional, and the chart omits it when no Secret is named
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A container secret in `ess-runtime/1` can be declared `optional: true`. The chart then renders
`secretKeyRef.optional: true`, and an environment that names no Secret for it renders no
environment variable for it, instead of a `secretKeyRef` with an empty `name:`.

## Evidence

An adopter on ess 0.55.0 got `containers[0].secrets[0]: unknown field `optional`, expected one of
`name`, `environment`, `key``. At tag 0.59.0 `SecretSlot` still has exactly those three fields and
rejects unknown ones (`crates/generate/ess-deployment/src/runtime.rs:42-52`), and the chart renders
every secret slot as `secretKeyRef` with `name: {{ index .Values.secrets "<slot>" "name" }}` and no
condition (`crates/generate/ess-deployment/src/environment.rs:858-860`), so an environment that
sets no name renders an empty `name:`. Read from the 0.59.0 source, not from a compile run.

## Acceptance

- `optional: true` on a container secret validates; omitted, it means `false` and every existing
  runtime document, compiled runtime and chart keeps its bytes.
- A chart for an optional secret renders `secretKeyRef.optional: true`, and the whole env entry is
  inside a condition on the Secret name, so an environment that sets none renders no entry for it.
- A required secret with no name set in the environment is refused at `deployment reconcile` (or
  keeps today's behaviour, decided and stated), never rendered as an empty `name:`.
- The format consequence for `ess-runtime/1` (a reader that rejects unknown fields meets the new
  key) is decided and recorded, with an old-reader compatibility test.
