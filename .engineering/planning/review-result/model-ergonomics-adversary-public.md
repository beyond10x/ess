---
format: aep.planning-md/3
id: review-result:model-ergonomics-adversary-public
kind: review-result
status: active
title: Generated model ergonomics adversary, publication-safe record
relations:
- reviews: story:feature-request-406
revision: 1
---
unit: story:feature-request-406 at 7b3d70c7c527a1a2ba2540fcc01a48ef2b43db25
verdict: NEEDS-CHANGE
cases: executed 10→15, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 18 paths under assigned scratch
needs-coordinator: land the clarified timestamp contract with the tests-only patch

```text
 crates/edge/ess-cli/tests/client.rs                                 |  25 ++
 crates/edge/ess-cli/tests/model_types.rs                            |  46 ++++
 crates/generate/schema-contract/tests/adversary_model_ergonomics.rs | 206 ++++++++++++++++++++
 3 files changed, 277 insertions(+)
```

The frozen tests-only patch is 9,701 bytes with SHA-256
`9cc389f9ae8fc6c328c93157eba93d6432cd415c3aef6ee25712e8f1a201d51c`. The reviewed commit diff
from parent `e68684efb6a4ac22052c77d3ed8292fd44f9ace5` has SHA-256
`7dce2a6388f7a6b4a203840350f5c8edb0112ca4ffd2736902175fb2f4b8274d`.

The added cases are:

- `crates/generate/schema-contract/tests/adversary_model_ergonomics.rs`: generated Rust accepts a
  fractional timestamp, carries the negative `i64` constant through `Default`, and writes the
  canonical RFC 3339 spelling. It is green now. Before the contract clarification its exact
  byte-fidelity assertion was red:

  ```text
  assertion `left == right` failed
    left: "{\"createdAt\":\"2026-10-03T00:00:00.5+02:00\",\"revision\":-2147483649}"
   right: "{\"createdAt\":\"2026-10-03T00:00:00.500+02:00\",\"revision\":-2147483649}"
  ```

- The same file: generated Go accepts the same timestamp, carries the negative `i64` constant
  through `NewProbeMeterRevision`, and writes the canonical spelling. It is green now. Its original
  assertion was red:

  ```text
  wire spelling changed: got {"createdAt":"2026-10-03T00:00:00.5+02:00","revision":-2147483649}, want {"createdAt":"2026-10-03T00:00:00.500+02:00","revision":-2147483649}
  ```

- The same file: a reachable `ReadingSamples` positional-name collision uses Go's documented,
  deterministic hash fallback while Rust retains its native `Vec` field. It is green.
- `crates/edge/ess-cli/tests/model_types.rs`: the default stays qualified and `--names short`
  reaches the real `generate types` command. It is green.
- `crates/edge/ess-cli/tests/client.rs`: `--names short` reaches the real `generate client` command
  and its generated publisher payload type. It is green.

The scoped suite ran after all five cases existed:

```text
$ cargo test -p schema-contract --locked --test adversary_model_ergonomics --test model_ergonomics --jobs 2 -- --nocapture
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
exit 0

$ cargo test -p ess-cli --locked --test model_types --test client --jobs 2 -- --nocapture
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
exit 0

$ cargo test -p schema-contract --locked --test binary64_structural complete_non_binary64_output_maps_remain_identical --jobs 2 -- --exact --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
exit 0
```

The current owned test sources have these identities:

```text
65c88f0a83f4a242b4f8c5f29da77bc868657a8ce4c5822d4d028a807fda2a42  crates/edge/ess-cli/tests/client.rs
5a2e6f2d33bd6e07d592951e705b5f088a5983442844750cb536aeab21826d16  crates/edge/ess-cli/tests/model_types.rs
429c14608196a2e3ebfb5bf59c19ab84068367e182573af6ed51181d61e60da2  crates/generate/schema-contract/tests/adversary_model_ergonomics.rs
```

Judgement finding:

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| `.engineering/planning/story/feature-request-406.md:29` | contract-drift | warning | NEEDS-CHANGE | introduced | Any valid timestamp spelling with non-canonical fractional precision, including `.500+02:00`, is accepted by the required native types and serialized canonically as `.5+02:00`. |

The reviewed story promises literal JSON byte fidelity while also requiring native Go `time.Time`
and Rust `time::OffsetDateTime`. Those native APIs preserve the timestamp value and numeric offset,
then emit canonical RFC 3339. The coordinator resolved this by clarifying the story and design
document; the generator needs no production change.

Could not break: invalid timestamp rejection, negative `i64` constant defaults and constructors,
positional collision determinism, qualified-name defaults, either short-name CLI route, publisher
type propagation, or the existing bundle-output witness. Rust formatting and `git diff --check`
also passed. Full repository checks, Clippy, site build and release work were outside this review.

Paths written outside the worktree, expressed relative to the assigned scratch root:

```text
assigned-scratch/
assigned-scratch/tmp/
assigned-scratch/adversary-test.log
assigned-scratch/collision-probe.log
assigned-scratch/collision-rerun.log
assigned-scratch/collision-green.log
assigned-scratch/adversary-final.log
assigned-scratch/adversary-red-preserved.log
assigned-scratch/adversary-green.log
assigned-scratch/short-name-cli.log
assigned-scratch/schema-affected-full.log
assigned-scratch/cli-affected-full.log
assigned-scratch/bundle-witness.log
assigned-scratch/tests-only.patch
assigned-scratch/findings.yaml
assigned-scratch/report.public.md
assigned-scratch/report.raw.md
assigned-scratch/report.public.corrected.md
```

```findings
- file: .engineering/planning/story/feature-request-406.md
  line: 29
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The byte-for-byte timestamp promise conflicts with the required native Go and Rust timestamp types, which emit canonical RFC 3339 spellings.
```
