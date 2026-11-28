Run from this directory with ess 0.99.9, target `interpreted`, `--path system.yaml` (a directory path would read `scenarios/` as specification files).

`system.yaml` synthesizes 11 scenarios, 2 of which fail on the interpreted target (the defect the
story `a-scenario-arranged-around-a-related-row-passes-the-commands-own-subject-refusals-and-sends-only-identities-some-row-carries`
names). `scenarios/` holds one authored scenario that passes. `known-failing.json` declares the 2
failures against the digest of the suite `mutate` synthesizes; `known-failing-from-the-gate.json`
declares them against the digest of the suite `run --scenarios scenarios` runs. Both bind the build
of the 0.99.9 executable by its SHA-256, so they are refused by another build.

```text
ess verify conform mutate --path system.yaml --target interpreted
  refusal[ESS-MUTATE-001], 2 scenarios failed

ess verify conform mutate --path system.yaml --target interpreted --scenarios scenarios
  refusal[ESS-MUTATE-001], the same 2

ess verify conform mutate --path system.yaml --target interpreted --scenarios scenarios --known-failing known-failing.json
  error: the argument '--scenarios <SCENARIOS>' cannot be used with '--known-failing <KNOWN_FAILING>'

ess verify conform mutate --path system.yaml --target interpreted --known-failing known-failing.json
  15 mutants, 8 killed, 3 inconclusive, 4 stillborn; the 2 declared scenarios are known failures

ess verify conform mutate --path system.yaml --target interpreted --known-failing known-failing-from-the-gate.json
  refused: the declaration is bound to suite bytes sha256:0572..., and the suite is sha256:5f58...
```

`ess verify conform run --target interpreted --report-format 2 --path system.yaml --scenarios scenarios`
reports 12 scenarios, the suite digest `sha256:0572...`; without `--scenarios` it reports 11 and
`sha256:5f58...`.
