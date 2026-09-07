---
format: aep.planning-md/1
id: review-result:cache-origin-source-pass1
kind: review-result
status: active
title: Cache-origin source attack pass 1
relations:
- reviews: story:review-cache-origin
revision: 1
---
unit: story:review-cache-origin — source 6fd6e796c580656b65d9199f9367ebe3b662f806 plus the test-only working tree
verdict: nothing found
cases: executed 226→235, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 assigned root — /home/timo/.cache/ess-w12-review1-tmp
needs-coordinator: none

1. git --no-pager diff --stat

The tracked diff is empty. Git's no-index diff accounts for the two new untracked test files:

```text
 .../ess-cli/tests/cache_origin_adversary_pass1.rs  | 623 +++++++++++++++++++++
 1 file changed, 623 insertions(+)
 .../tests/support/cache_origin_attack_client.rs    | 104 +++++++++++++++++++++
 1 file changed, 104 insertions(+)
```

The complete Git-produced test-only patch is test-only.patch. No production file, existing test,
public page, binding, dependency/lock, planning record or Git state was edited. The separately
retained subject.patch is the implementation diff I read, not a diff I wrote.

2. Cases written before the package run

All nine first executions selected exactly one case and were green. No assertion, compiler or fixture failure occurred. The first seven cases preceded the final two additions; each receipt preserves its exact before/after input hashes. The implementation baseline is the implementor's 226 passed, not a baseline suite run by this adversary.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 decoded_optional_duplicates_and_nulls_are_refused_cold_and_warm -- --exact`

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling syn v3.0.4
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling serde_json v1.0.151
   Compiling cfg-if v1.0.4
   Compiling itoa v1.0.18
   Compiling serde_derive v1.0.229
   Compiling typenum v1.20.1
   Compiling syn v2.0.119
   Compiling hybrid-array v0.4.14
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling const-oid v0.10.2
   Compiling equivalent v1.0.2
   Compiling digest v0.11.3
   Compiling hashbrown v0.17.1
   Compiling cpufeatures v0.3.1
   Compiling sha2 v0.11.0
   Compiling serde_derive_internals v0.29.1
   Compiling thiserror v2.0.20
   Compiling schemars v0.8.22
   Compiling schemars_derive v0.8.22
   Compiling indexmap v2.14.1
   Compiling thiserror-impl v2.0.20
   Compiling unsafe-libyaml v0.2.11
   Compiling ryu v1.0.23
   Compiling dyn-clone v1.0.20
   Compiling serde_yaml v0.9.34+deprecated
   Compiling autocfg v1.5.1
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-primitives)
   Compiling num-traits v0.2.19
   Compiling libc v0.2.189
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-domain)
   Compiling num-integer v0.1.47
   Compiling heck v0.5.0
   Compiling pulldown-cmark v0.13.4
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.56
   Compiling version_check v0.9.5
   Compiling ahash v0.8.12
   Compiling num-bigint v0.4.8
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-compiler)
   Compiling unicase v2.9.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.1
   Compiling parking_lot_core v0.9.12
   Compiling regex-syntax v0.8.11
   Compiling num-rational v0.4.2
   Compiling num-iter v0.1.46
   Compiling num-complex v0.4.6
   Compiling ref-cast-impl v1.0.27
   Compiling aho-corasick v1.1.5
   Compiling scopeguard v1.2.0
   Compiling utf8parse v0.2.2
   Compiling once_cell v1.21.4
   Compiling smallvec v1.16.0
   Compiling anstyle-parse v1.0.0
   Compiling lock_api v0.4.14
   Compiling regex-automata v0.4.18
   Compiling num v0.4.3
   Compiling ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-gen)
   Compiling infra-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-domain)
   Compiling colorchoice v1.0.5
   Compiling bit-vec v0.8.0
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling unicode-general-category v1.1.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling borrow-or-share v0.2.4
   Compiling fluent-uri v0.4.1
   Compiling infra-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-compiler)
   Compiling anstream v1.0.0
   Compiling bit-set v0.8.0
   Compiling fraction v0.17.0
   Compiling parking_lot v0.12.5
   Compiling strum_macros v0.28.0
   Compiling micromap v0.3.0
   Compiling bytecount v0.6.9
   Compiling vsimd v0.8.0
   Compiling clap_lex v1.1.0
   Compiling percent-encoding v2.3.2
   Compiling outref v0.5.2
   Compiling num-cmp v0.1.0
   Compiling strsim v0.11.1
   Compiling jsonschema-value v0.52.1
   Compiling clap_builder v4.6.6
   Compiling uuid-simd v0.8.0
   Compiling referencing v0.52.1
   Compiling strum v0.28.0
   Compiling infra-analyze v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-analyze)
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling jsonschema-regex v0.52.1
   Compiling clap_derive v4.6.4
   Compiling email_address v0.2.9
   Compiling anyhow v1.0.104
   Compiling data-encoding v2.11.1
   Compiling clap v4.6.6
   Compiling jsonschema v0.52.1
   Compiling infra-spec v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-spec)
   Compiling ess-conformance v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/verify/ess-conformance)
   Compiling ess-realization v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-realization)
   Compiling semver v1.0.28
   Compiling base64 v0.22.1
   Compiling ess-deployment v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-deployment)
   Compiling schema-contract v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/schema-contract)
   Compiling ess-diff v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/verify/ess-diff)
   Compiling infra-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-project)
   Compiling ess-kubernetes v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/ess-kubernetes)
   Compiling ess-synth v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-synth)
   Compiling ess-composition v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-composition)
   Compiling ess-openapi v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-openapi)
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 43.15s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test decoded_optional_duplicates_and_nulls_are_refused_cold_and_warm ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 1.12s

```

Direct exit: 0; wall seconds: 44.28994540893473. Original receipt: focused-1.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 original_profiles_and_decoded_utf8_limits_keep_exact_owned_bytes -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test original_profiles_and_decoded_utf8_limits_keep_exact_owned_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.54s

```

Direct exit: 0; wall seconds: 0.6119843400083482. Original receipt: focused-2.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 warm_frame_extremes_and_cross_profile_entries_never_fetch_or_consume -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test warm_frame_extremes_and_cross_profile_entries_never_fetch_or_consume ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.36s

```

Direct exit: 0; wall seconds: 0.4290336779085919. Original receipt: focused-3.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 integer_layer_tokens_refuse_before_any_fetch_of_a_blob -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test integer_layer_tokens_refuse_before_any_fetch_of_a_blob ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.36s

```

Direct exit: 0; wall seconds: 0.42356515605933964. Original receipt: focused-4.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 late_blob_outputs_must_be_regular_complete_and_successful -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test late_blob_outputs_must_be_regular_complete_and_successful ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.69s

```

Direct exit: 0; wall seconds: 0.818208466982469. Original receipt: focused-5.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 a_late_complete_or_corrupt_winner_is_preserved_without_replacement -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test a_late_complete_or_corrupt_winner_is_preserved_without_replacement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.21s

```

Direct exit: 0; wall seconds: 0.27583981794305146. Original receipt: focused-6.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped has been running for over 60 seconds
test a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 60.10s

```

Direct exit: 0; wall seconds: 60.16754902806133. Original receipt: focused-7.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 self_consistent_other_original_bytes_cannot_replace_the_requested_identity -- --exact`

```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test self_consistent_other_original_bytes_cannot_replace_the_requested_identity ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.47s

```

Direct exit: 0; wall seconds: 0.8378132940270007. Original receipt: focused-8.json.

Command: `cargo test --locked -p ess-cli --test cache_origin_adversary_pass1 offline_helm_consumes_the_owned_snapshot_after_shared_entry_replacement -- --exact`

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 1 test
test offline_helm_consumes_the_owned_snapshot_after_shared_entry_replacement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.10s

```

Direct exit: 0; wall seconds: 0.17082296905573457. Original receipt: focused-9.json.

The assertions of the nine cases, all green in the final package run:

- decoded_optional_duplicates_and_nulls_are_refused_cold_and_warm checks escaped duplicate keys in
  every admitted annotation map, duplicate optional annotation fields, present-null and non-string
  annotations, plus null artifact/data options, with valid requested raw identity and both consumers.
- original_profiles_and_decoded_utf8_limits_keep_exact_owned_bytes exercises original whitespace,
  bundle fixed data absent/present, Helm artifactType absent/present, both provenance orders and
  zero-length opaque content, and decoded UTF-8 key/value limits; cold and offline warm bytes agree.
- warm_frame_extremes_and_cross_profile_entries_never_fetch_or_consume independently frames every
  blob, tests truncation boundaries, zero/max counts, each length maximum plus one and u64::MAX,
  cross-profile namespace confusion and concatenated trailing proofs, preserving refused entries.
- integer_layer_tokens_refuse_before_any_fetch_of_a_blob attacks later layer/provenance exact
  integer tokens while the requested manifest hash and unrelated fields remain valid.
- late_blob_outputs_must_be_regular_complete_and_successful attacks final bundle/provenance output
  failures, missing, directory, symlink and truncated content, checks absent/existing output and
  empty invocation staging, and drains a successful late client's 512 KiB on each stream.
- a_late_complete_or_corrupt_winner_is_preserved_without_replacement introduces a complete or
  corrupt winner through a separate actual process after acquisition starts; final winner bytes
  remain intact, valid content is consumed and corrupt content refuses without replacement.
- a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped spends 20, 10 and 5 seconds
  in earlier fetches, then stalls on provenance; refusal still occurs near 60 seconds and the
  actual owned process is observed absent from /proc before returning from the case.
- self_consistent_other_original_bytes_cannot_replace_the_requested_identity tests whitespace-only
  and self-consistent annotated substitutes at the old requested digest, cold and warm, against
  both consumers and absent/existing bundle outputs, with unchanged corrupt-entry bytes.
- offline_helm_consumes_the_owned_snapshot_after_shared_entry_replacement replaces the warm shared
  entry as actual fake Helm starts, checks the original consumed bytes and snapshot cleanup after
  completion, then observes the next invocation refuse the corrupt shared entry without another call.

3. Suite and strict checks

The first complete package run also passed 235 cases in 172.700027770 seconds. Subsequent strict
Clippy found only my new helper's four-boolean constructor, then the replacement enum lacking Copy.
Those two original lint outputs are retained below. I changed the test helper's content selector
into the Copy enum Content::{Original, Empty}; no assertion was removed, skipped or weakened.
The replacement script first expected 12 call sites and stopped before writing when it counted 14;
that bookkeeping guard and the exact corrected count are retained in helper-correction.txt.
After the helper correction, strict Clippy, formatting and the full package ran against identical
source inputs. Their final raw outputs follow. The first seven source bytes were reconstructed
from their retained prefix and verified against the original receipt hash; the first package's
attack binary and full nine-case source are retained before-clippy-helper-correction/.

Command: `cargo clippy --locked -p ess-cli --all-targets -- -D warnings`

```text
    Checking serde_core v1.0.229
    Checking memchr v2.8.3
    Checking itoa v1.0.18
    Checking cfg-if v1.0.4
    Checking zmij v1.0.23
    Checking typenum v1.20.1
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking hybrid-array v0.4.14
    Checking crypto-common v0.2.2
    Checking block-buffer v0.12.1
    Checking allocator-api2 v0.2.21
    Checking foldhash v0.2.0
    Checking equivalent v1.0.2
    Checking const-oid v0.10.2
    Checking hashbrown v0.17.1
    Checking digest v0.11.3
    Checking cpufeatures v0.3.1
    Checking sha2 v0.11.0
    Checking indexmap v2.14.1
    Checking dyn-clone v1.0.20
    Checking unsafe-libyaml v0.2.11
    Checking ryu v1.0.23
    Checking schemars v0.8.22
    Checking serde_yaml v0.9.34+deprecated
    Checking thiserror v2.0.20
    Checking ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-primitives)
    Checking num-traits v0.2.19
    Checking ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-domain)
    Checking num-integer v0.1.47
    Checking libc v0.2.189
    Checking num-bigint v0.4.8
    Checking ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-compiler)
    Checking regex-syntax v0.8.11
    Checking pulldown-cmark-escape v0.11.0
    Checking unicase v2.9.0
    Checking bitflags v2.13.1
    Checking pulldown-cmark v0.13.4
    Checking num-rational v0.4.2
    Checking getrandom v0.3.4
    Checking zerocopy v0.8.56
    Checking num-iter v0.1.46
    Checking num-complex v0.4.6
    Checking aho-corasick v1.1.5
    Checking smallvec v1.16.0
    Checking scopeguard v1.2.0
    Checking once_cell v1.21.4
    Checking utf8parse v0.2.2
    Checking anstyle-parse v1.0.0
    Checking lock_api v0.4.14
    Checking parking_lot_core v0.9.12
    Checking ref-cast v1.0.27
    Checking regex-automata v0.4.18
    Checking ahash v0.8.12
    Checking num v0.4.3
    Checking ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-gen)
    Checking infra-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-domain)
    Checking anstyle v1.0.14
    Checking is_terminal_polyfill v1.70.2
    Checking bit-vec v0.8.0
    Checking borrow-or-share v0.2.4
    Checking anstyle-query v1.1.5
    Checking colorchoice v1.0.5
    Checking infra-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-compiler)
    Checking anstream v1.0.0
    Checking fluent-uri v0.4.1
    Checking bit-set v0.8.0
    Checking fraction v0.17.0
    Checking parking_lot v0.12.5
    Checking bytecount v0.6.9
    Checking vsimd v0.8.0
    Checking strsim v0.11.1
    Checking num-cmp v0.1.0
    Checking micromap v0.3.0
    Checking clap_lex v1.1.0
    Checking percent-encoding v2.3.2
    Checking outref v0.5.2
    Checking uuid-simd v0.8.0
    Checking referencing v0.52.1
    Checking clap_builder v4.6.6
    Checking jsonschema-value v0.52.1
    Checking strum v0.28.0
    Checking infra-analyze v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-analyze)
    Checking fancy-regex v0.19.0
    Checking unicode-general-category v1.1.0
    Checking regex v1.13.1
    Checking jsonschema-regex v0.52.1
    Checking email_address v0.2.9
    Checking data-encoding v2.11.1
    Checking clap v4.6.6
    Checking infra-spec v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-spec)
    Checking jsonschema v0.52.1
    Checking ess-conformance v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/verify/ess-conformance)
    Checking ess-realization v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-realization)
    Checking semver v1.0.28
    Checking base64 v0.22.1
    Checking ess-deployment v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-deployment)
    Checking schema-contract v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/schema-contract)
    Checking ess-diff v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/verify/ess-diff)
    Checking infra-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/infra-project)
    Checking ess-kubernetes v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/infra/ess-kubernetes)
    Checking anyhow v1.0.104
    Checking ess-synth v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-synth)
    Checking ess-composition v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/specify/ess-composition)
    Checking ess-openapi v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/generate/ess-openapi)
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/edge/ess-cli)
error: more than 3 bools in function parameters
   --> crates/edge/ess-cli/tests/cache_origin_adversary_pass1.rs:100:5
    |
100 | /     fn new(bundle: bool, reverse: bool, empty: bool, optional: bool) -> Self {
101 | |         let (config, chart) = if bundle {
102 | |             (
103 | |                 b"{}".to_vec(),
...   |
149 | |     }
    | |_____^
    |
    = help: consider refactoring bools into two-variant enums
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#fn_params_excessive_bools
    = note: `-D clippy::fn-params-excessive-bools` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::fn_params_excessive_bools)]`

error: could not compile `ess-cli` (test "cache_origin_adversary_pass1") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

Direct exit: 101; wall seconds: 23.121921663056128. Receipt: clippy.json.

Command: `cargo clippy --locked -p ess-cli --all-targets -- -D warnings`

```text
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/edge/ess-cli)
error: this argument is passed by value, but not consumed in the function body
   --> crates/edge/ess-cli/tests/cache_origin_adversary_pass1.rs:105:50
    |
105 |     fn new(bundle: bool, reverse: bool, content: Content, optional: bool) -> Self {
    |                                                  ^^^^^^^
    |
help: or consider marking this type as `Copy`
   --> crates/edge/ess-cli/tests/cache_origin_adversary_pass1.rs:93:1
    |
 93 | enum Content {
    | ^^^^^^^^^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
    = note: `-D clippy::needless-pass-by-value` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::needless_pass_by_value)]`
help: consider taking a reference instead
    |
105 |     fn new(bundle: bool, reverse: bool, content: &Content, optional: bool) -> Self {
    |                                                  +

error: could not compile `ess-cli` (test "cache_origin_adversary_pass1") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

Direct exit: 101; wall seconds: 0.24564346997067332. Receipt: final-clippy.json.

Command: `cargo clippy --locked -p ess-cli --all-targets -- -D warnings`

```text
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.16s
```

Direct exit: 0; wall seconds: 0.20583555800840259. Receipt: sealed-clippy.json.

Command: `cargo fmt -p ess-cli --check`

```text
```

Direct exit: 0; wall seconds: 0.20491094409953803. Receipt: sealed-format.json.

Command: `cargo test --locked -p ess-cli`

```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-cache-origin/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.46s
     Running unittests src/main.rs (target/debug/deps/ess-63929f179d60f606)

running 17 tests
test oci_cache::tests::every_binary_frame_bound_is_checked_before_read_allocation ... ok
test coverage::tests::suite5_pair_refusal_precedes_target_construction ... ok
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test oci_cache::tests::owned_open_handle_keeps_verified_bytes_after_shared_path_replacement ... ok
test oci_cache::tests::complete_concurrent_winner_is_reused_and_corrupt_winner_is_preserved ... ok
test oci_cache::tests::staged_write_failure_and_failed_hard_link_leave_no_final_entry ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test oci_cache::tests::descriptor_limit_equal_is_admitted_and_plus_one_is_refused ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-3ba618a94d75005d)

running 25 tests
test author_empty ... ok
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test web_empty ... ok
test web_nested_only ... ok
test go_nonmatching_only ... ok
test run_empty ... ok
test run_nested_only ... ok
test run_nonmatching_only ... ok
test go_empty ... ok
test ir_nonmatching_only ... ok
test go_nested_only ... ok
test ir_nested_only ... ok
test web_nonmatching_only ... ok
test ir_empty ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-fbc20393f78d5213)

running 9 tests
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

     Running tests/authored_site.rs (target/debug/deps/authored_site-1a420fe3beb09d76)

running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-00fd33a384eed561)

running 1 test
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-25ef58b4641736f8)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.98s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-ba03872d9e795d41)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-d76d1dd27f537ab9)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/cache_origin.rs (target/debug/deps/cache_origin-ff23be205e84970c)

running 21 tests
test legacy_self_consistent_helm_cache_cannot_claim_requested_origin ... ok
test shared_cache_replacement_cannot_change_helm_private_snapshot ... ok
test legacy_entries_and_interrupted_stages_are_retained_while_cold_proof_is_published ... ok
test symlink_and_directory_entries_refuse_without_following_or_replacing ... ok
test legacy_bundle_cache_is_ignored_and_cold_failure_preserves_both_sentinels ... ok
test failed_later_chart_stops_its_helm_call_after_preserving_earlier_release ... ok
test descriptor_proof_does_not_admit_noncanonical_or_invalid_bundle_semantics ... ok
test client_failures_missing_output_and_bounded_diagnostics_preserve_admission ... ok
test two_real_writers_publish_one_complete_proof_without_replacement ... ok
test warm_substitution_rechecks_every_blob_and_requested_manifest_identity ... ok
test both_consumers_reject_wrong_original_manifest_even_whitespace_only ... ok
test both_finite_profiles_cold_then_offline_warm_revalidate_original_bytes ... ok
test corrupt_warm_entries_are_not_repaired_and_never_launch_a_client ... ok
test descriptor_changes_and_size_disagreement_refuse_at_exact_blob_boundary ... ok
test all_descriptor_limits_refuse_before_fetch_and_returned_reads_are_bounded ... ok
test unsupported_fields_profiles_and_nulls_refuse_before_any_blob_call ... ok
test manifest_limit_accepts_exact_bound_and_refuses_one_more_original_byte ... ok
test unknown_fields_and_embedded_data_refuse_before_any_blob_call ... ok
test duplicate_keys_and_noninteger_tokens_keep_valid_requested_identity ... ok
test annotation_limits_apply_to_every_map_with_boundaries_admitted ... ok
test actual_stalled_client_is_killed_and_reaped_at_the_shared_deadline has been running for over 60 seconds
test actual_stalled_client_is_killed_and_reaped_at_the_shared_deadline ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 60.10s

     Running tests/cache_origin_adversary_pass1.rs (target/debug/deps/cache_origin_adversary_pass1-fa392af442fc4775)

running 9 tests
test offline_helm_consumes_the_owned_snapshot_after_shared_entry_replacement ... ok
test a_late_complete_or_corrupt_winner_is_preserved_without_replacement ... ok
test warm_frame_extremes_and_cross_profile_entries_never_fetch_or_consume ... ok
test integer_layer_tokens_refuse_before_any_fetch_of_a_blob ... ok
test self_consistent_other_original_bytes_cannot_replace_the_requested_identity ... ok
test original_profiles_and_decoded_utf8_limits_keep_exact_owned_bytes ... ok
test late_blob_outputs_must_be_regular_complete_and_successful ... ok
test decoded_optional_duplicates_and_nulls_are_refused_cold_and_warm ... ok
test a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped has been running for over 60 seconds
test a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 60.11s

     Running tests/command_surface.rs (target/debug/deps/command_surface-71a1dc5a6624aa01)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-604eebcbf9b30edb)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-aeedc7adf842489b)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-1a7e7875d1ea7c87)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-3af0e671176d1e0b)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s

     Running tests/coverage_browser.rs (target/debug/deps/coverage_browser-bf192040179e69f7)

running 4 tests
test retained_legacy_player_bytes_still_replay_in_actual_firefox ... ok
test actual_browser_admits_the_pair_before_creating_replay_state ... ok
test actual_browser_and_rust_refuse_every_closed_model_field_boundary ... ok
test actual_browser_checks_full_lineage_and_integer_metadata ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.70s

     Running tests/coverage_cli.rs (target/debug/deps/coverage_cli-994c933e9b49eced)

running 6 tests
test coverage_cli_refuses_binary64_model_before_each_new_production_surface ... ok
test coverage_cli_authored_roots_relocate_without_losing_exact_text_and_refuse_unrepresentable_paths ... ok
test explicit_suite5_cli_produces_exact_inventory_and_requires_report2_before_execution ... ok
test select_cli_preserves_all_parent_bytes_and_explicit_empty_selection ... ok
test impact_cli_requires_exact_complete_input_and_keeps_the_persisted_v3_shape ... ok
test generated_go_executes_the_admitted_coverage_inventory_and_preserves_pairing_defaults ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

     Running tests/coverage_lineage.rs (target/debug/deps/coverage_lineage-a42eee64e165b553)

running 2 tests
test generated_go_checks_original_lineage_and_typed_defaults ... ok
test go_execution_adapts_only_selected_integer_fields_before_target_effects ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.36s

     Running tests/coverage_producers.rs (target/debug/deps/coverage_producers-1b45cfea40b4b957)

running 3 tests
test actual_rust_coverage_exports_match_the_independent_plan ... ok
test actual_coverage_producers_refuse_noninvoked_negative_clock_and_report1_without_output ... ok
test actual_go_coverage_exports_match_the_independent_plan ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.14s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 2 tests
test go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown ... ok
test browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.91s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-efcbea4be28fb826)

running 1 test
test browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.95s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-ac70ba28528e59d8)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-d5a347d9ae9103cd)

running 13 tests
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.52s

     Running tests/model_types.rs (target/debug/deps/model_types-b0076ddf3b38585d)

running 3 tests
test all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight ... ok
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-47a1f33d5a547d54)

running 17 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test output_cannot_replace_any_declared_input ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.88s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-406279f2f9d35352)

running 2 tests
test cli_runtime_grammar_controls_preserve_existing_output ... ok
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/normalization_typescript.rs (target/debug/deps/normalization_typescript-5542aa1874e06ad7)

running 7 tests
test unqualified_profile_refuses_with_source_pointer_and_no_partial_files ... ok
test model_inputs_are_recompiled_pinned_and_protected_before_typescript_generation ... ok
test retained_recipe_and_bundle_inputs_cannot_be_overwritten ... ok
test typescript_cli_emits_an_accounted_executable_package ... ok
test module_and_late_path_conflicts_refuse_before_any_publication ... ok
test generated_file_and_parent_links_refuse_without_touching_their_destinations ... ok
test every_planned_file_participates_in_read_only_drift_checks ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-1ef9acb3dfcb1eba)

running 1 test
test generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-3024341d0a106a96)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-d2bcefe19c39a1dd)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-ae258705b1349988)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/output_containment.rs (target/debug/deps/output_containment-5c56b28ecc91698b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-8deef286ecfcabd1)

running 6 tests
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-6f1b8cef6ccfdb14)

running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/schema_registry_identity.rs (target/debug/deps/schema_registry_identity-1d7aadfcc5659f8d)

running 9 tests
test idless_generated_schemas_are_not_registry_resources ... ok
test typescript_uses_root_id_and_preserves_equal_and_stale_check_behavior ... ok
test accepted_instances_can_coexist_with_later_selector_failures ... ok
test typescript_id_and_projection_refusals_happen_before_output_writes ... ok
test offline_missing_references_and_exact_duplicate_ids_refuse_the_registry ... ok
test syntax_acceptance_is_distinct_from_domain_roster_assembly ... ok
test both_dialects_accept_separate_adopter_resources_and_report_the_envelope ... ok
test selectors_and_strict_envelopes_do_not_modify_domain_payloads ... ok
test nested_payload_constraints_are_checked_through_both_resource_roots ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-cf3f1a6623400628)

running 6 tests
test registry_admission_and_selected_typescript_projection_have_distinct_boundaries ... ok
test inlining_idless_generated_payloads_breaks_their_original_root_references ... ok
test decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity ... ok
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok
test envelope_definitions_cannot_shadow_either_payload_resource_root ... ok
test the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/target_failure.rs (target/debug/deps/target_failure-7d9cc8e18982b595)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

```

Direct exit: 0; wall seconds: 158.57862060295884. Receipt: sealed-package.json.

The baseline count comes from the handed-off implementor report (SHA256
e929825bf3e7686c2dadb4e0f762d9721025b8e7062dac435057f12e0b52a873), not a preliminary suite run.
The final package has 37 native summaries, 235 passed, 0 failed and 0 ignored. The newly selected
integration target ran all nine cases. The pre-existing 36 lanes retain their counts, including
17 main tests, 21 cache-origin tests and six persisted-delivery tests. native-case-counts.json
records each exact runner and summary. All final check inputs match before/after and between checks.

4. Judgement findings

No findings. This pass covers source 6fd6e796c580656b65d9199f9367ebe3b662f806 and the two added tests.
There is no origin classification to infer or route. The original implementation base was read as
Git objects; no base behavior was labeled pre-existing without a run, and the tree was never moved.

5. Attacks that did not break the change

The nine new actual-CLI cases retained requested raw identity, descriptor closure, finite profiles,
closed JSON admission, bounded frame reads, complete-winner preservation and owned consumption.
The actual final-provenance stall retained the original 60-second deadline and observed its owned
PID dead; the full package also reran both existing bundle/Helm deadline controls.
The full unchanged cache target reran legacy cold misses, all descriptor/read bounds, original
identity and descriptor mutations' targeted assertions, full bounded diagnostics, two real writers,
interrupted stages and the invocation snapshot; deterministic production-unit I/O seams also ran.
The existing six persisted-delivery cases preserved invalid-plan/prevalidation and the exact positive
three ORAS calls followed by two Helm calls for one reused chart; the existing later-chart test
preserved an earlier completed release and stopped the failing later release.
These were local synthetic ORAS/Helm peers. They do not establish publisher authorization, actual
Helm archive/provenance semantics, a hostile-process sandbox, power-loss durability, general cache
cleanup, rollback or execution recovery. No live registry, cluster or executor was contacted.
The coordinator's full integration, public delivery and planning/Git lifecycle work were not run here.

6. Every path written outside the worktree and the handoff

The only assigned external writable root was:

```text
/home/timo/.cache/ess-w12-review1-tmp
```

All test/CLI temporary writes, including retained original-byte fixtures, locally compiled peer
executables and actual process output records, are beneath that root. external-paths.txt lists every
retained path in full as a JSON-quoted string, preserving non-UTF-8 native names through escapes;
external-inventory.json records its native metadata, regular-file SHA-256
and literal symlink target without following links. Existing test-owned temporary deletion and
production staging/snapshot Drop behavior remain their normal behavior; no agent cleanup ran.
Cargo output remained this unit's target. CARGO_HOME and GOCACHE were assigned scratch children;
CARGO_HOME's offline Git and registry links are inventoried literally. Stable Rust and Node24 paths,
versions and hashes are in tool-identities.json, and all command receipts retain the exact environment,
clock times, direct exit and free-disk measurements. No observed measurement approached the 8 GiB floor.
source-tool-inputs.json records repository inputs plus the role/brief/authority/report inputs.
executed-binaries.json records final package runner and actual CLI identities. The complete scratch
census and file records are scratch-inventory.json. seal.json binds these records and this report;
seal-readback.json records the subsequent full hash/native-metadata/census verification and the
actual dead-child readback. A first path-list rendering attempt encountered an existing non-UTF-8 filename and stopped before
creating the seal; seal-attempt1-error.txt preserves the bookkeeping correction. Only
self-referential seal bookkeeping and the documented scratch-root
metadata changes from writing that bookkeeping are excepted. No more writes occur after handing off.

7. Findings for the coordinator

```findings
[]
```
