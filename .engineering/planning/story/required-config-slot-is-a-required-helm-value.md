---
format: aep.planning-md/3
id: story:required-config-slot-is-a-required-helm-value
kind: story
status: active
title: A projected chart refuses to render without its required settings and image
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-deployment/src/environment.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/deployment.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/workload_kinds/mod.rs
- confidence: inferred
  path: website/docs/concepts/component-delivery.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:48Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:50Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A Helm chart projected from a runtime IR refuses to render without its required settings: a `kind: required`
configuration slot and each image's repository and digest render through Helm's `required`, so `helm template`
without them fails naming the missing value, while `helm lint` on a fresh chart still passes.

## Evidence

Reported by an adopter on ess 0.59.0, reproduced on 0.60.0 and again on 0.92.0. In
`crates/generate/ess-deployment/src/environment.rs`, `ConfigKind::Required` and `ConfigKind::Optional` render the same
line (~:1185-1190), a build-output image renders as `repository@digest` unchecked (~:1063) with
`repository: ""` / `digest: ""` defaults in `values.yaml` (~:675), and the values schema names no required key under
`config` or `images` (~:698-712). Only ESS's own deployment compile refuses an unbound required slot (~:398).

Reproduction on 0.92.0: an `ess-runtime-ir/1` with one process whose image is the build output `app` and one container
with config slots `broker-url` (`kind: required`) and `log-level` (`kind: optional`):

```text
$ ess generate project helm --ir runtime-ir.json --chart demo --version 1.0.0 --out chart
5 Helm chart file(s) projected to chart without applying them
$ helm lint chart
1 chart(s) linted, 0 chart(s) failed
$ helm template r chart
          image: "@"
            - name: BROKER_URL
              value:
            - name: LOG_LEVEL
              value:
exit=0
```

`values.schema.json`: `config` is `{"type": "object", "additionalProperties": {"type": "string"}}`, `images` is
`{"type": "object"}`. A probe wrapping the slot in `required` keeps `helm lint` green (as
`website/docs/concepts/component-delivery.md` promises) and makes `helm template` fail with
`config.broker-url is required`.

## Acceptance

All tests are in `cargo test -p ess-deployment --locked --test deployment` (projection bytes, and a `helm` run where
the binary is on `PATH`, each `helm` test skipped by name where it is not):

- `required_config_slot_unset_fails_helm_template`: `helm template` of the chart above with no values exits non-zero
  and its output names `broker-url`.
- `required_values_set_render_the_chart`: `helm template` with `--set config.broker-url=x`,
  `images.app.repository` and `images.app.digest` set exits 0.
- `empty_image_digest_fails_helm_template`: with `broker-url` and the repository set and `images.app.digest` left
  empty, `helm template` exits non-zero and its output names the image's digest.
- `empty_image_repository_fails_helm_template`: with `broker-url` and the digest set and `images.app.repository`
  left empty, `helm template` exits non-zero and its output names the image's repository.
- `fresh_chart_still_passes_helm_lint`: `helm lint` on the fresh projection exits 0.
- Optional slots, literal slots, `Reference` images and set values render byte-identically to today: the chart byte
  pins of a chart with no required slot and no build-output image keep their values, and the change's diff moves
  only pins of charts that hold one (`RUNTIME_1_CHART_DIGEST` in `tests/workload_kinds/mod.rs` is one: its process
  image `app` is a build output).

## Scope

- cited: crates/generate/ess-deployment/src/environment.rs
- cited: crates/generate/ess-deployment/tests/workload_kinds/mod.rs
- cited: crates/generate/ess-deployment/tests/deployment.rs
- inferred: website/docs/concepts/component-delivery.md
confidence: high

Shared surfaces: proposes its `CHANGELOG.md` sentence, naming each chart pin it moves; moves no row of the
repository-wide byte-pin targets.

## Decisions

- Each moved chart pin is named in the proposed `CHANGELOG.md` sentence rather than in an acceptance bullet, because
  the changelog is written by the coordinator at merge (epic § Shared surfaces).
