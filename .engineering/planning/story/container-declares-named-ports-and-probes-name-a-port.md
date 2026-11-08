---
format: aep.planning-md/3
id: story:container-declares-named-ports-and-probes-name-a-port
kind: story
status: draft
title: A container declares named ports beside its HTTP port, and a probe or a provided endpoint names the port it uses
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
revision: 1
---

## Outcome

A runtime's container role can declare named ports beside `http_port`, an HTTP readiness or
liveness probe can name the port its path is served on, and a provided endpoint can name the port
its Service exposes. The generated chart renders every declared port, probes against the named
port and Services on it, so a process that serves traffic on one port and health on another needs
no hand-written chart.

## Evidence

An adopter on 0.90.0 deploying a WebSocket push service behind an edge proxy hand-wrote the Helm
chart, because the proxy serves traffic on one port and readiness and liveness (`/readyz`,
`/healthz`) on an admin port (9090). On current main:

- `ContainerRole` has one port, `http_port: Option<u16>`
  (`crates/generate/ess-deployment/src/runtime.rs:154-156`), rendered as the only container port,
  always named `http` (`crates/generate/ess-deployment/src/environment.rs:1085-1091`).
- `readiness_path` and `liveness_path` (`runtime.rs:157-162`) render `httpGet` against `port: http`
  only (`environment.rs:1092-1093`, `1102-1103`), and are refused without an `http_port`
  (`runtime.rs:1161-1170`).
- The TCP probe forms (`readiness_tcp_port`, `liveness_tcp_port`, `runtime.rs:163-168`, rendered
  `environment.rs:1095-1111`) take a port number but no path, so an HTTP health path on a second
  port cannot be said; a container takes a path or a TCP port per probe, not both
  (`crates/generate/ess-deployment/src/workload_kind.rs:676-708`).
- A provided endpoint exposes the container's `http_port` only (`runtime.rs:106-116`, refused
  without one at `runtime.rs:446-453` and `runtime.rs:800-806`), and the Service renders the one
  port named `http` (`environment.rs:1242`).

Minimal reproduction (neutral, `ess-runtime/2`), refused today with an unknown-field error on
`ports`:

```yaml
containers:
  - name: proxy
    process: proxy
    http_port: 8080
    ports:
      - {name: admin, port: 9090}
    readiness_path: /readyz
    readiness_port: admin
    liveness_path: /healthz
    liveness_port: admin
```

## Fit review

1. **The need, apart from the syntax.** A process may listen on more than one TCP port, and its
   health may be served on a port other than its traffic port. The runtime document can state one
   port per container (`runtime.rs:154-156`), so the chart can neither open the second port nor
   probe a path on it. The requester's proposal (theirs): "named ports beyond `http_port`, and
   probes that name a port". Reproduction above.
2. **Class: gap.** A real deployment fact cannot be expressed: no runtime key reaches a second
   container port (`runtime.rs:149-190`), and no workaround renders an HTTP probe on it
   (`environment.rs:1092-1111`).
3. **Can it already be expressed?** No. `readiness_tcp_port: 9090` opens no port and checks only a
   TCP connect, not the path that reports readiness; moving the health path onto `http_port` changes
   the process, not the specification. `docs/design/runtime-workload-kinds.md:35-47` lists every pod
   setting `/2` takes; none is a port list. Probe timing, startup probes and port protocols are not
   in it either.
4. **Fit.** This is deployment intent, not specification language: it reaches no cell of grid R, O
   or G2 (`docs/design/language-grids.md`). Vocabulary: the port name follows Kubernetes' port-name
   rule and the existing `http` name (`environment.rs:1088`); ESS already spells "a port, a number
   or a declared port name" in `ess infra` (`crates/infra/infra-spec/src/raw.rs:186-210`), so a
   probe naming a declared port is the same idea. Siblings: the same question ("which port") arises
   for both probes and for the provided endpoint (`runtime.rs:106-116`); all three take the name,
   or the story records why not. The TCP probes keep their number (a `/2` key), because a TCP probe
   names no path and its number already says the port. Targets: the Helm projection
   (`environment.rs:1056-1265`) renders it; `ess generate deployment reconcile`'s recovery profile
   admits Deployment, StatefulSet and Service only (`crates/edge/ess-cli/src/recovery/model.rs:517-546`)
   and gains no kind here; the stack lock carries a provided endpoint's port
   (`crates/generate/ess-deployment/src/stack.rs:182-190`), which becomes the named port's number.
   Generated code, synthesis, the interpreter and `ess verify diff` do not read runtime documents.
5. **A second adopter.** A service whose framework serves management endpoints (metrics, health) on
   a separate management port, with traffic on the main port; or a gRPC server on one port with an
   HTTP health endpoint on another. Both need a second declared port and a probe naming it.
6. **Cost.** New keys in a released format: `/2` has been read since 0.62.0 (`CHANGELOG.md:1029`),
   so the precedent that late keys may join an unreleased format (`docs/design/runtime-workload-kinds.md:86-91`)
   no longer applies, and the keys need `ess-runtime/3` (and `ess-runtime-ir/3`) beside `/1` and
   `/2`. One format per decision (`docs/design/generic-language-features.md` § Policy, policy 2):
   this story, `story:workload-declares-a-shutdown-drain` and
   `story:workload-declares-a-rollout-and-a-disruption-budget` open one `/3` together, whichever
   lands first; a released `/3` sends the rest to `/4`. New diagnostics: an unknown port name, a
   duplicate port name or number in one pod, a name outside the Kubernetes rule or equal to `http`.
   No migration: `/1` and `/2` documents keep their bytes.
7. **Considered.** (a) Change nothing: the adopter keeps a hand-written chart, and the runtime
   document states a single port that is false for the process. (b) The requester's shape, named
   ports plus probes naming a port: taken. (c) Structured probes in `ess infra`'s remedy shape
   (`readiness: {http_get: {path, port}}`, `raw.rs:161-201`): refused, because the runtime document
   would then have two spellings of one probe (`readiness_path` and `readiness:`), the red flag the
   review names. Timing fields that shape would bring are a separate need nobody reported.

## Decisions

- **Accept, redesigned.** The need is accepted as reported; the design names the keys after the
  runtime document's existing flat probe keys instead of adding a probe object.
- `ContainerRole.ports: [{name, port}]` declares additional TCP ports; `http_port` stays and keeps
  the reserved name `http`.
- `readiness_port` / `liveness_port` name the port a `readiness_path` / `liveness_path` is served on;
  absent, the port is `http` as today. Naming `http` explicitly is allowed and renders identically.
- `ProvidedEndpoint.port` names the port the Service exposes; absent, `http` as today. A provided
  endpoint no longer needs `http_port` when it names a declared port.
- Format consequence: `ess-runtime/3` compiling to `ess-runtime-ir/3`, beside `/1` and `/2`; a `/1`
  or `/2` document carrying any of these keys is refused naming the key and `ess-runtime/3`.

## Acceptance

- A `/3` container with `ports: [{name: admin, port: 9090}]`, `readiness_path: /readyz`,
  `readiness_port: admin` compiles; the chart renders both container ports (`http` first, then the
  declared ports in document order) and `readinessProbe.httpGet` with `port: admin`.
- A `/3` provided endpoint with `port: admin` renders a Service port named `admin` with
  `targetPort: admin` and the declared number; the stack lock's provided-endpoint port is that
  number.
- Refused by name: a probe or endpoint port naming no declared port; a declared port named `http`;
  two ports with one name or one number in the same pod (across its containers); a port name
  outside the Kubernetes port-name rule; port 0; `readiness_port` without `readiness_path` (and the
  same for liveness).
- An unset setting changes no chart bytes: a `/3` runtime without `ports`, `readiness_port`,
  `liveness_port` or endpoint `port` renders a chart byte-identical to the same runtime as `/2`.
- A `/1` and a `/2` document carrying any of the new keys is refused naming the key and
  `ess-runtime/3`; every existing `/1` and `/2` fixture keeps its IR digest and chart bytes (old-reader
  compatibility test beside `an_ess_runtime_1_document_keeps_its_bytes_and_digest`).
- `docs/design/runtime-workload-kinds.md` records the `/3` decision and the keys before the code;
  `website/docs/guides/deliver/build-and-chart.md` and `website/docs/reference/formats.md` name
  `ess-runtime/3`.

## Scope (inferred)

- `crates/generate/ess-deployment/src/runtime.rs` (`ContainerRole`, `ProvidedEndpoint`, format
  constants, endpoint and probe validation).
- `crates/generate/ess-deployment/src/workload_kind.rs` (version-gated key lists, port-name rule).
- `crates/generate/ess-deployment/src/environment.rs` (`render_container`, `render_services`).
- `crates/generate/ess-deployment/src/stack.rs` (provided-endpoint port derivation).
- `crates/generate/ess-deployment/tests/workload_kinds/`.
- `docs/design/runtime-workload-kinds.md`, `website/docs/guides/deliver/build-and-chart.md`,
  `website/docs/reference/formats.md`, `CHANGELOG.md`.
- Shares `ess-runtime/3` with the shutdown and rollout stories: not parallel-safe with them on
  `runtime.rs` and `workload_kind.rs`.
