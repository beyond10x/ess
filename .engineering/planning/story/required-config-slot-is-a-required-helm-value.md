---
format: aep.planning-md/3
id: story:required-config-slot-is-a-required-helm-value
kind: story
status: draft
title: A projected chart refuses to render without its required settings and image
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

A Helm chart projected from a runtime IR refuses to render without its required settings: a
`kind: required` configuration slot and each image's repository and digest render through Helm's
`required`, so `helm template` without them fails naming the missing value, while `helm lint` on a
fresh chart still passes.

## Evidence

Reported by an adopter on ess 0.59.0 and reproduced on 0.60.0: a runtime with a required slot
(`broker-url`) projects to `config: {}` and `images.app: {repository: "", digest: ""}`;
`helm template` exits 0 and renders `value:` empty and `image: "@"`. In
`crates/generate/ess-deployment/src/environment.rs`, `Required` and `Optional` render the same
line (~:847-851), the image is `repository@digest` unchecked (~:794), and the values schema names
no required key (~:705). Only ESS's own deployment compile refuses an unbound slot (~:395). A probe
wrapping the slot in `required` keeps `helm lint` green (as `website/docs/concepts/component-delivery.md`
promises) and makes `helm template` fail with `config.broker-url is required`.

## Acceptance

- `required_config_slot_unset_fails_helm_template`
- `empty_image_digest_fails_helm_template`
- `fresh_chart_still_passes_helm_lint`

Optional slots and set values render byte-identically to today; the chart byte pins move only for
required slots and images.
