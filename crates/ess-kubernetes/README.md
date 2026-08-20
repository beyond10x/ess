# ess-kubernetes

The **actor** beside [`aep`](../aep): scans a Kubernetes
cluster into an `infra-observation/1` bundle the pure toolchain consumes. This repo holds the
kubeconfig and shells out to `kubectl`; the analysis side never holds a credential — that
boundary is `aep/docs/VISION.md` § "What this is deliberately not", and this
repository exists so that boundary can stay.

```console
ess-kubernetes contexts
ess-kubernetes scan --context k3d-example --out cluster.observation.json
```

Secret values never touch disk: every `data`/`stringData` value is replaced with its SHA-256 and
length before the bundle is written, and the `last-applied-configuration` annotation — which can
carry values — is dropped. The shape (which secrets exist, which keys) survives, which is what
dependency mapping needs.

v0 deliberately shells out to `kubectl` instead of linking a Kubernetes client: credentials, exec
plugins and API skew stay kubectl's problem. Applying anything is not in this tool yet, and when
it arrives it will be behind an explicit flag with a server-side dry-run first.
