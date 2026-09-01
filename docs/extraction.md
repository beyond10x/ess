# Extraction baseline

The standalone repository retains filtered history for ESS, infrastructure, schema-contract,
example, suite, generated-artifact, and Kubernetes adapter paths. Unrelated planning, protocol,
integration, website, and governance paths are not part of this repository.

The extraction keeps the existing persisted formats (`ess-diff/1`, `ess-impact/2`,
`ess-conformance/1`, `infra-observation/1`, `infra-ir/1`, and the infrastructure report formats).
No shared IR envelope or `ess-ir/2` was introduced. The new `ess-conformance-report/1` is an
additional standalone report rather than a changed existing envelope.

Generated fixtures remain governed by their existing byte-determinism tests. The canonical
standalone command is `ess`.
