# AGENTS.md — ess-kubernetes

The Kubernetes credential-edge adapter inside ESS. The repository-wide contract in the root
`AGENTS.md` applies here; these rules describe the additional credential boundary.

## Serves

- **O1 — governed reach.** Only this adapter may invoke `kubectl` with caller-selected authority.
- **O2 — decisions as data, with evidence.** It produces sanitized `infra-observation/1` input for
  the deterministic infrastructure compiler.

## Invariants

1. Secret `data` and `stringData` values are replaced with their digest and length before any
   serialization or filesystem write.
2. The last-applied configuration annotation is removed from Secrets because it may duplicate
   values already redacted from the Secret body.
3. The adapter preserves observation facts but performs no diagnosis, planning, or projection.
4. The collected kind list is explicit and ordered. Extending it requires a leak-surface review
   and a compatibility decision for `infra-observation/1`.
5. Credentials, kubeconfig contents, tokens, certificates, and unsanitized observations never
   appear in output, diagnostics, fixtures, or repository files.
6. Live scans use `kubectl` intentionally so exec plugins and API-version negotiation stay at the
   credential edge. Replacing it with a client library is a design change.
7. This adapter reads clusters. It never applies or mutates infrastructure.

## Gate

The root `task check` is offline and includes this crate's redaction, determinism, malformed-input,
and import tests. Live-cluster checks require explicit caller credentials and remain outside it.

Verify a redaction guard by mutating the sanitizer, observing the named test fail, and restoring the
guard before committing. Use the Atlas-owned bot tooling for commits; this public repository never
carries credential or token-minting helpers.
