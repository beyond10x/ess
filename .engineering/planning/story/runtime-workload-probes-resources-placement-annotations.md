---
format: aep.planning-md/3
id: story:runtime-workload-probes-resources-placement-annotations
kind: story
status: active
title: A workload states its pod settings (probes, resources, security, secret volumes, image digest, placement, annotations), and the chart renders them
tags:
- adopter-report
- deployment-chain
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T03:06:49Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-15T03:06:50Z", actor: "human:timo", revision: 5}
---
## Outcome

A runtime's workloads and containers can state TCP readiness and liveness probes, resource
requests and limits, affinity or anti-affinity, and annotations, and the generated chart renders
them.

## Evidence

An adopter on ess 0.55.0 found none of these in the runtime or chart. On main:

- probes: `ContainerRole` has only `readiness_path`/`liveness_path`
  (`crates/generate/ess-deployment/src/runtime.rs:142-146`), rendered as `httpGet` only
  (`environment.rs:816,819`); a TCP probe exists only as an `ess infra` remedy
  (`crates/infra/infra-spec/src/raw.rs:170,199`).
- resources: none on containers (`runtime.rs:130-162`); the only chart `resources:` is the PVC
  request (`environment.rs:885,893`).
- affinity: none on `Workload` (`runtime.rs:174-191`) or in `render_workloads`.
- annotations: metadata is name and labels only (`environment.rs:751`).

## Acceptance

- Each setting validates in the runtime document, is rendered into the chart, and is checked by
  a chart projection test; an unset setting changes no chart bytes.
- Resources and placement can be overridden per environment, or the story records why not.
- The runtime format consequence is decided and recorded.

### Pod settings every workload kind takes (Deployment, StatefulSet and the scheduled job)

Tests in an existing test binary of the crate, each naming its setting:

- `a_container_declares_requests_and_limits`: cpu and memory requests and limits render the
  container's `resources`.
- `a_workload_declares_its_pod_security`: `run_as_user`, `run_as_group`, `fs_group` and
  `run_as_non_root` render the pod `securityContext`; `read_only_root_filesystem` per container
  renders the container `securityContext`.
- `a_workload_opts_out_of_the_service_account_token`: `service_account_token: false` renders
  `automountServiceAccountToken: false`.
- `a_secret_slot_is_mounted_as_a_volume`: a secret slot mounted at a path, read-only, with items
  mapping a key to a relative path and a `mode` (octal, 0400) renders a `secret` volume with
  `items` and `defaultMode` and a read-only `volumeMount`; a path that escapes the mount, a
  duplicate item path or a mode outside 0000-0777 is refused by name.
- `an_image_is_pinned_by_digest`: an image reference by `@sha256:<64 hex>` renders verbatim; a
  malformed digest is refused by name.
- `a_runtime_without_pod_settings_keeps_its_chart_bytes`.

## Delivery notes

- `readiness_tcp_port`, `liveness_tcp_port`, `annotations` and `affinity` join `ess-runtime/2`
  (no release has read `/2` yet); `/1` refuses each by name and its digests are unchanged
  (`docs/design/runtime-workload-kinds.md`).
- Annotations render on the pod template only. `affinity` carries `node_labels`,
  `spread_across` and `spread` (preferred by default, or required).
- Not in this story: per-environment overrides of resources and placement; `/1` keeps accepting
  `{{`, names outside the Kubernetes rules, `replicas` past int32 and `http_port: 0`; Secret keys
  used as environment variables are not checked in either format. Each needs its own decision.
