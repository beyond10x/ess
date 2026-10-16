# Runtime workload kinds and pod settings

Two stories meet in one format change: a workload that runs on a schedule
(`story:runtime-realizes-a-scheduled-job`), and the pod settings every workload kind takes
(`story:runtime-workload-probes-resources-placement-annotations`). Both add keys to the authored
runtime document (`RuntimeSpec`, `crates/generate/ess-deployment/src/runtime.rs`) and to its
compiled form (`RuntimeIr`), and both are rendered by `render_workloads`
(`crates/generate/ess-deployment/src/environment.rs`). The format consequence is decided once, here,
for both.

## The format consequence: `ess-runtime/2` beside `/1`

`ess-runtime/1` and `ess-runtime-ir/1` reject unknown fields. A new key is a new statement in a
persisted document: a workload with a `schedule:` is not a Deployment, and a process whose `image`
is a pinned reference names no build output. The repository's format rule asks for a new version
when meaning changes, so:

- `ess-runtime/2` is the authored format that admits the new keys, and compiles to
  `ess-runtime-ir/2`. Both are accepted beside `/1`.
- `ess-runtime/1` compiles to `ess-runtime-ir/1` exactly as before. A `/1` document keeps its
  compiled bytes and its digest, so every release, stack lock and deployment that pins a `/1`
  runtime digest is unchanged.
- A `/1` document (authored or compiled) that carries a `/2` key is refused, naming the key and
  `ess-runtime/2`. A `/1` workload must still declare `replicas`.
- A released `ess` that reads only `/1` refuses a `/2` IR: its reader denies the unknown keys and its
  format check names `ess-runtime-ir/1`. It never reads a scheduled workload as a Deployment.

`RUNTIME_FORMAT` and `RUNTIME_IR_FORMAT` keep naming `/1`; `RUNTIME_FORMAT_V2` and
`RUNTIME_IR_FORMAT_V2` name `/2`. The old-reader test
(`tests/workload_kinds/mod.rs`, `an_ess_runtime_1_document_keeps_its_bytes_and_digest`) pins the
`/1` fixture's IR digest and chart bytes as they were before the change.

## Where the keys live

| Key | On | Renders |
|---|---|---|
| `schedule:` | `Workload` | `batch/v1` `CronJob` and its job template, instead of `Deployment`/`StatefulSet` |
| `replicas` | `Workload` | now optional: required on every workload that is not scheduled, refused beside `schedule:` |
| `security: {run_as_user, run_as_group, fs_group, run_as_non_root}` | `Workload` | pod `securityContext` |
| `service_account_token: false` | `Workload` | `automountServiceAccountToken: false` |
| `resources: {requests, limits}` with `cpu` and `memory` | `ContainerRole` | container `resources` |
| `read_only_root_filesystem` | `ContainerRole` | container `securityContext.readOnlyRootFilesystem` |
| `mount: {path, items, mode}` | `SecretSlot` | a read-only `secret` volume with `items` and `defaultMode`, and its `volumeMount` |
| `image: <repository>@sha256:<64 hex>` | `Process` | the reference verbatim, instead of the values-filled build output |
| `readiness_tcp_port`, `liveness_tcp_port` | `ContainerRole` | a `tcpSocket` readiness or liveness probe; a container takes a path or a TCP port for each probe, not both |
| `annotations: {<qualified name>: <text>}` | `Workload` | the pod template's `metadata.annotations` |
| `affinity: {spread_across, spread, node_labels}` | `Workload` | the pod `affinity`: `node_labels` as required node affinity (`In` one value per label), `spread_across` (a node label key) as pod anti-affinity against the workload's own pods, preferred with weight 100 unless `spread: required` |

Pod settings sit on the workload because a pod has one security context and one token mount;
resources and the root filesystem sit on the container role because Kubernetes sets them per
container. All of them are taken by every workload kind: Deployment, StatefulSet and the scheduled
job render the pod template through one function.

A secret slot with `mount:` may omit `environment` and `key`: the mount's `items` name the keys,
and a slot that is only a file exports no variable. `environment` and `key` go together, and a slot
declares at least one of them or a mount. The values file and its schema still take the Secret's
`name` for a mounted slot; `key` is optional there. The mode is a string of four octal digits
(`"0400"`), because YAML readers disagree on whether `0400` is octal; it renders as the decimal
`defaultMode` Kubernetes reads.

`Process.image` is one string: an identifier names a build output, as in `/1`; anything else is a
reference that must be pinned by digest, and is refused by name otherwise. A pinned reference is
not a build output, so it takes no `images:` entry in the values file.

### What the API server and Helm accept

A `/2` document is refused, by name, wherever the chart it renders would not install or would not
say what the document says:

- an integer past the Kubernetes field's type: `retry_limit`, `successful_runs`, `failed_runs` and
  `replicas` (`int32`), and `run_as_user`, `run_as_group` and `fs_group` above 2147483647;
- a day of week of 7: Kubernetes reads a schedule with `robfig/cron` v3, whose day of week is 0-6;
- a pod volume, container or workload name that is not a DNS-1123 label, including the
  `secret-<slot>` volume of a mounted secret slot;
- a secret item key outside `[-._a-zA-Z0-9]`, or one that starts with `..`, and an item path that
  starts with `..` (the kubelet's atomic writer reserves `..data`);
- a cpu or memory request above its limit, compared exactly over the quantity's suffix;
- a TCP probe port of 0, or a path and a TCP port for the same probe;
- an annotation key that is not a qualified name, annotations above 256 KiB, an affinity label key
  or value Kubernetes refuses, an empty `annotations:` or `affinity:`;
- `{{` in any string the chart renders as written (an image reference, an entrypoint or argument,
  a probe path, a config literal or environment name, a volume or secret mount path, a secret item
  key or path, an annotation value). Every chart file is a Helm template, so `{{` would be evaluated
  at install time, and an action such as `lookup` could read a cluster object into the manifest.

The rules that bear on keys `/1` already reads (`replicas`, container and volume names, probe paths,
config literals, mount paths) apply to `/2` documents only: a `/1` document compiles exactly as it
did. The keys added after the first `/2` change (`readiness_tcp_port`, `liveness_tcp_port`,
`annotations`, `affinity`) join `/2` rather than a `/3`, because no `ess` release has read `/2` yet;
`/1` refuses each of them naming `ess-runtime/2`, and every compiled `/1` and `/2` IR without them
keeps its bytes.

## The scheduled workload

```yaml
workloads:
  - name: ingest-a
    components: [ingest]
    containers: [runner]
    schedule:
      cron: "2-59/10 * * * *"
      time_zone: Europe/Berlin       # optional; an IANA name, canonical spelling
      overlap: forbid                # forbid (default) | replace | allow
      successful_runs: 3             # optional history limits
      failed_runs: 1
      suspended: true                # optional; absent renders suspend: false
      start_deadline_seconds: 120    # optional, positive
      run_deadline_seconds: 900      # required, positive
      retry_limit: 2                 # optional, not negative
      restart: never                 # never (default) | on_failure
    volumes:                         # optional: state that survives between runs
      - name: state
        size: 1Gi
```

It renders one `CronJob` whose job template carries the pod exactly as a Deployment's template
would (containers, environment, config, secret references, mounts and pod settings), with
`restartPolicy: Never` unless `restart: on_failure`. Named `volumes:` render one
`PersistentVolumeClaim` each, named `<release>-<workload>-<volume>`, which every run mounts; the
values file sizes and classes them as it does a StatefulSet's claims. No StatefulSet and no
headless Service is rendered for a scheduled workload.

Refused, each naming the workload and what it lacks or conflicts with:

- no `run_deadline_seconds`: a run that may never end is unbounded;
- a cron expression that is not five fields of minute, hour, day of month, month and day of week
  (or one of `@yearly`, `@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`), or
  one that carries `TZ=`/`CRON_TZ=` instead of `time_zone`;
- a `time_zone` that is not a canonical IANA name in the bundled database (`jiff-tzdb`, compiled
  into the binary so the answer does not depend on the machine);
- `replicas`, the legacy single `storage`, a provided endpoint (a Service port) selecting the
  workload, or a container with a readiness probe;
- a deadline of zero or less, or a negative retry limit.

### How it realizes a component

A scheduled workload realizes its components as any workload does: each component exactly once,
and a component whose topology is not stateless needs `volumes:`. The topology's replica range is
not applied: a scheduled job has no standing instances to count, and with `overlap: forbid` it runs
at most one at a time. This is the one place a runtime satisfies a component without replicas, and
it is visible in the document by the `schedule:` key.

## What the deployment-chain gaps share

Resources, probes, placement and annotations are all pod-template statements. They share the
renderer the scheduled job uses, so each lands once for every workload kind. All of them are
part of this change: resources, security, the token opt-out, secret volumes, digest-pinned images,
TCP probes, affinity and annotations. Annotations are pod annotations (what admission
controllers, sidecar injectors and scrapers read), not controller annotations.

### Per-environment overrides

Resources and placement are fixed in the runtime document and not overridden per
environment. The runtime IR is pinned by digest into every release, and an environment binds only
configuration, secrets, endpoints and the service account (`ess-environment/1`); a per-environment
resource value would need a new environment key and a values path the projection reads. That is a
separate change to `ess-environment`, not to the runtime.

## What is not decided here

Running or triggering a job, a one-shot `Job` workload, and admission of a `CronJob` or
`PersistentVolumeClaim` by `ess generate deployment reconcile`'s recovery profile, which admits the
object kinds a `/1` chart renders.
