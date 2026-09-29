---
title: Deploy an environment
sidebar_position: 3
description: Bind a stack lock to an environment, compare two deployments, preview a reconcile, and reconcile under a recovery authority.
---

# Deploy an environment

An environment document (`ess-environment/1`) binds an exact stack lock to one cluster and
namespace, and supplies what the charts leave open: release names, service accounts, secret
references and external endpoints. `ess generate deployment compile` turns the two into an
`ess-deployment/1` document. `deployment diff` compares two of them, and `deployment reconcile`
applies the difference with Helm.

The commands on this page continue [Resolve a stack](./resolve-a-stack.md). Every command except
reconcile's execution was run for this page; see
[what could not be run](#what-was-not-run-for-this-page).

## Compile a deployment

```yaml
format: ess-environment/1
environment: staging
stack_digest: sha256:657848e4e799c369d9e870cbf7eb1b78d3a1c9b0fd134393cc385a47f1acc207
cluster: staging-cluster
namespace: oracle
releases:
  - service: oracle
    release_name: oracle
    service_account: oracle
    secrets:
      database-password:
        name: oracle-database
        key: password
```

`stack_digest` is the SHA-256 of the lock file (`sha256sum stack-lock.json`), not the
`stack_digest` field inside the lock. A mismatch is refused:

```shell-session
$ ess generate deployment compile --path environment.yaml --stack-lock stack-lock.json --out deployment.json
lowering was refused:
[digest_mismatch:Deployment] environment stack_digest does not match the supplied exact stack lock
```

With the right digest:

```shell-session
$ ess generate deployment compile --path environment.yaml --stack-lock stack-lock.json --out deployment.json
staging — 1 independent release(s), compiled to deployment.json
```

Secret entries name a Kubernetes Secret and key; the secret value never appears in any ESS
document. An endpoint a runtime requires from another service in the lock, or from a typed external
system, is derived by the compiler, and the environment can still override it explicitly. The
deployment also records the rollout order from the stack's `depends_on:`.

## Compare two deployments

```shell-session
$ ess generate deployment diff --from deployment.json --to deployment-2.json
added: []
changed: ["oracle"]
removed: []
$ ess generate deployment diff --from deployment.json --to deployment-2.json --format json
{
  "added": [],
  "changed": [
    "oracle"
  ],
  "format": "ess-deployment-diff/1",
  "from": "sha256:8a6b3bee…",
  "removed": [],
  "to": "sha256:b7cbbbeb…"
}
```

`deployment-2.json` was compiled from the same lock with a different service account. The diff
lists services, not fields, and exits 0 whether or not anything differs. It compares two documents;
it does not look at a cluster.

## Preview a reconcile

```shell-session
$ ess generate deployment reconcile --path deployment-2.json --current deployment.json \
    --cache cache --dry-run
apply: oracle
remove: 
unverified local comparison preview: this states no live, applied or absent fact about any target
```

`--path` is the desired deployment. `--current` is the *admitted baseline*: the deployment you
last asked for, which is not proof that it was ever applied. Omit it for a first deployment.
`--dry-run` compares the two documents and stops. It reads no cluster, runs no Helm or ORAS, writes
nothing to the cache and records nothing.

A release that is in the baseline but not in the desired deployment is removed only when you say
so, and the check runs before anything else, even for a preview:

```shell-session
$ ess generate deployment reconcile --path deployment.json --current two-releases.json \
    --cache cache --dry-run
error: the deployment removes oracle-b; rerun with --allow-removals after reviewing the retirement set
$ ess generate deployment reconcile --path deployment.json --current two-releases.json \
    --cache cache --dry-run --allow-removals
apply: 
remove: oracle-b
unverified local comparison preview: this states no live, applied or absent fact about any target
```

A desired deployment on a different cluster than its baseline is refused; retire the old deployment
explicitly instead.

## Reconcile under a recovery authority

Without `--dry-run`, `reconcile` requires `--authority`, and refuses before it reads anything else:

```shell-session
$ ess generate deployment reconcile --path deployment-2.json --current deployment.json --cache cache
error: normal execution requires a caller-provisioned recovery authority: pass --authority <UUID> naming an entry of the protected registry, or --dry-run for a local unverified preview
```

`--authority <UUID>` names one entry of the recovery registry, a set of files an administrator
provisions under `/etc/ess/recovery/` (the registry, a host identity, and each authority's
retained revisions). ESS never creates or approves an authority. An authority pins the cluster's
API endpoint and CA, the namespaces and the service account by UID, the exact desired and baseline
deployment digests, the Helm binary, and a local state store. The whole registry is read and
checked before one entry is selected; an unreadable or invalid entry anywhere stops the
invocation. On a host with no registry:

```shell-session
$ ess generate deployment reconcile --path deployment-2.json --current deployment.json \
    --cache cache --authority 6f1c2d3e-4b5a-4c6d-8e7f-9a0b1c2d3e4f
selected: oracle
settled: (none)
incomplete execution evidence — AuthorityMismatch: /etc/ess is unreadable: No such file or directory (os error 2)
error: AuthorityMismatch: /etc/ess is unreadable: No such file or directory (os error 2)
```

Under an admitted authority, one invocation:

1. reserves an invocation record in the authority's state store and takes the target's lock;
2. reads the current state of each affected release from the cluster;
3. for each release in rollout order, fetches its chart by digest into `--cache` and checks it,
   attempts at most one Helm operation, and records what it can establish before the next;
4. handles removals afterwards, in reverse rollout order of the baseline;
5. stops at the first thing it cannot establish.

A stop leaves that release's state **unknown**: not absent, not rolled back and not reconciled.
Releases completed earlier stay applied; ESS issues no compensating calls. A later invocation reads
the cluster and decides again; it does not replay a remembered list of commands.

| Option | Meaning |
|---|---|
| `--cache <DIR>` | where verified, digest-pinned chart artifacts are cached; required |
| `--timeout <DURATION>` | the wait timeout passed to Helm; default `5m` |
| `--retry-of <EPOCH>:<NONCE>` | names an earlier invocation this one follows, as `<store epoch>:<nonce>`, both UUIDs. It adds context; it never hides other retained history |
| `--allow-removals` | allows uninstalling releases absent from the desired deployment |

The command exits 0 only when every selected release was settled and recorded. The report lists
the selected releases, those settled, and the reason it stopped.

## What was not run for this page

Execution under an authority needs a root-provisioned registry, a Kubernetes cluster, the pinned
Helm binary and the OCI registry holding the charts. None was available, so the numbered steps,
`--timeout` and a successful `--retry-of` come from the command's help and source, not from a run.
[Limitations](../../status/limitations.md) lists what a completed reconcile does and does not
establish.
