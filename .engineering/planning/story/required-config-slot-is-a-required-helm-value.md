---
format: aep.planning-md/3
id: story:required-config-slot-is-a-required-helm-value
kind: story
status: draft
title: A projected chart refuses to render without its required settings and image
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
- decomposes: epic:zero-open-issues-2026-11
revision: 3
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

Tests in `crates/generate/ess-deployment/tests/deployment.rs` (projection bytes) plus a `helm` run where the binary is
on `PATH`, skipped by name where it is not:

- `required_config_slot_unset_fails_helm_template`: `helm template` of the chart above with no values exits non-zero
  and names `broker-url`; with `--set config.broker-url=x` and both image values set it exits 0.
- `empty_image_digest_fails_helm_template`: with `broker-url` set and `images.app.digest` left empty, `helm template`
  exits non-zero naming the image's digest (likewise the repository).
- `fresh_chart_still_passes_helm_lint`: `helm lint` on the fresh projection exits 0.
- Optional slots, literal slots, `Reference` images and set values render byte-identically to today; the chart byte
  pins (for example `RUNTIME_1_CHART_DIGEST` in `tests/workload_kinds/mod.rs`) move only where a chart has a required
  slot or a build-output image, and each moved pin is named in the CHANGELOG.

## Scope

- cited: crates/generate/ess-deployment/src/environment.rs
- cited: crates/generate/ess-deployment/tests/workload_kinds/mod.rs
- inferred: crates/generate/ess-deployment/tests/deployment.rs
- inferred: website/docs/concepts/component-delivery.md

confidence: high
