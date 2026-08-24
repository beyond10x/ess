# AGENTS.md — ess-kubernetes

The contract for changing **this** repository. Org-wide rules — the naming convention, the
former-brand rule (atlas ADR 0001) and its four exemption categories, and the rule that renaming
anything another repo verifies is a coordinated migration with an ADR — live in `atlas/AGENTS.md`
and are not restated here.

`README.md` shows how to run a scan. This file says what must not break.

## Why this repository exists

**It holds the kubeconfig so the analysis never does.** `aep` reads an
`infra-observation/1` bundle as a file and holds no credential — that boundary is
`aep/docs/VISION.md` § *What this is deliberately not*, and this repository is
what lets it stay true. Every rule below is downstream of that one sentence.

## Invariants

Each is a claim that can be checked. Breaking one is a design change, not a refactor.

1. **A secret value never touches disk.** Every `data` and `stringData` value in a Secret is
   replaced by `{sha256, length}` before the bundle is written (`src/main.rs:154-176`). The *shape*
   — which secrets exist, which keys they hold — survives, because that is what dependency mapping
   needs. The values do not.
2. **The `kubectl.kubernetes.io/last-applied-configuration` annotation is dropped from Secrets**
   (`src/main.rs:177-183`). It can carry the values invariant 1 just removed; redacting one field
   and leaving its copy is not redaction.
3. **The bundle is otherwise raw.** Objects are recorded keyed by kind, exactly as the API returned
   them. Interpretation, validation, the IR, the digest, the dependency graph and diagnosis all live
   in the pure toolchain. **Nothing here analyses anything.**
4. **`KINDS` is the observation surface, and extending it is a format change.** Adding or removing a
   kind changes what every downstream consumer sees; it belongs in a commit that says so
   (`src/main.rs:24`).
5. **The credential stays on this side.** No bundle, log line, error message or stdout write may
   carry a kubeconfig path's contents, a token, or a client certificate.
6. **v0 shells out to `kubectl` on purpose.** Credential handling, exec plugins and API version skew
   stay kubectl's problem. Linking a Kubernetes client would move all three into this binary, which
   is a design decision and needs one — not a dependency bump.
7. **This tool reads. It does not apply.** Applying anything is not in the tool. When it arrives it
   goes behind an explicit flag with a server-side dry-run first, and never as a default path.

## Safety envelope

The whole binary is the safety envelope: it is the one process in the pipeline holding a live
cluster credential.

- **Redaction is unconditional and happens before the write** — never behind a flag, never
  "unless the operator asked for values", never as a post-processing pass over a file already on
  disk.
- **A new collected kind is a new leak surface.** Before adding one, answer what fields of it can
  carry a value, an annotation copy of a value, or a token — Secrets are the known case, not the
  only possible one.
- **Nothing here writes to a cluster** (invariant 7).
- **No credential in the repository.** No kubeconfig, no context dump, no captured bundle from a
  real cluster.

## Out of scope

| Belongs elsewhere | Repo |
|---|---|
| Validation, the `infra-ir/1`, the dependency graph, diagnosis rules, drift, projection | `aep` (`infra-domain`, `infra-compiler`, `infra-analyze`, `infra-spec`, `infra-project`) |
| Anything that mutates a cluster | nowhere yet — invariant 7 |
| Bundle *schema* ownership | `aep`, which validates `infra-observation/1` |

The split is the product: an actor that holds credentials and does nothing clever, and a toolchain
that is clever and holds nothing.

## The gate

There is no `scripts/gate.sh` here yet. Until there is, the gate is:

```console
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Green before any push.

**A green local gate does not guarantee a green CI.** The steps mirror each other; the toolchain does
not — CI installs whatever `stable` is that day, and a newer clippy can fail a commit that passed
locally. Run `rustup update` before pushing, and read the gate's own exit status, never a
pipeline's.

No gate step contacts a cluster. A test that needs a live cluster is not a test this gate runs.

There is no `scripts/check-brand.sh` in this repository. If a former brand name appears at the
surface, add the check rather than an exemption (atlas ADR 0001).

## Releases

The tag is the bare version — `0.2.0`, the version and nothing else (atlas § *Naming*), annotated, at
a fully gated `main` commit. There is no `CHANGELOG.md`; a change to `infra-observation/1` needs one
before it ships, because a downstream consumer verifies those bytes.

**A change to the bundle format is a coordinated migration**, not an edit: `aep`
reads `infra-observation/1` and pins committed examples against it. That needs an ADR in `atlas`
naming the relying parties and the order they move in.

## Where work is tracked

| What | Where |
|---|---|
| The consumer's plan for this data | `aep/docs/plan/infra-wave-1-observe.md`, `infra-wave-2-analyze.md` |
| The boundary this repository exists to hold | `aep/docs/VISION.md` § *What this is deliberately not* |

There is no backlog in this repository. Work on the scanner is scheduled from the consumer's plan
pages.

## Conventions

- Rust CLIs use `clap`'s derive API. Hand-rolled argv parsing is not accepted.
- Prefer no dependency. The manifest carries four, each with its reason on the line that adds it;
  `chrono` was refused and `date -u` used instead (`src/main.rs:192`). Keep that bar.

## Bot identity

Automated commits and pushes go through the GitHub App via `the private Atlas delivery procedure`, never a human
credential. `the private Atlas credential procedure` mints the token; its bot-org default (`the private Atlas credential procedure:8`)
is `beyond10x` today — confirm that before relying on it.
