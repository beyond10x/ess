# ESS Kubernetes adapter

Imports Kubernetes into sanitized infrastructure IR while keeping credentials at one explicit
edge. It accepts either an existing `infra-observation/1` bundle or a caller-selected live context:

```console
ess import kubernetes --path observation.json --out infrastructure.json
ess import kubernetes --context example --observation-out observation.json --out infrastructure.json
```

Live scanning shells out to `kubectl`. Before any observation is written, Secret `data` and
`stringData` values become digest-and-length records and the last-applied annotation is removed.
The adapter never applies infrastructure. Analysis, graphing, diagnosis, drift, intent, and
projection operate only on the sanitized typed values elsewhere in ESS.

Live-cluster tests require explicit caller credentials and are intentionally outside `task check`.
