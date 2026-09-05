unit: story:review-semantic-diff-coverage — complete F01 semantic coverage and versioned sliced provenance
verdict: green
cases: executed 459→483, red 15
origin: n/a
wrote-outside-worktree: none
needs-coordinator: yes — independent review, integration and remaining downstream rollout checks; no outside-scope source patch

## 1. Unit and acceptance

story:review-semantic-diff-coverage: every F01 omission produces semantic changes and conservative artifact obligations, mixed unclassified edits remain conservative, graph slices gain honest dependencies, and corrected sliced provenance/versioned writers preserve the explicitly frozen whole-model contracts.

Base: 28e97095d9e06c8b4585876a681a5eda5278c1ab. Branch: impl/review-semantic-diff-coverage. All source, tests, design and regeneration changes are uncommitted. The coordinator approved the binding addendum before production edits; baseline and initial test preparation preceded implementation.

Confirmed scope: existing graph, diff, impact and provenance owners are the required repair surfaces. Graph directions remain dependent→dependency, reverse impact closure, forward artifact slicing and before/after graph union. Whole provenance bypasses graph; that inferred compatibility premise was verified in source and through frozen identity/index/HTTP plan controls. The CLI consumes these contracts without requiring a source edit; no impact parser was invented. The new private stamp module stays within ess-gen. Corpus snapshots are within its assigned package. The source-derived four HTTP embedded payload differences were measured, with all other emitted HTTP artifacts equal to their committed files. The broader generated reservation produced 37 projection differences and five corpus differences, reported to the coordinator before retention.

The binding front addendum in docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md supersedes the historical format/coverage/graph and AEP framing. It binds ess-diff/2, ess-impact/3, slice-sha256/2:<64lowerhex>, version-specific vocabulary admission, checked legacy writing and authoritative stamp framing. No specific supplied code-move mechanism was taken on faith: the omitted-family, graph, format, stamp and independent residual decisions were measured red first.

Implemented acceptance coverage:

- Entity relation names/kinds/targets/cardinality/via, component reach and top-level/grouped CLI configuration, outcome sets/refusal behavior, and view parameters/ranking now have typed semantic changes. Changes preserve category ordering and existing IDs; each new subtype requires diff v2. Parameter naming defaults retain their existing effective-equivalence control.
- An independently compared residual normalized IR detects unclassified families even alongside classified changes. Known compared fields and documented derived facts are removed explicitly. Common transition order has an independent residual check; parsed-predicate equivalence and explicit naming-default equivalence remain covered. Unclassified changes force conservative whole obligations with the existing UncomparedFamilyChanged reason.
- Four relation spellings are added, retaining all original 21 variants: relation-target, ownership-carrier, exposes-view, parameter-type. The ownership reverse carrier edge accounts for target artifacts that receive owner annotations. Tests exercise the actual compiled graph and both sides of unioned impact.
- New readers accept frozen valid diff v1 and current v2, reject v2-only changes mislabeled v1, and reject unsupported formats. The explicit legacy writer and Serialize implementation refuse unrepresentable envelopes. A base-writer v1 fixture is retained and round-trips byte-identically.
- Every Constructs stamp uses the new profile even when its underlying hash is unchanged. Legacy bare slice stamps are owed. The new reader admits authoritative comment/structured envelopes, validates duplicate keys and conflicting locations, and refuses malformed/unknown profiles without falling back to marker-looking content. Docs-ir remains per-page nested provenance; it is deliberately not flattened into a first-page whole-document stamp.

The decisive regression matrix used valid compiled source and public diff/impact/provenance boundaries. It includes relation mutation/removal; reach; CLI grouping/binary/views; outcome sets and refusal independent of payload/error changes; parameter effective naming and ranking removal; pure/mixed unclassified references; pure/mixed transition order; reverse ownership closure and old/new graph union; all new subtype read/write admission; emitted stamp forms and actual legacy reader behavior; unchanged whole identities and HTTP plan bytes.

## 2. Actual diff

Tracked diff stat (git --no-pager diff --stat):

```text
 crates/generate/ess-gen/src/provenance.rs          |  51 +--
 .../corpus/billing/docs/domains/billing-email.md   |   4 +-
 .../corpus/billing/docs/domains/billing-invoice.md |   4 +-
 .../corpus/gatepass/docs/domains/gatepass-visit.md |   4 +-
 .../oracle-fixture/docs/domains/oracle-dispatch.md |   4 +-
 .../oracle-fixture/docs/domains/oracle-order.md    |   4 +-
 crates/generate/ess-gen/tests/openapi.rs           |  11 +-
 crates/generate/ess-gen/tests/provenance.rs        | 249 ++++++++++++
 crates/generate/ess-synth/tests/http.rs            |  37 ++
 crates/specify/ess-compiler/src/graph.rs           |  50 ++-
 crates/verify/ess-diff/src/change.rs               | 190 ++++++++-
 crates/verify/ess-diff/src/delta.rs                |  98 ++++-
 crates/verify/ess-diff/src/diff.rs                 | 451 +++++++++++++++++++--
 crates/verify/ess-diff/src/impact.rs               |  86 ++--
 crates/verify/ess-diff/src/lib.rs                  |   4 +-
 crates/verify/ess-diff/src/raw.rs                  |  11 +
 crates/verify/ess-diff/tests/artifacts.rs          |  70 +++-
 crates/verify/ess-diff/tests/canonical.rs          |  55 ++-
 crates/verify/ess-diff/tests/families.rs           | 212 ++++++++++
 crates/verify/ess-diff/tests/graph.rs              | 130 ++++++
 crates/verify/ess-diff/tests/impact.rs             |  26 +-
 ...s-semantic-diff-impact-evolution-design-v0.1.md | 170 +++++++-
 generated/asyncapi/email-service.yaml              |   4 +-
 generated/asyncapi/invoice-service.yaml            |   4 +-
 generated/docs/domains/billing-email.md            |   4 +-
 generated/docs/domains/billing-invoice.md          |   4 +-
 generated/go/gatepass/server/pass-service.docs.md  |  12 +-
 .../go/gatepass/server/pass-service.openapi.json   | 211 +++++++++-
 generated/openapi/email-service.yaml               |   4 +-
 generated/openapi/invoice-service.yaml             |   4 +-
 .../gatepass-server/src/pass-service.docs.md       |  12 +-
 .../gatepass-server/src/pass-service.openapi.json  | 211 +++++++++-
 .../commands/billing.email.SendEmail.schema.json   |   2 +-
 .../billing.invoice.CancelInvoice.schema.json      |   2 +-
 .../billing.invoice.CreateInvoice.schema.json      |   2 +-
 .../billing.invoice.IssueInvoice.schema.json       |   2 +-
 .../billing.invoice.PayInvoice.schema.json         |   2 +-
 .../entities/billing.invoice.Account.schema.json   |   2 +-
 .../entities/billing.invoice.Invoice.schema.json   |   2 +-
 .../errors/billing.email.Undeliverable.schema.json |   2 +-
 .../billing.invoice.InvalidAmount.schema.json      |   2 +-
 ...illing.invoice.InvoiceStateConflict.schema.json |   2 +-
 .../billing.email.DeliveryEscalated.schema.json    |   2 +-
 .../events/billing.email.EmailSent.schema.json     |   2 +-
 .../billing.invoice.InvoiceCancelled.schema.json   |   2 +-
 .../billing.invoice.InvoiceCreated.schema.json     |   2 +-
 .../billing.invoice.InvoiceIssued.schema.json      |   2 +-
 .../events/billing.invoice.InvoicePaid.schema.json |   2 +-
 .../types/billing.email.EmailAddress.schema.json   |   2 +-
 .../types/billing.email.MessageId.schema.json      |   2 +-
 .../types/billing.email.TemplateId.schema.json     |   2 +-
 .../billing.invoice.Account.State.schema.json      |   2 +-
 .../types/billing.invoice.AccountId.schema.json    |   2 +-
 .../types/billing.invoice.Channel.schema.json      |   2 +-
 .../types/billing.invoice.CompanyRef.schema.json   |   2 +-
 .../schema/types/billing.invoice.Email.schema.json |   2 +-
 .../billing.invoice.Invoice.State.schema.json      |   2 +-
 .../types/billing.invoice.InvoiceId.schema.json    |   2 +-
 .../types/billing.invoice.LineItem.schema.json     |   2 +-
 .../schema/types/billing.invoice.Money.schema.json |   2 +-
 .../schema/types/billing.invoice.Payee.schema.json |   2 +-
 generated/site/domains/billing-email.html          |   4 +-
 generated/site/domains/billing-invoice.html        |   4 +-
 63 files changed, 2234 insertions(+), 223 deletions(-)
```

Complete status (git status --short --untracked-files=all):

```text
 M crates/generate/ess-gen/src/provenance.rs
 M crates/generate/ess-gen/tests/corpus/billing/docs/domains/billing-email.md
 M crates/generate/ess-gen/tests/corpus/billing/docs/domains/billing-invoice.md
 M crates/generate/ess-gen/tests/corpus/gatepass/docs/domains/gatepass-visit.md
 M crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/domains/oracle-dispatch.md
 M crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/domains/oracle-order.md
 M crates/generate/ess-gen/tests/openapi.rs
 M crates/generate/ess-gen/tests/provenance.rs
 M crates/generate/ess-synth/tests/http.rs
 M crates/specify/ess-compiler/src/graph.rs
 M crates/verify/ess-diff/src/change.rs
 M crates/verify/ess-diff/src/delta.rs
 M crates/verify/ess-diff/src/diff.rs
 M crates/verify/ess-diff/src/impact.rs
 M crates/verify/ess-diff/src/lib.rs
 M crates/verify/ess-diff/src/raw.rs
 M crates/verify/ess-diff/tests/artifacts.rs
 M crates/verify/ess-diff/tests/canonical.rs
 M crates/verify/ess-diff/tests/families.rs
 M crates/verify/ess-diff/tests/graph.rs
 M crates/verify/ess-diff/tests/impact.rs
 M docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md
 M generated/asyncapi/email-service.yaml
 M generated/asyncapi/invoice-service.yaml
 M generated/docs/domains/billing-email.md
 M generated/docs/domains/billing-invoice.md
 M generated/go/gatepass/server/pass-service.docs.md
 M generated/go/gatepass/server/pass-service.openapi.json
 M generated/openapi/email-service.yaml
 M generated/openapi/invoice-service.yaml
 M generated/rust/gatepass/crates/gatepass-server/src/pass-service.docs.md
 M generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json
 M generated/schema/commands/billing.email.SendEmail.schema.json
 M generated/schema/commands/billing.invoice.CancelInvoice.schema.json
 M generated/schema/commands/billing.invoice.CreateInvoice.schema.json
 M generated/schema/commands/billing.invoice.IssueInvoice.schema.json
 M generated/schema/commands/billing.invoice.PayInvoice.schema.json
 M generated/schema/entities/billing.invoice.Account.schema.json
 M generated/schema/entities/billing.invoice.Invoice.schema.json
 M generated/schema/errors/billing.email.Undeliverable.schema.json
 M generated/schema/errors/billing.invoice.InvalidAmount.schema.json
 M generated/schema/errors/billing.invoice.InvoiceStateConflict.schema.json
 M generated/schema/events/billing.email.DeliveryEscalated.schema.json
 M generated/schema/events/billing.email.EmailSent.schema.json
 M generated/schema/events/billing.invoice.InvoiceCancelled.schema.json
 M generated/schema/events/billing.invoice.InvoiceCreated.schema.json
 M generated/schema/events/billing.invoice.InvoiceIssued.schema.json
 M generated/schema/events/billing.invoice.InvoicePaid.schema.json
 M generated/schema/types/billing.email.EmailAddress.schema.json
 M generated/schema/types/billing.email.MessageId.schema.json
 M generated/schema/types/billing.email.TemplateId.schema.json
 M generated/schema/types/billing.invoice.Account.State.schema.json
 M generated/schema/types/billing.invoice.AccountId.schema.json
 M generated/schema/types/billing.invoice.Channel.schema.json
 M generated/schema/types/billing.invoice.CompanyRef.schema.json
 M generated/schema/types/billing.invoice.Email.schema.json
 M generated/schema/types/billing.invoice.Invoice.State.schema.json
 M generated/schema/types/billing.invoice.InvoiceId.schema.json
 M generated/schema/types/billing.invoice.LineItem.schema.json
 M generated/schema/types/billing.invoice.Money.schema.json
 M generated/schema/types/billing.invoice.Payee.schema.json
 M generated/site/domains/billing-email.html
 M generated/site/domains/billing-invoice.html
?? crates/generate/ess-gen/src/stamp.rs
?? crates/verify/ess-diff/tests/fixtures/legacy-delta-v1.json
```

The tracked statistic excludes the two new files: crates/generate/ess-gen/src/stamp.rs and crates/verify/ess-diff/tests/fixtures/legacy-delta-v1.json. No other new source/generated files or deletions. git diff --check exited 0 at handoff.

## 3. Red evidence

Initial meaningful red cases: 15 = eight omitted-family cases, one graph case, one default-format case, four stamp cases and one independent transition-order residual case. Compiler preparation failures executed zero cases and are excluded. The legacy fixture capture test passed while the default-format case failed; it is not counted red. Expanded mechanism checks later executed 23 cases, and the later residual case makes 24 added cases in the final full suite.

Preparation mistakes are retained rather than counted as defect reproductions: PayloadTable.clear, a graph API call, RawEntity.lifecycle, and later QualifiedName.as_str did not compile. The second named logs contain their corrected, executed reproductions. test-preparation-compile.log preserves a duplicate of red-families.log.

### red-families.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families review_
```

Exit: 101. Full preserved output:

```text
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling hashbrown v0.17.1
   Compiling indexmap v2.14.1
   Compiling serde_yaml v0.9.34+deprecated
   Compiling schemars v0.8.22
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-primitives)
   Compiling ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-domain)
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
error[E0599]: no method named `clear` found for struct `PayloadTable` in the current scope
    --> crates/verify/ess-diff/tests/families.rs:1839:22
     |
1839 |         outcome.sets.clear();
     |                      ^^^^^ method not found in `PayloadTable`
     |
help: one of the expressions' fields has a method of the same name
     |
1839 |         outcome.sets.0.clear();
     |                      ++

For more information about this error, try `rustc --explain E0599`.
error: could not compile `ess-diff` (test "families") due to 1 previous error

```

### red-families-2.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families review_
```

Exit: 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 0.43s
     Running tests/families.rs (target/debug/deps/families-18b80718065c44da)

running 8 tests
test review_reach_is_a_change_without_an_unrelated_surface_edit ... FAILED
test review_outcome_refusal_is_independent_of_its_error ... FAILED
test review_cli_top_level_grouped_views_and_binary_are_changes ... FAILED
test review_view_parameter_naming_is_compared_without_a_filter_edit ... FAILED
test review_residual_refs_cannot_hide_beside_a_classified_change ... FAILED
test review_outcome_sets_are_independent_of_event_payload ... FAILED
test review_view_ranking_is_compared_without_a_filter_edit ... FAILED
test review_relation_cardinality_name_and_removal_are_changes ... FAILED

failures:

---- review_reach_is_a_change_without_an_unrelated_surface_edit stdout ----

thread 'review_reach_is_a_change_without_an_unrelated_surface_edit' (969910) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing reach-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "144cade022f3fe01f371e43000cec4e953e1328b8ad01ecd88bafcbfd300eb47"
  },
  "after": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "4721682490cf770e9dc1b6c786487e6413f4923119709dbdb77bf54092191787"
  },
  "changes": []
}

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- review_outcome_refusal_is_independent_of_its_error stdout ----

thread 'review_outcome_refusal_is_independent_of_its_error' (969908) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing outcome-refuses-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "144cade022f3fe01f371e43000cec4e953e1328b8ad01ecd88bafcbfd300eb47"
  },
  "after": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "9c09b82f2d30c8a00e1823a7b13d46f736a305a42a8eab9ecd24a825ce15d086"
  },
  "changes": [
    {
      "id": "command/witness.orders.CloseOrder/outcome-error-changed/wrong-state",
      "relation": "changed",
      "change": {
        "category": "command",
        "subject": "witness.orders.CloseOrder",
        "changed": {
          "kind": "outcome-error-changed",
          "outcome": "wrong-state",
          "before": "witness.orders.OrderStateConflict",
          "after": null
        }
      }
    }
  ]
}


---- review_cli_top_level_grouped_views_and_binary_are_changes stdout ----

thread 'review_cli_top_level_grouped_views_and_binary_are_changes' (969907) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing cli-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "58f86b695b5e721e585b7c4478feec5f2e779284ed74a5642251cdd0908d379b"
  },
  "after": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "6aa2ef1202e15d5b2571e11d12e444f2a37ea98d2174d8a1f291779366551e26"
  },
  "changes": []
}


---- review_view_parameter_naming_is_compared_without_a_filter_edit stdout ----

thread 'review_view_parameter_naming_is_compared_without_a_filter_edit' (969913) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing params-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "963c91423f6e1e6e88e98c3d1bb346fcb8acf9a910cb57ae2bacc7b2362bdd69"
  },
  "after": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "d8f94f62c6b34f61ee4542b3e653cd04475066d710c33fcc2caeaa942cf1c33d"
  },
  "changes": []
}


---- review_residual_refs_cannot_hide_beside_a_classified_change stdout ----

thread 'review_residual_refs_cannot_hide_beside_a_classified_change' (969912) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing unclassified-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "144cade022f3fe01f371e43000cec4e953e1328b8ad01ecd88bafcbfd300eb47"
  },
  "after": {
    "system": "witness",
    "specification_version": 1,
    "spec_digest": "9b34c5ecb1635589085bfd01fa6b51bc4287fdb7a9c5bcfd3dbb1d367072325b"
  },
  "changes": []
}


---- review_outcome_sets_are_independent_of_event_payload stdout ----

thread 'review_outcome_sets_are_independent_of_event_payload' (969909) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing outcome-sets-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942"
  },
  "after": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "94814261251815f1e7f37d37f7fd4ca983e36aa2441be7d1904c01312fd28621"
  },
  "changes": []
}


---- review_view_ranking_is_compared_without_a_filter_edit stdout ----

thread 'review_view_ranking_is_compared_without_a_filter_edit' (969914) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing ranking-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942"
  },
  "after": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "60d09e7d1c7d64f1628095eff2246e870823efd9e092f3b4d71474e0ed33e7cd"
  },
  "changes": []
}


---- review_relation_cardinality_name_and_removal_are_changes stdout ----

thread 'review_relation_cardinality_name_and_removal_are_changes' (969911) panicked at crates/verify/ess-diff/tests/families.rs:1785:5:
missing relations-changed: {
  "format": "ess-diff/1",
  "before": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942"
  },
  "after": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "7732d4c5b455c84e6d2afa862943c8dcfa32dfdc00fd070cfd718a3738719e96"
  },
  "changes": []
}



failures:
    review_cli_top_level_grouped_views_and_binary_are_changes
    review_outcome_refusal_is_independent_of_its_error
    review_outcome_sets_are_independent_of_event_payload
    review_reach_is_a_change_without_an_unrelated_surface_edit
    review_relation_cardinality_name_and_removal_are_changes
    review_residual_refs_cannot_hide_beside_a_classified_change
    review_view_parameter_naming_is_compared_without_a_filter_edit
    review_view_ranking_is_compared_without_a_filter_edit

test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.16s

error: test failed, to rerun pass `-p ess-diff --test families`

```

### red-graph-format.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test graph --test canonical review_ -- --nocapture
```

Exit: 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
error[E0599]: no method named `contains` found for struct `BTreeMap<K, V, A>` in the current scope
   --> crates/verify/ess-diff/tests/graph.rs:205:67
    |
205 |     assert!(graph.slice(&[invoice.clone()].into_iter().collect()).contains(&account));
    |                                                                   ^^^^^^^^
    |
help: there is a method `contains_key` with a similar name
    |
205 |     assert!(graph.slice(&[invoice.clone()].into_iter().collect()).contains_key(&account));
    |                                                                           ++++

error[E0599]: no method named `contains_key` found for struct `ess_compiler::Reach` in the current scope
   --> crates/verify/ess-diff/tests/graph.rs:207:37
    |
207 |     assert!(union.closure(&account).contains_key(&invoice), "removed relation still affects its old carrier");
    |                                     ^^^^^^^^^^^^ method not found in `ess_compiler::Reach`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `ess-diff` (test "graph") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...

```

### red-graph-format-2.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test graph --test canonical review_ --no-fail-fast -- --nocapture
```

Exit: 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 0.29s
     Running tests/canonical.rs (target/debug/deps/canonical-62cb9a27f38bef2f)

running 2 tests

thread 'review_new_default_delta_format_is_version_two' (981724) panicked at crates/verify/ess-diff/tests/canonical.rs:1143:5:
assertion `left == right` failed
  left: "ess-diff/1"
 right: "ess-diff/2"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test review_new_default_delta_format_is_version_two ... FAILED
LEGACY_BEGIN
{
  "format": "ess-diff/1",
  "before": {
    "system": "catalog",
    "specification_version": 2,
    "spec_digest": "9aa886fb68a2447af40c92cf53ed260af0d102507ac87e73a8e31fb7d20a0916"
  },
  "after": {
    "system": "catalog",
    "specification_version": 2,
    "spec_digest": "2dcf59ba04dd2fb953218bf8c60146d4efd4fca8282af8cd53c2063f4f4616be"
  },
  "changes": [
    {
      "id": "type/catalog.pricing.Currency/variant-added/CHF",
      "relation": "expanded",
      "change": {
        "category": "type",
        "subject": "catalog.pricing.Currency",
        "changed": {
          "kind": "variant-added",
          "variant": "CHF"
        }
      }
    },
    {
      "id": "type/catalog.pricing.Currency/variant-removed/GBP",
      "relation": "narrowed",
      "change": {
        "category": "type",
        "subject": "catalog.pricing.Currency",
        "changed": {
          "kind": "variant-removed",
          "variant": "GBP"
        }
      }
    },
    {
      "id": "entity/catalog.pricing.PriceList/invariants-changed",
      "relation": "changed",
      "change": {
        "category": "entity",
        "subject": "catalog.pricing.PriceList",
        "changed": {
          "kind": "invariants-changed",
          "before": [
            "floor.amount >= 0"
          ],
          "after": [
            "floor.amount > 0"
          ]
        }
      }
    },
    {
      "id": "command/catalog.pricing.CreatePriceList/outcome-condition-changed/created",
      "relation": "changed",
      "change": {
        "category": "command",
        "subject": "catalog.pricing.CreatePriceList",
        "changed": {
          "kind": "outcome-condition-changed",
          "outcome": "created",
          "before": "when floor.amount > 0",
          "after": "when floor.amount >= 1"
        }
      }
    },
    {
      "id": "actor/catalog.pricing.Auditor/grant-removed/catalog.pricing.RetirePriceList",
      "relation": "narrowed",
      "change": {
        "category": "actor",
        "subject": "catalog.pricing.Auditor",
        "changed": {
          "kind": "grant-removed",
          "command": "catalog.pricing.RetirePriceList"
        }
      }
    },
    {
      "id": "actor/catalog.pricing.PricingManager/grant-added/catalog.pricing.RetirePriceList",
      "relation": "expanded",
      "change": {
        "category": "actor",
        "subject": "catalog.pricing.PricingManager",
        "changed": {
          "kind": "grant-added",
          "command": "catalog.pricing.RetirePriceList"
        }
      }
    }
  ]
}
LEGACY_END
test review_freeze_legacy_delta_bytes ... ok

failures:

failures:
    review_new_default_delta_format_is_version_two

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-diff --test canonical`
     Running tests/graph.rs (target/debug/deps/graph-428d50882447a7e7)

running 1 test

thread 'review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union' (981726) panicked at crates/verify/ess-diff/tests/graph.rs:203:5:
assertion failed: edges.contains(&(account.clone(), serde_json::json!("relation-target"),
            invoice.clone()))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... FAILED

failures:

failures:
    review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-diff --test graph`
error: 2 targets failed:
    `-p ess-diff --test canonical`
    `-p ess-diff --test graph`

```

### red-provenance.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-gen --test provenance review_
```

Exit: 101. Full preserved output:

```text
   Compiling getrandom v0.3.4
   Compiling ahash v0.8.12
   Compiling jsonschema-value v0.52.1
   Compiling referencing v0.52.1
   Compiling jsonschema v0.52.1
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 4.70s
     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 4 tests
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... FAILED
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... FAILED
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... FAILED
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... FAILED

failures:

---- review_new_reader_requires_envelopes_and_exact_digest_tokens stdout ----

thread 'review_new_reader_requires_envelopes_and_exact_digest_tokens' (980111) panicked at crates/generate/ess-gen/tests/provenance.rs:309:5:
unframed prose is not provenance
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- review_new_reader_refuses_unsupported_profile_without_legacy_fallback stdout ----

thread 'review_new_reader_refuses_unsupported_profile_without_legacy_fallback' (980110) panicked at crates/generate/ess-gen/tests/provenance.rs:302:5:
an unsupported authoritative header cannot fall back to model prose

---- review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare stdout ----

thread 'review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare' (980109) panicked at crates/generate/ess-gen/tests/provenance.rs:289:5:
a3d6fd96a879d0d2eb21900933e07fd8a8ac23d72f99135179956e8aac921f3a

---- review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices stdout ----

thread 'review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices' (980112) panicked at crates/generate/ess-gen/tests/provenance.rs:326:13:
docs/domains/probe-core.md


failures:
    review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare
    review_new_reader_refuses_unsupported_profile_without_legacy_fallback
    review_new_reader_requires_envelopes_and_exact_digest_tokens
    review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices

test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-gen --test provenance`

```

### red-residual-order.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families review_unclassified_transition_order
```

Exit: 101. Full preserved output:

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
error[E0609]: no field `lifecycle` on type `&&mut RawEntitySpec`
    --> crates/verify/ess-diff/tests/families.rs:1961:39
     |
1961 |                 .find(|entity| entity.lifecycle.transitions.len() > 1).expect("billing declares multiple moves");
     |                                       ^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `identity`, `fields`, `relations`, `invariants` ... and 2 others

error[E0609]: no field `lifecycle` on type `&mut RawEntitySpec`
    --> crates/verify/ess-diff/tests/families.rs:1962:20
     |
1962 |             entity.lifecycle.transitions.swap(0, 1);
     |                    ^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `identity`, `fields`, `relations`, `invariants` ... and 2 others

For more information about this error, try `rustc --explain E0609`.
error: could not compile `ess-diff` (test "families") due to 2 previous errors

```

### red-residual-order-2.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families review_unclassified_transition_order
```

Exit: 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 1.48s
     Running tests/families.rs (target/debug/deps/families-18b80718065c44da)

running 1 test
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... FAILED

failures:

---- review_unclassified_transition_order_cannot_hide_beside_a_classified_edit stdout ----

thread 'review_unclassified_transition_order_cannot_hide_beside_a_classified_edit' (1166834) panicked at crates/verify/ess-diff/tests/families.rs:1792:5:
no new variant may leak into the legacy writer
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    review_unclassified_transition_order_cannot_hide_beside_a_classified_edit

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.17s

error: test failed, to rerun pass `-p ess-diff --test families`

```

## 4. Baseline and final gates

The identical unfiltered four-package command executed 459→483 cases. Every runner summary reports zero failed and zero ignored. The 24 new cases appear in the intended lanes: families +9, graph +2, canonical +3, artifacts +2, provenance +7, HTTP +1. Unchanged lanes received no new cases. Four empty doc-test lanes are reported as zero, not execution evidence. Isolated commands intentionally filtered existing lanes; their actual selected summaries remain in the raw outputs below.

Per-lane figures derived from the runner summaries of baseline.log and packages-final.log:

```text
ess-compiler::lib: executed 19 → 19, exit 0
ess-compiler::adversarial.rs: executed 2 → 2, exit 0
ess-compiler::billing.rs: executed 14 → 14, exit 0
ess-compiler::oracle_fixture.rs: executed 11 → 11, exit 0
ess-compiler::sealed_state.rs: executed 3 → 3, exit 0
ess-compiler::view_shapes.rs: executed 1 → 1, exit 0
ess-diff::lib: executed 9 → 9, exit 0
ess-diff::artifacts.rs: executed 11 → 13, exit 0
ess-diff::canonical.rs: executed 17 → 20, exit 0
ess-diff::families.rs: executed 60 → 69, exit 0
ess-diff::graph.rs: executed 6 → 8, exit 0
ess-diff::impact.rs: executed 15 → 15, exit 0
ess-diff::revision_pair.rs: executed 11 → 11, exit 0
ess-gen::lib: executed 55 → 55, exit 0
ess-gen::agreement.rs: executed 4 → 4, exit 0
ess-gen::asyncapi.rs: executed 18 → 18, exit 0
ess-gen::corpus.rs: executed 3 → 3, exit 0
ess-gen::determinism.rs: executed 2 → 2, exit 0
ess-gen::docs.rs: executed 32 → 32, exit 0
ess-gen::openapi.rs: executed 35 → 35, exit 0
ess-gen::provenance.rs: executed 9 → 16, exit 0
ess-gen::relations.rs: executed 4 → 4, exit 0
ess-gen::schema.rs: executed 27 → 27, exit 0
ess-synth::lib: executed 8 → 8, exit 0
ess-synth::clap.rs: executed 9 → 9, exit 0
ess-synth::go.rs: executed 19 → 19, exit 0
ess-synth::http.rs: executed 7 → 8, exit 0
ess-synth::relations.rs: executed 2 → 2, exit 0
ess-synth::synthesis.rs: executed 29 → 29, exit 0
ess-synth::web.rs: executed 17 → 17, exit 0
Doc-tests ess_compiler: executed 0 → 0, exit 0
Doc-tests ess_diff: executed 0 → 0, exit 0
Doc-tests ess_gen: executed 0 → 0, exit 0
Doc-tests ess_synth: executed 0 → 0, exit 0
```

### baseline.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth
```

Exit: 0. Full preserved output:

```text
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling cfg-if v1.0.4
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling itoa v1.0.18
   Compiling serde_json v1.0.151
   Compiling equivalent v1.0.2
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling typenum v1.20.1
   Compiling schemars v0.8.22
   Compiling thiserror v2.0.20
   Compiling dyn-clone v1.0.20
   Compiling const-oid v0.10.2
   Compiling unsafe-libyaml v0.2.11
   Compiling ryu v1.0.23
   Compiling cpufeatures v0.3.1
   Compiling bitflags v2.13.1
   Compiling pulldown-cmark v0.13.4
   Compiling libc v0.2.189
   Compiling unicase v2.9.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling autocfg v1.5.1
   Compiling hashbrown v0.17.1
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.56
   Compiling regex-syntax v0.8.11
   Compiling version_check v0.9.5
   Compiling parking_lot_core v0.9.12
   Compiling ref-cast v1.0.27
   Compiling scopeguard v1.2.0
   Compiling once_cell v1.21.4
   Compiling num-traits v0.2.19
   Compiling aho-corasick v1.1.5
   Compiling smallvec v1.16.0
   Compiling bit-vec v0.8.0
   Compiling ahash v0.8.12
   Compiling heck v0.5.0
   Compiling borrow-or-share v0.2.4
   Compiling unicode-general-category v1.1.0
   Compiling percent-encoding v2.3.2
   Compiling hybrid-array v0.4.14
   Compiling vsimd v0.8.0
   Compiling bytecount v0.6.9
   Compiling lock_api v0.4.14
   Compiling outref v0.5.2
   Compiling micromap v0.3.0
   Compiling indexmap v2.14.1
   Compiling num-cmp v0.1.0
   Compiling data-encoding v2.11.1
   Compiling bit-set v0.8.0
   Compiling unarray v0.1.4
   Compiling jsonschema-regex v0.52.1
   Compiling regex-automata v0.4.18
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling uuid-simd v0.8.0
   Compiling num-integer v0.1.47
   Compiling num-complex v0.4.6
   Compiling digest v0.11.3
   Compiling num-bigint v0.4.8
   Compiling num-iter v0.1.46
   Compiling parking_lot v0.12.5
   Compiling sha2 v0.11.0
   Compiling syn v3.0.4
   Compiling syn v2.0.119
   Compiling rand_core v0.9.5
   Compiling ppv-lite86 v0.2.21
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling num-rational v0.4.2
   Compiling num v0.4.3
   Compiling rand v0.9.5
   Compiling rand_xorshift v0.4.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling ref-cast-impl v1.0.27
   Compiling fraction v0.17.0
   Compiling serde_derive_internals v0.29.1
   Compiling strum_macros v0.28.0
   Compiling schemars_derive v0.8.22
   Compiling rand_chacha v0.9.0
   Compiling proptest v1.11.0
   Compiling strum v0.28.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling fluent-uri v0.4.1
   Compiling email_address v0.2.9
   Compiling jsonschema-value v0.52.1
   Compiling referencing v0.52.1
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-primitives)
   Compiling jsonschema v0.52.1
   Compiling ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-domain)
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 16.32s
     Running unittests src/lib.rs (target/debug/deps/ess_compiler-b4c0bc8f0757838a)

running 19 tests
test graph::tests::a_closure_walks_the_edges_backwards_and_not_forwards ... ok
test graph::tests::a_closure_keeps_the_edges_that_explain_each_construct_it_reached ... ok
test graph::tests::a_command_in_a_slice_brings_its_outcomes_and_what_they_name ... ok
test resolve::tests::a_code_the_bridge_has_no_class_for_still_gets_one ... ok
test graph::tests::a_slice_includes_its_seeds_each_with_no_path ... ok
test resolve::tests::a_needle_in_two_files_is_not_located_because_one_of_them_is_wrong ... ok
test graph::tests::merging_two_graphs_can_only_ever_reach_more ... ok
test graph::tests::the_construct_that_changed_is_in_its_own_closure_with_no_path ... ok
test resolve::tests::a_declaration_written_once_is_located_at_its_own_line_and_column ... ok
test resolve::tests::a_bridged_refusal_is_located_by_the_declaration_its_path_names ... ok
test graph::tests::a_slice_reaches_what_a_seed_rests_on_transitively ... ok
test resolve::tests::a_needle_that_occurs_twice_is_not_located_because_the_wrong_line_is_worse_than_none ... ok
test resolve::tests::a_refusal_from_the_domain_crate_keeps_the_code_the_compiler_would_have_given_it ... ok
test resolve::tests::a_refusal_is_filed_under_the_layer_its_document_path_names ... ok
test resolve::tests::every_code_renders_as_its_family_and_number ... ok
test resolve::tests::every_named_code_is_a_family_paired_with_a_class ... ok
test resolve::tests::the_register_lists_every_code_it_declares ... ok
test resolve::tests::the_second_needle_is_tried_when_the_first_is_ambiguous ... ok
test resolve::tests::with_no_files_named_a_span_still_carries_the_document_path ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial.rs (target/debug/deps/adversarial-b2f6b2147eadfc2d)

running 2 tests
test the_generator_reaches_both_compilation_and_refusal ... ok
test every_document_is_refused_with_reasons_or_compiled_identically_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/billing.rs (target/debug/deps/billing-127704819ded2aac)

running 14 tests
test a_refusal_from_the_whole_pipeline_carries_a_code_and_the_line_it_belongs_on ... ok
test no_source_file_in_the_compiler_reads_a_clock_or_an_unordered_map ... ok
test every_stable_reference_from_the_compiler_graph_resolves_against_its_ir ... ok
test the_billing_specification_resolves ... ok
test canonical_json_ends_in_a_newline ... ok
test a_binding_that_escalates_carries_the_event_it_emits_as_a_handle ... ok
test every_handle_in_the_ir_names_something_the_ir_holds ... ok
test a_field_keeps_the_shape_of_its_type_rather_than_a_rendering_of_it ... ok
test the_crossing_between_two_contexts_is_recorded_with_the_reason_someone_gave_for_it ... ok
test the_reaction_graph_names_the_binding_that_causes_each_command ... ok
test the_json_orders_its_keys_the_way_a_btreemap_does ... ok
test compiling_without_the_file_list_still_reports_the_document_path ... ok
test the_source_digest_names_exactly_the_canonical_semantic_model ... ok
test compiling_the_billing_example_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/oracle_fixture.rs (target/debug/deps/oracle_fixture-58359c9d77095e64)

running 11 tests
test dropping_one_binding_leaves_others_with_scenarios_of_their_own ... ok
test every_on_failure_policy_the_model_has_is_reachable_in_this_fixture ... ok
test an_outcome_updates_an_entity_without_moving_it_and_that_entity_declares_an_invariant ... ok
test an_illegal_transition_can_be_attempted_from_a_state_a_scenario_can_reach ... ok
test the_eventual_view_converges_on_a_state_the_creating_command_does_not_reach ... ok
test the_command_every_binding_invokes_can_be_forced_to_fail ... ok
test a_binding_maps_an_event_field_that_has_a_same_typed_sibling ... ok
test the_fixture_compiles_from_the_files_it_lives_in ... ok
test a_row_reaches_the_read_your_writes_view_after_a_single_command ... ok
test every_input_the_oracle_needs_is_carried_by_one_of_the_examples ... ok
test the_fixture_carries_something_the_normative_example_does_not ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/sealed_state.rs (target/debug/deps/sealed_state-fe1b4fe83ef9373a)

running 3 tests
test validated_and_resolved_state_have_no_public_fields ... ok
test provenance_never_hashes_an_empty_serialization_fallback ... ok
test every_compiler_entrance_validates_before_resolution ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/view_shapes.rs (target/debug/deps/view_shapes-6be70769d1254075)

running 1 test
test a_shape_is_one_handle_with_checked_fields_in_every_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-6c940e1b4fa9bdeb)

running 9 tests
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 11 tests
test a_change_to_the_system_header_owes_every_artifact ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 17 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_document_with_six_defects_reports_six ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 60 tests
test a_bindings_failure_policy_is_compared ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_command_added_is_one_change ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_binding_added_is_one_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test a_view_added_is_one_change ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_input_that_changed_type_is_reported ... ok
test an_outcomes_summary_is_compared ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test the_error_a_branch_reports_is_compared ... ok
test reordering_a_views_fields_is_reported_once ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test reordering_a_commands_input_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test reordering_an_entitys_fields_is_reported_once ... ok

test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 6 tests
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/impact.rs (target/debug/deps/impact-130b9b43fe12ce7e)

running 15 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test a_suite_for_another_system_is_refused ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-6384e25d4d698c13)

running 11 tests
test a_revision_compared_with_itself_reports_nothing ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok
test the_delta_survives_being_written_and_read_back ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_gen-9f7cfaccfa26d347)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-4af69d28643a528e)

running 4 tests
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-f449a9e3ff824c48)

running 18 tests
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/corpus.rs (target/debug/deps/corpus-b5c508be613e316e)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-82fd72ce337b58c9)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-9d17c089cbb09f3a)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test every_page_says_which_specification_produced_it ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/openapi.rs (target/debug/deps/openapi-84c5c21bb15c13da)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test a_command_is_only_ever_a_post ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_component_gets_one_document_named_after_it ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/provenance.rs (target/debug/deps/provenance-97bf54f63a94402c)

running 9 tests
test a_damaged_digest_reads_as_nothing ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test a_text_without_both_digests_reads_as_nothing ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/relations.rs (target/debug/deps/relations-9e1fbfdeb5e1e1b5)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/schema.rs (target/debug/deps/schema-c872c2e57cbce1cf)

running 27 tests
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_schema_says_which_specification_it_came_from ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-38a9c8cb806bed12)

running 8 tests
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 9 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test the_binary_generates_its_own_completions ... ok
test a_placed_view_becomes_a_verb ... ok
test every_placed_word_is_an_obligation ... ok
test a_string_field_offers_no_values ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test the_emission_is_deterministic ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/go.rs (target/debug/deps/go-408e408e0985c694)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test emitting_twice_is_byte_identical ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 7 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/relations.rs (target/debug/deps/relations-0da2273dd262509e)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f193ff4f35681316)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test grants_are_refused_rather_than_owed ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test the_billing_plan_counts_are_pinned ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test only_the_initial_state_can_be_constructed ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_plan_never_names_the_emission_language ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-1cb70fdbab20ef2a)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

   Doc-tests ess_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

### packages-final.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth
```

Exit: 0. Full preserved output:

```text
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 8.54s
     Running unittests src/lib.rs (target/debug/deps/ess_compiler-b4c0bc8f0757838a)

running 19 tests
test graph::tests::a_closure_walks_the_edges_backwards_and_not_forwards ... ok
test graph::tests::a_closure_keeps_the_edges_that_explain_each_construct_it_reached ... ok
test graph::tests::a_command_in_a_slice_brings_its_outcomes_and_what_they_name ... ok
test graph::tests::a_slice_includes_its_seeds_each_with_no_path ... ok
test graph::tests::a_slice_reaches_what_a_seed_rests_on_transitively ... ok
test graph::tests::the_construct_that_changed_is_in_its_own_closure_with_no_path ... ok
test graph::tests::merging_two_graphs_can_only_ever_reach_more ... ok
test resolve::tests::a_code_the_bridge_has_no_class_for_still_gets_one ... ok
test resolve::tests::a_bridged_refusal_is_located_by_the_declaration_its_path_names ... ok
test resolve::tests::a_declaration_written_once_is_located_at_its_own_line_and_column ... ok
test resolve::tests::a_needle_in_two_files_is_not_located_because_one_of_them_is_wrong ... ok
test resolve::tests::a_needle_that_occurs_twice_is_not_located_because_the_wrong_line_is_worse_than_none ... ok
test resolve::tests::a_refusal_from_the_domain_crate_keeps_the_code_the_compiler_would_have_given_it ... ok
test resolve::tests::a_refusal_is_filed_under_the_layer_its_document_path_names ... ok
test resolve::tests::every_code_renders_as_its_family_and_number ... ok
test resolve::tests::every_named_code_is_a_family_paired_with_a_class ... ok
test resolve::tests::with_no_files_named_a_span_still_carries_the_document_path ... ok
test resolve::tests::the_second_needle_is_tried_when_the_first_is_ambiguous ... ok
test resolve::tests::the_register_lists_every_code_it_declares ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial.rs (target/debug/deps/adversarial-b2f6b2147eadfc2d)

running 2 tests
test the_generator_reaches_both_compilation_and_refusal ... ok
test every_document_is_refused_with_reasons_or_compiled_identically_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/billing.rs (target/debug/deps/billing-127704819ded2aac)

running 14 tests
test a_refusal_from_the_whole_pipeline_carries_a_code_and_the_line_it_belongs_on ... ok
test no_source_file_in_the_compiler_reads_a_clock_or_an_unordered_map ... ok
test the_billing_specification_resolves ... ok
test every_handle_in_the_ir_names_something_the_ir_holds ... ok
test every_stable_reference_from_the_compiler_graph_resolves_against_its_ir ... ok
test the_json_orders_its_keys_the_way_a_btreemap_does ... ok
test a_binding_that_escalates_carries_the_event_it_emits_as_a_handle ... ok
test a_field_keeps_the_shape_of_its_type_rather_than_a_rendering_of_it ... ok
test the_crossing_between_two_contexts_is_recorded_with_the_reason_someone_gave_for_it ... ok
test the_reaction_graph_names_the_binding_that_causes_each_command ... ok
test canonical_json_ends_in_a_newline ... ok
test compiling_without_the_file_list_still_reports_the_document_path ... ok
test compiling_the_billing_example_twice_produces_byte_identical_json ... ok
test the_source_digest_names_exactly_the_canonical_semantic_model ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/oracle_fixture.rs (target/debug/deps/oracle_fixture-58359c9d77095e64)

running 11 tests
test an_outcome_updates_an_entity_without_moving_it_and_that_entity_declares_an_invariant ... ok
test dropping_one_binding_leaves_others_with_scenarios_of_their_own ... ok
test a_row_reaches_the_read_your_writes_view_after_a_single_command ... ok
test an_illegal_transition_can_be_attempted_from_a_state_a_scenario_can_reach ... ok
test every_on_failure_policy_the_model_has_is_reachable_in_this_fixture ... ok
test the_command_every_binding_invokes_can_be_forced_to_fail ... ok
test the_eventual_view_converges_on_a_state_the_creating_command_does_not_reach ... ok
test a_binding_maps_an_event_field_that_has_a_same_typed_sibling ... ok
test the_fixture_compiles_from_the_files_it_lives_in ... ok
test every_input_the_oracle_needs_is_carried_by_one_of_the_examples ... ok
test the_fixture_carries_something_the_normative_example_does_not ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/sealed_state.rs (target/debug/deps/sealed_state-fe1b4fe83ef9373a)

running 3 tests
test validated_and_resolved_state_have_no_public_fields ... ok
test provenance_never_hashes_an_empty_serialization_fallback ... ok
test every_compiler_entrance_validates_before_resolution ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/view_shapes.rs (target/debug/deps/view_shapes-6be70769d1254075)

running 1 test
test a_shape_is_one_handle_with_checked_fields_in_every_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-6c940e1b4fa9bdeb)

running 9 tests
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 13 tests
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test review_new_default_delta_format_is_version_two ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test a_document_with_six_defects_reports_six ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 69 tests
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_binding_added_is_one_change ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_command_added_is_one_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_view_added_is_one_change ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test a_views_naming_is_compared_key_by_key ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_a_views_fields_is_reported_once ... ok
test reordering_a_commands_input_is_reported_once ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test an_input_that_changed_type_is_reported ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_error_a_branch_reports_is_compared ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s

     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 8 tests
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-130b9b43fe12ce7e)

running 15 tests
test two_specifications_of_different_systems_are_refused_here_too ... ok
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test a_suite_for_another_system_is_refused ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-6384e25d4d698c13)

running 11 tests
test a_revision_compared_with_itself_reports_nothing ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test the_delta_survives_being_written_and_read_back ... ok
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_gen-9f7cfaccfa26d347)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-4af69d28643a528e)

running 4 tests
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-f449a9e3ff824c48)

running 18 tests
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/corpus.rs (target/debug/deps/corpus-b5c508be613e316e)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-82fd72ce337b58c9)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-9d17c089cbb09f3a)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test every_page_says_which_specification_produced_it ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/openapi.rs (target/debug/deps/openapi-84c5c21bb15c13da)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test a_command_is_only_ever_a_post ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test every_component_gets_one_document_named_after_it ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/provenance.rs (target/debug/deps/provenance-97bf54f63a94402c)

running 16 tests
test a_whole_model_slice_is_stamped_as_one ... ok
test a_damaged_digest_reads_as_nothing ... ok
test a_text_without_both_digests_reads_as_nothing ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... ok
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok
test review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp ... ok
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... ok
test review_conflicting_structured_and_comment_stamps_are_unreadable ... ok
test review_marker_looking_model_content_does_not_override_real_emitted_stamps ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/relations.rs (target/debug/deps/relations-9e1fbfdeb5e1e1b5)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/schema.rs (target/debug/deps/schema-c872c2e57cbce1cf)

running 27 tests
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_schema_says_which_specification_it_came_from ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-38a9c8cb806bed12)

running 8 tests
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 9 tests
test a_placed_view_becomes_a_verb ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test every_placed_word_is_an_obligation ... ok
test the_binary_generates_its_own_completions ... ok
test a_string_field_offers_no_values ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test the_emission_is_deterministic ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/go.rs (target/debug/deps/go-408e408e0985c694)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 8 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/relations.rs (target/debug/deps/relations-0da2273dd262509e)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f193ff4f35681316)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test grants_are_refused_rather_than_owed ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test only_the_initial_state_can_be_constructed ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test the_plan_never_names_the_emission_language ... ok
test the_billing_plan_counts_are_pinned ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-1cb70fdbab20ef2a)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests ess_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

### fmt-final.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt -p ess-compiler -p ess-diff -p ess-gen -p ess-synth -- --check
```

Exit: 0. Full preserved output:

```text

```

### clippy-4.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --all-targets -- -D warnings
```

Exit: 0. Full preserved output:

```text
    Checking ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `dev` profile [unoptimized] target(s) in 1.94s

```

## 5. Deliberate exclusions, compatibility and regeneration

No CLI/xtask/root manifest/lock/website/conformance/composition source changes, no AEP dependency, no new impact parser, no model version/property-bag expansion, no emitter redesign, no live executors or external service requests. The coordinator owns full task check, projection/site integration gates, governance and publication.

Explicit compatibility limits:

- source_digest and whole contract hashing remain unchanged. Frozen Billing source 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942 and whole contract cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c are checked. Gatepass source f2e0f8ff51c077fa1c713d8151544379bafac36a5a927e71c685042d53ab6e61 and whole contract e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e remain unchanged. Whole indexes and neutral HTTP PLAN.md/plan.json bytes compare unchanged. Suite v4 uses the unchanged whole identity; no new full-suite-v4 byte fixture or runtime was claimed.
- Sliced output bytes intentionally change. Frozen legacy v1 delta bytes remain accepted and writable; new delta vocabulary cannot be downgraded. Impact defaults to v3, with no hypothetical impact reader offered as evidence.
- Old read_digests code is preserved in tests. It refuses ordinary new sliced outputs, but marker-looking emitted model content demonstrates that its substring fallback can still fabricate a bare stamp. Prefixing cannot retroactively secure that old parser. Public Provenance String fields and generic Serde readers remain structural readers, not admission validators.
- ess-docs/1 retains its per-page nested SlicedProvenance shape. Generic Deserialize still accepts an unknown profile string, demonstrated in a test. read_digests returns None for the document-level request; it does not silently pick a page. Strict consumers need explicit profile validation when using the public string model.
- The parent reports Atlas ADR 0036 and relying inventory published at 974b2a2bc4896bd76293a734f36ac254895221c4. This is coordinator-provided governance state, not independently executed rollout evidence here. The old SDK baseline/catalog limitation is recorded there. The coordinator owns the pending old/new Service SDK complete-byte experiment and remaining pin/integration work. The SDK was found to have no diff/impact/stamp parser; its pinned generator compares complete bytes. No deployed consumer admission or readiness claim is made.

Measured regeneration: 46 retained file differences = 37 reserved Billing projections + five ess-gen corpus domain pages + four Gatepass HTTP payloads. Billing emitted 48 artifacts; Rust Gatepass 20 and Go Gatepass 15. Only two payloads per HTTP target differed, and unexpected HTTP differences were empty. All other emitted HTTP bytes, including plans, matched committed files. No new generated files or deletions. The first read-only measurement stopped at an absent site/assets/mermaid.LICENSE; source inspection confirmed xtask excludes site/assets constant assets. The corrected measurement used that existing exclusion; no files had been retained by the failed measurement.

Exact before/after hashes, byte counts and map source keys are preserved in regeneration-measurement.json. The full path inventory follows. Generator stdout maps are preserved as projections-billing.json, docs-gatepass.json, docs-oracle.json, http-rust.json and http-go.json; these raw JSON maps are not duplicated into this report. Corresponding command/stderr logs are appended below.

```text
projections-billing.json -> generated: artifacts 48, measured reserved changes 37, unexpected HTTP differences []
projections-billing.json -> crates/generate/ess-gen/tests/corpus/billing: artifacts 48, measured reserved changes 2, unexpected HTTP differences []
docs-gatepass.json -> crates/generate/ess-gen/tests/corpus/gatepass: artifacts 5, measured reserved changes 1, unexpected HTTP differences []
docs-oracle.json -> crates/generate/ess-gen/tests/corpus/oracle-fixture: artifacts 6, measured reserved changes 2, unexpected HTTP differences []
http-rust.json -> generated/rust/gatepass: artifacts 20, measured reserved changes 2, unexpected HTTP differences []
http-go.json -> generated/go/gatepass: artifacts 15, measured reserved changes 2, unexpected HTTP differences []
Total measured changes: 46
generated/asyncapi/email-service.yaml
generated/asyncapi/invoice-service.yaml
generated/docs/domains/billing-email.md
generated/docs/domains/billing-invoice.md
generated/openapi/email-service.yaml
generated/openapi/invoice-service.yaml
generated/schema/commands/billing.email.SendEmail.schema.json
generated/schema/commands/billing.invoice.CancelInvoice.schema.json
generated/schema/commands/billing.invoice.CreateInvoice.schema.json
generated/schema/commands/billing.invoice.IssueInvoice.schema.json
generated/schema/commands/billing.invoice.PayInvoice.schema.json
generated/schema/entities/billing.invoice.Account.schema.json
generated/schema/entities/billing.invoice.Invoice.schema.json
generated/schema/errors/billing.email.Undeliverable.schema.json
generated/schema/errors/billing.invoice.InvalidAmount.schema.json
generated/schema/errors/billing.invoice.InvoiceStateConflict.schema.json
generated/schema/events/billing.email.DeliveryEscalated.schema.json
generated/schema/events/billing.email.EmailSent.schema.json
generated/schema/events/billing.invoice.InvoiceCancelled.schema.json
generated/schema/events/billing.invoice.InvoiceCreated.schema.json
generated/schema/events/billing.invoice.InvoiceIssued.schema.json
generated/schema/events/billing.invoice.InvoicePaid.schema.json
generated/schema/types/billing.email.EmailAddress.schema.json
generated/schema/types/billing.email.MessageId.schema.json
generated/schema/types/billing.email.TemplateId.schema.json
generated/schema/types/billing.invoice.Account.State.schema.json
generated/schema/types/billing.invoice.AccountId.schema.json
generated/schema/types/billing.invoice.Channel.schema.json
generated/schema/types/billing.invoice.CompanyRef.schema.json
generated/schema/types/billing.invoice.Email.schema.json
generated/schema/types/billing.invoice.Invoice.State.schema.json
generated/schema/types/billing.invoice.InvoiceId.schema.json
generated/schema/types/billing.invoice.LineItem.schema.json
generated/schema/types/billing.invoice.Money.schema.json
generated/schema/types/billing.invoice.Payee.schema.json
generated/site/domains/billing-email.html
generated/site/domains/billing-invoice.html
crates/generate/ess-gen/tests/corpus/billing/docs/domains/billing-email.md
crates/generate/ess-gen/tests/corpus/billing/docs/domains/billing-invoice.md
crates/generate/ess-gen/tests/corpus/gatepass/docs/domains/gatepass-visit.md
crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/domains/oracle-dispatch.md
crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/domains/oracle-order.md
generated/rust/gatepass/crates/gatepass-server/src/pass-service.docs.md
generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json
generated/go/gatepass/server/pass-service.docs.md
generated/go/gatepass/server/pass-service.openapi.json
```

Assertion changes are limited to the approved contract transition: the unsupported delta-version test now uses /3 because /2 is supported; sliced stamps expect profile plus 64 lowercase hex; previously silent unclassified-family tests now require the explicit unclassified delta while retaining their whole-obligation assertion; measured corpus/generated bytes carry the new contract. No ignores, removed checks or relaxed error-summary assertion. The initial full run caught that error summary was left in residual despite an existing typed comparator; production normalization was fixed and the original assertion retained.

Intermediate results are preserved in full below: mechanism-1 exposed comment-envelope classification of ordinary CSS body comments; mechanism-2 passed after narrowing the actual generated-header boundary. packages-1 reported 473 passed/9 failed from expected format/snapshot updates plus the real error-summary residual defect. Clippy iterations required ordinary lint/refactoring corrections. None is substituted for the final gates.

Resource measurement: baseline available bytes 140170137600; handoff available bytes 125523296256; required reserve 8589934592. All Cargo children retained the prescribed cache wrapper/socket, compact profiles, offline/locked settings and own target. No CARGO_TARGET_DIR, cache lifecycle or target deletion. All own Cargo sessions have finished. Token/duration accounting is unavailable and is not invented.

## 6. Writes outside the assigned worktree

None. Source, tests, generated retention, temporary workflow scripts/logs/maps and this report stayed in the assigned tree; scratch/build is target and target/review-boundaries-4. The prescribed existing coordinator-owned sccache server was used through its socket; this agent did not operate its lifecycle or directly write sibling paths. No Git mutation, AEP/store write, staging, publication, worktree lifecycle operation or cleanup. No outside-scope patch is pending. Source writes are relinquished with this handoff.

## Appendix: complete intermediate command outputs

These records are retained to expose preparation failures and iterative corrections. Full raw logs remain adjacent to the report; command-results.json also records the command/exit mapping.

### mechanism-1.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families --test graph --test canonical review_ --no-fail-fast
```

Exit: 101. Full preserved output:

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 8.98s
     Running tests/canonical.rs (target/debug/deps/canonical-62cb9a27f38bef2f)

running 2 tests
test review_freeze_legacy_delta_bytes ... ok
test review_new_default_delta_format_is_version_two ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s

     Running tests/families.rs (target/debug/deps/families-18b80718065c44da)

running 8 tests
test review_residual_refs_cannot_hide_beside_a_classified_change ... FAILED
test review_reach_is_a_change_without_an_unrelated_surface_edit ... FAILED
test review_outcome_refusal_is_independent_of_its_error ... FAILED
test review_view_parameter_naming_is_compared_without_a_filter_edit ... FAILED
test review_cli_top_level_grouped_views_and_binary_are_changes ... FAILED
test review_relation_cardinality_name_and_removal_are_changes ... FAILED
test review_view_ranking_is_compared_without_a_filter_edit ... FAILED
test review_outcome_sets_are_independent_of_event_payload ... FAILED

failures:

---- review_residual_refs_cannot_hide_beside_a_classified_change stdout ----

thread 'review_residual_refs_cannot_hide_beside_a_classified_change' (1061056) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- review_reach_is_a_change_without_an_unrelated_surface_edit stdout ----

thread 'review_reach_is_a_change_without_an_unrelated_surface_edit' (1061054) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit

---- review_outcome_refusal_is_independent_of_its_error stdout ----

thread 'review_outcome_refusal_is_independent_of_its_error' (1061052) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit

---- review_view_parameter_naming_is_compared_without_a_filter_edit stdout ----

thread 'review_view_parameter_naming_is_compared_without_a_filter_edit' (1061057) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit

---- review_cli_top_level_grouped_views_and_binary_are_changes stdout ----

thread 'review_cli_top_level_grouped_views_and_binary_are_changes' (1061051) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit

---- review_relation_cardinality_name_and_removal_are_changes stdout ----

thread 'review_relation_cardinality_name_and_removal_are_changes' (1061055) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit

---- review_view_ranking_is_compared_without_a_filter_edit stdout ----

thread 'review_view_ranking_is_compared_without_a_filter_edit' (1061058) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit

---- review_outcome_sets_are_independent_of_event_payload stdout ----

thread 'review_outcome_sets_are_independent_of_event_payload' (1061053) panicked at crates/generate/ess-gen/src/artifact.rs:168:13:
the `site` generator wrote `assets/style.css` without readable provenance; an artifact that cannot say what it derives from is an artifact nobody can audit


failures:
    review_cli_top_level_grouped_views_and_binary_are_changes
    review_outcome_refusal_is_independent_of_its_error
    review_outcome_sets_are_independent_of_event_payload
    review_reach_is_a_change_without_an_unrelated_surface_edit
    review_relation_cardinality_name_and_removal_are_changes
    review_residual_refs_cannot_hide_beside_a_classified_change
    review_view_parameter_naming_is_compared_without_a_filter_edit
    review_view_ranking_is_compared_without_a_filter_edit

test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p ess-diff --test families`
     Running tests/graph.rs (target/debug/deps/graph-428d50882447a7e7)

running 1 test
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s

error: 1 target failed:
    `-p ess-diff --test families`

```

### mechanism-2.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families --test graph --test canonical review_ --no-fail-fast
```

Exit: 0. Full preserved output:

```text
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 8.31s
     Running tests/canonical.rs (target/debug/deps/canonical-62cb9a27f38bef2f)

running 2 tests
test review_freeze_legacy_delta_bytes ... ok
test review_new_default_delta_format_is_version_two ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s

     Running tests/families.rs (target/debug/deps/families-18b80718065c44da)

running 8 tests
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.52s

     Running tests/graph.rs (target/debug/deps/graph-428d50882447a7e7)

running 1 test
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s


```

### mechanism-3.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff -p ess-gen -p ess-synth review_ --no-fail-fast
```

Exit: 101. Full preserved output:

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
error[E0599]: no method named `as_str` found for struct `QualifiedName` in the current scope
   --> crates/verify/ess-diff/tests/graph.rs:218:30
    |
218 |                 if view.name.as_str() == "billing.invoice.InvoiceById" {
    |                              ^^^^^^ method not found in `QualifiedName`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `ess-diff` (test "graph") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

```

### mechanism-4.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff -p ess-gen -p ess-synth review_ --no-fail-fast
```

Exit: 0. Full preserved output:

```text
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 8.78s
     Running unittests src/lib.rs (target/debug/deps/ess_diff-6c940e1b4fa9bdeb)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 2 tests
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.22s

     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 3 tests
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test review_new_default_delta_format_is_version_two ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 8 tests
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.50s

     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 2 tests
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s

     Running tests/impact.rs (target/debug/deps/impact-130b9b43fe12ce7e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-6384e25d4d698c13)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_gen-5cfeec7d828080d8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-a6d7a7ff380699da)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-9e439fb4245702f8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.00s

     Running tests/corpus.rs (target/debug/deps/corpus-93718f8b1fc63993)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-9942695ed2e87dec)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.00s

     Running tests/openapi.rs (target/debug/deps/openapi-cbc5ba4392fca057)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s

     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 7 tests
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... ok
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... ok
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... ok
test review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp ... ok
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... ok
test review_conflicting_structured_and_comment_stamps_are_unreadable ... ok
test review_marker_looking_model_content_does_not_override_real_emitted_stamps ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.02s

     Running tests/relations.rs (target/debug/deps/relations-7cdc743b0b26371d)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/schema.rs (target/debug/deps/schema-e0945bde8d462715)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-38a9c8cb806bed12)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

     Running tests/go.rs (target/debug/deps/go-408e408e0985c694)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 1 test
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.04s

     Running tests/relations.rs (target/debug/deps/relations-0da2273dd262509e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f193ff4f35681316)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.00s

     Running tests/web.rs (target/debug/deps/web-1cb70fdbab20ef2a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.00s


```

### packages-1.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --no-fail-fast
```

Exit: 101. Full preserved output:

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 9.85s
     Running unittests src/lib.rs (target/debug/deps/ess_compiler-b4c0bc8f0757838a)

running 19 tests
test graph::tests::a_closure_keeps_the_edges_that_explain_each_construct_it_reached ... ok
test graph::tests::a_closure_walks_the_edges_backwards_and_not_forwards ... ok
test graph::tests::a_slice_includes_its_seeds_each_with_no_path ... ok
test graph::tests::a_command_in_a_slice_brings_its_outcomes_and_what_they_name ... ok
test graph::tests::a_slice_reaches_what_a_seed_rests_on_transitively ... ok
test graph::tests::merging_two_graphs_can_only_ever_reach_more ... ok
test resolve::tests::a_code_the_bridge_has_no_class_for_still_gets_one ... ok
test resolve::tests::a_bridged_refusal_is_located_by_the_declaration_its_path_names ... ok
test graph::tests::the_construct_that_changed_is_in_its_own_closure_with_no_path ... ok
test resolve::tests::a_declaration_written_once_is_located_at_its_own_line_and_column ... ok
test resolve::tests::a_needle_in_two_files_is_not_located_because_one_of_them_is_wrong ... ok
test resolve::tests::a_needle_that_occurs_twice_is_not_located_because_the_wrong_line_is_worse_than_none ... ok
test resolve::tests::a_refusal_from_the_domain_crate_keeps_the_code_the_compiler_would_have_given_it ... ok
test resolve::tests::a_refusal_is_filed_under_the_layer_its_document_path_names ... ok
test resolve::tests::every_code_renders_as_its_family_and_number ... ok
test resolve::tests::every_named_code_is_a_family_paired_with_a_class ... ok
test resolve::tests::the_register_lists_every_code_it_declares ... ok
test resolve::tests::the_second_needle_is_tried_when_the_first_is_ambiguous ... ok
test resolve::tests::with_no_files_named_a_span_still_carries_the_document_path ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial.rs (target/debug/deps/adversarial-b2f6b2147eadfc2d)

running 2 tests
test the_generator_reaches_both_compilation_and_refusal ... ok
test every_document_is_refused_with_reasons_or_compiled_identically_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/billing.rs (target/debug/deps/billing-127704819ded2aac)

running 14 tests
test a_refusal_from_the_whole_pipeline_carries_a_code_and_the_line_it_belongs_on ... ok
test no_source_file_in_the_compiler_reads_a_clock_or_an_unordered_map ... ok
test the_reaction_graph_names_the_binding_that_causes_each_command ... ok
test every_stable_reference_from_the_compiler_graph_resolves_against_its_ir ... ok
test canonical_json_ends_in_a_newline ... ok
test a_field_keeps_the_shape_of_its_type_rather_than_a_rendering_of_it ... ok
test a_binding_that_escalates_carries_the_event_it_emits_as_a_handle ... ok
test the_crossing_between_two_contexts_is_recorded_with_the_reason_someone_gave_for_it ... ok
test the_billing_specification_resolves ... ok
test every_handle_in_the_ir_names_something_the_ir_holds ... ok
test the_json_orders_its_keys_the_way_a_btreemap_does ... ok
test compiling_without_the_file_list_still_reports_the_document_path ... ok
test the_source_digest_names_exactly_the_canonical_semantic_model ... ok
test compiling_the_billing_example_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/oracle_fixture.rs (target/debug/deps/oracle_fixture-58359c9d77095e64)

running 11 tests
test an_outcome_updates_an_entity_without_moving_it_and_that_entity_declares_an_invariant ... ok
test a_row_reaches_the_read_your_writes_view_after_a_single_command ... ok
test an_illegal_transition_can_be_attempted_from_a_state_a_scenario_can_reach ... ok
test dropping_one_binding_leaves_others_with_scenarios_of_their_own ... ok
test the_command_every_binding_invokes_can_be_forced_to_fail ... ok
test a_binding_maps_an_event_field_that_has_a_same_typed_sibling ... ok
test every_on_failure_policy_the_model_has_is_reachable_in_this_fixture ... ok
test the_fixture_compiles_from_the_files_it_lives_in ... ok
test the_eventual_view_converges_on_a_state_the_creating_command_does_not_reach ... ok
test every_input_the_oracle_needs_is_carried_by_one_of_the_examples ... ok
test the_fixture_carries_something_the_normative_example_does_not ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/sealed_state.rs (target/debug/deps/sealed_state-fe1b4fe83ef9373a)

running 3 tests
test validated_and_resolved_state_have_no_public_fields ... ok
test provenance_never_hashes_an_empty_serialization_fallback ... ok
test every_compiler_entrance_validates_before_resolution ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/view_shapes.rs (target/debug/deps/view_shapes-6be70769d1254075)

running 1 test
test a_shape_is_one_handle_with_checked_fields_in_every_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-6c940e1b4fa9bdeb)

running 9 tests
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 13 tests
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... FAILED
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

failures:

---- a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim stdout ----

thread 'a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim' (1121825) panicked at crates/verify/ess-diff/tests/artifacts.rs:351:5:
assertion `left == right` failed
  left: 79
 right: 64
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim

test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

error: test failed, to rerun pass `-p ess-diff --test artifacts`
     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test review_freeze_legacy_delta_bytes ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test review_new_default_delta_format_is_version_two ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_document_with_six_defects_reports_six ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 68 tests
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_binding_added_is_one_change ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_command_added_is_one_change ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test a_view_added_is_one_change ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_input_that_changed_type_is_reported ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test reordering_a_views_fields_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test reordering_a_commands_input_is_reported_once ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test the_error_a_branch_reports_is_compared ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_error_tells_the_caller_is_compared ... FAILED
test writing_out_a_naming_default_is_not_a_change ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

failures:

---- what_an_error_tells_the_caller_is_compared stdout ----

thread 'what_an_error_tells_the_caller_is_compared' (1121924) panicked at crates/verify/ess-diff/tests/families.rs:255:5:
assertion `left == right` failed: one edit should be one change, and this produced 2:
  system/witness/unclassified-changed
  error/witness.orders.OrderStateConflict/summary-changed
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    what_an_error_tells_the_caller_is_compared

test result: FAILED. 67 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s

error: test failed, to rerun pass `-p ess-diff --test families`
     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 8 tests
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-130b9b43fe12ce7e)

running 15 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test a_suite_for_another_system_is_refused ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... FAILED
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... FAILED

failures:

---- a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain stdout ----

thread 'a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain' (1121973) panicked at crates/verify/ess-diff/tests/impact.rs:488:5:
the delta has no entry for a domain naming change: EssDelta { format: DeltaFormat(Version(2)), before: EssRevisionRef { system: QualifiedName(billing), specification_version: Version(3), spec_digest: SpecDigest("56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942") }, after: EssRevisionRef { system: QualifiedName(billing), specification_version: Version(3), spec_digest: SpecDigest("21b8feb2db2142a6df4f9f76a96463802c276beb484b759b6ba2f7067cba3b73") }, changes: [System { subject: QualifiedName(billing), changed: UnclassifiedChanged }] }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite stdout ----

thread 'a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite' (1121972) panicked at crates/verify/ess-diff/tests/impact.rs:446:5:
the delta has no entry for a topology change: EssDelta { format: DeltaFormat(Version(2)), before: EssRevisionRef { system: QualifiedName(billing), specification_version: Version(3), spec_digest: SpecDigest("56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942") }, after: EssRevisionRef { system: QualifiedName(billing), specification_version: Version(3), spec_digest: SpecDigest("44d59fe6b111dd8dfe99afbd9c75925ebc5ca3576fbb1ee3bbce87c8fda407d6") }, changes: [System { subject: QualifiedName(billing), changed: UnclassifiedChanged }] }


failures:
    a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite
    a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain

test result: FAILED. 13 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

error: test failed, to rerun pass `-p ess-diff --test impact`
     Running tests/revision_pair.rs (target/debug/deps/revision_pair-6384e25d4d698c13)

running 11 tests
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test the_delta_survives_being_written_and_read_back ... ok
test a_revision_compared_with_itself_reports_nothing ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_gen-9f7cfaccfa26d347)

running 55 tests
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-4af69d28643a528e)

running 4 tests
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-f449a9e3ff824c48)

running 18 tests
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok
test every_component_gets_one_document_named_after_it ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/corpus.rs (target/debug/deps/corpus-b5c508be613e316e)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... FAILED
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... FAILED
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... FAILED

failures:

---- the_gatepass_documentation_is_byte_for_byte_what_is_pinned stdout ----

thread 'the_gatepass_documentation_is_byte_for_byte_what_is_pinned' (1122081) panicked at crates/generate/ess-gen/tests/corpus.rs:112:9:
assertion `left == right` failed: `docs/domains/gatepass-visit.md` of `gatepass` is not what is pinned. This is what every adopter's committed pages would gain in `git diff`; if the change is deliberate, regenerate the corpus in the commit that meant to make it and say why in the message
  left: "<!--\ngenerated from gatepass v1\nmodel digest f2e0f8ff51c077fa1c713d8151544379bafac36a5a927e71c685042d53ab6e61\ncontract digest slice-sha256/2:e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e\ndo not edit: regenerate with `ess generate`\n-->\n\n# Visits\n\nExpecting a visitor, letting them in, and letting them out again.\n\n`gatepass.visit` is one of gatepass's bounded contexts. [Back to the index](../index.md).\n\n## Types\n\n### `Badge`\n\n`gatepass.visit.Badge` is a record of three fields:\n\n- `serial` — `String`\n- `printed_at` — `Optional<Timestamp>`, which may be absent\n- `signature` — `Bytes`\n\n### `Building`\n\n`gatepass.visit.Building` is one of `North`, `South` and `Annex`.\n\nShown to a person as \"Building\".\n\n### `Deposit`\n\n`gatepass.visit.Deposit` is a record of two fields:\n\n- `amount` — `Decimal`\n- `currency` — `String`\n\nEvery value satisfies `amount >= 0`.\n\n### `EmployeeId`\n\n`gatepass.visit.EmployeeId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `Host`\n\n`gatepass.visit.Host` is one of two shapes, told apart by a `kind` field — tagged, so a decoder never has to guess which branch it is reading:\n\n- `contractor` — `gatepass.visit.VendorRef`\n- `employee` — `gatepass.visit.EmployeeId`\n\n### `VendorRef`\n\n`gatepass.visit.VendorRef` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `VisitId`\n\n`gatepass.visit.VisitId` wraps `Uuid` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `VisitorName`\n\n`gatepass.visit.VisitorName` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n## Entities\n\nAn entity is what this context is about: something with an identity that outlives any one request, a shape, and a lifecycle. The lifecycle is exhaustive — a move that is not drawn below is a move this specification does not permit, and that is the only way it says so. Every move is labelled with the command that takes it, because a move nothing can trigger is refused rather than drawn.\n\n### `Visit`\n\n`gatepass.visit.Visit`.\n\nAn instance is identified by `visit_id`, a `gatepass.visit.VisitId`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.\n\nIt holds:\n\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n- `host` — `gatepass.visit.Host`\n- `expected_minutes` — `Integer`\n- `expected_stay` — `Duration`\n- `deposit` — `gatepass.visit.Deposit`\n- `escorts` — `List<gatepass.visit.VisitorName>`\n- `notes` — `Map<String, String>`\n- `badge` — `Optional<gatepass.visit.Badge>`, which may be absent\n- `on_watchlist` — `Boolean`\n\nIt declares no relation to another entity, and no other entity names it.\n\nEvery instance satisfies `deposit.amount >= 0` and `expected_minutes > 0` — a predicate over this entity's own fields, checked against them rather than stored as a sentence, so an invariant reading something the entity does not have is refused instead of documented.\n\nIts state is a `gatepass.visit.Visit.State`, one of `Departed`, `Expected` and `OnSite`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.\n\nAn instance is created in `Expected`. `Departed` is terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.\n\n```mermaid\nstateDiagram-v2\n    [*] --> Expected\n    Expected --> OnSite: arrive (AdmitVisitor)\n    OnSite --> Departed: depart (SignOutVisitor)\n    Departed --> [*]\n```\n\nEach move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:\n\n- `arrive` — taken by `gatepass.visit.AdmitVisitor` on its `admitted` outcome\n- `depart` — taken by `gatepass.visit.SignOutVisitor` on its `signed-out` outcome\n\nAn instance is brought into existence by `gatepass.visit.RegisterVisit` on its `registered` outcome.\n\nIllegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.\n\n- `Departed` may not become `Expected`\n- `Departed` may not become `OnSite`\n- `Expected` may not become `Departed`\n- `OnSite` may not become `Expected`\n\nTwo views project it: [`ExpectedVisits`](#expectedvisits) and [`VisitById`](#visitbyid).\n\n## Views\n\nA view is what the outside world is promised it can observe. Each one says which instances it contains and how soon it reflects a command that has already returned, because \"you can read this\" without \"how soon\" is the promise every flaky suite is built on.\n\n### `ExpectedVisits`\n\n`gatepass.visit.ExpectedVisits`, shown to a person as \"Expected visits\" and called `expected` on the wire.\n\nIt reads [`Visit`](#visit).\n\nIt contains the instances where `state == Expected` holds, and only those — so an instance a caller cannot find in here has been filtered out rather than lost.\n\nIt exposes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n- `deposit` — `gatepass.visit.Deposit`\n\nIt declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.\n\n**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.\n\nA generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.\n\n### `VisitById`\n\n`gatepass.visit.VisitById`, shown to a person as \"Visit by id\" and called `by-id` on the wire.\n\nIt reads [`Visit`](#visit).\n\nIt contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.\n\nIt exposes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `visitor` — `gatepass.visit.VisitorName`\n- `host` — `gatepass.visit.Host`\n- `escorts` — `List<gatepass.visit.VisitorName>`\n- `notes` — `Map<String, String>`\n- `badge` — `Optional<gatepass.visit.Badge>`, which may be absent\n\nIt declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.\n\n**Eventual**: it catches up some time after the command returns, so a caller that reads it immediately may legitimately not see its own write yet. Nothing here says how long that takes, so nothing here lets a caller wait a fixed time and call it correct.\n\nA generated scenario therefore retries the assertion until the projection catches up, rather than asserting once and racing it. The repair everyone reaches for instead is a sleep, which turns the suite into a test of the machine it runs on.\n\n## Commands\n\n### `AdmitVisitor`\n\n`gatepass.visit.AdmitVisitor`, shown to a person as \"Admit the visitor\" and called `admit-visitor` on the wire.\n\nIt takes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `badge` — `gatepass.visit.Badge`\n\nIt has two outcomes.\n\n**`admitted`** — The visitor is on site, holding the badge that was printed. The default branch, taken when no other outcome's condition matched. It moves a `gatepass.visit.Visit` from `Expected` to `OnSite`, along the declared move `arrive`. The instance is the one named by the input field `visit_id`. It emits `gatepass.visit.VisitorAdmitted`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`wrong-state`** — The visit is not Expected, so nobody was admitted. Taken when the subject is resting in a state none of this command's moves start from — a `gatepass.visit.Visit` in `Departed` and `OnSite`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `gatepass.visit.VisitStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.\n\n### `RegisterVisit`\n\n`gatepass.visit.RegisterVisit`, shown to a person as \"Register a visit\" and called `register-visit` on the wire.\n\nIt takes:\n\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n- `host` — `gatepass.visit.Host`\n- `expected_minutes` — `Integer`\n- `expected_stay` — `Duration`\n- `deposit` — `gatepass.visit.Deposit`\n- `escorts` — `List<gatepass.visit.VisitorName>`\n- `notes` — `Map<String, String>`\n- `on_watchlist` — `Boolean`\n\nIt has two outcomes.\n\n**`registered`** — The visit is recorded, and the visitor is Expected. Taken when `expected_minutes > 0` holds of the input. It creates a `gatepass.visit.Visit`, which starts in `Expected`. The new instance's identity is published as `visit_id` on `gatepass.visit.VisitRegistered`. It emits `gatepass.visit.VisitRegistered`. A test reaches it by constructing an input that satisfies that condition.\n\n**`refused`** — The expected length was not positive, and nothing was recorded. The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It reports `gatepass.visit.InvalidVisitLength`, carrying `submitted`. It emits nothing. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n### `SignOutVisitor`\n\n`gatepass.visit.SignOutVisitor`, shown to a person as \"Sign the visitor out\" and called `sign-out-visitor` on the wire.\n\nIt takes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n\nIt has two outcomes.\n\n**`signed-out`** — The visitor has left the building. The default branch, taken when no other outcome's condition matched. It moves a `gatepass.visit.Visit` from `OnSite` to `Departed`, along the declared move `depart`. The instance is the one named by the input field `visit_id`. It emits `gatepass.visit.VisitorDeparted`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`wrong-state`** — The visit is not OnSite, so nobody was signed out. Taken when the subject is resting in a state none of this command's moves start from — a `gatepass.visit.Visit` in `Departed` and `Expected`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `gatepass.visit.VisitStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.\n\n## Events\n\n### `VisitRegistered`\n\n`gatepass.visit.VisitRegistered`.\n\nIt carries:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n\nEmitted by `gatepass.visit.RegisterVisit` on its `registered` outcome.\n\nNothing in this system reacts to it.\n\n### `VisitorAdmitted`\n\n`gatepass.visit.VisitorAdmitted`.\n\nIt carries:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `badge` — `gatepass.visit.Badge`\n\nEmitted by `gatepass.visit.AdmitVisitor` on its `admitted` outcome.\n\nNothing in this system reacts to it.\n\n### `VisitorDeparted`\n\n`gatepass.visit.VisitorDeparted`.\n\nIt carries:\n\n- `visit_id` — `gatepass.visit.VisitId`\n\nEmitted by `gatepass.visit.SignOutVisitor` on its `signed-out` outcome.\n\nNothing in this system reacts to it.\n\n## Errors\n\n### `InvalidVisitLength`\n\nThe expected length of the visit is not a positive number of minutes.\n\nIt carries:\n\n- `submitted` — `Integer`\n\nReported by `gatepass.visit.RegisterVisit` on its `refused` outcome.\n\n### `VisitStateConflict`\n\nThe visit is not in a state this command acts from, so nothing moved.\n\nIt carries:\n\n- `state` — `gatepass.visit.Visit.State`\n\nReported by `gatepass.visit.AdmitVisitor` on its `wrong-state` outcome.\n\nReported by `gatepass.visit.SignOutVisitor` on its `wrong-state` outcome.\n\n## Actors\n\nAn actor is who may ask this context for something. Every grant below points at a command this specification declares — a grant is a resolved reference, so \"may invoke\" something nobody wrote is not a permission this model can express, and an authorisation that authorises nothing cannot ship quietly.\n\n### `Receptionist`\n\n`gatepass.visit.Receptionist`, shown to a person as \"Receptionist\".\n\nIt may invoke [`AdmitVisitor`](#admitvisitor), [`RegisterVisit`](#registervisit) and [`SignOutVisitor`](#signoutvisitor).\n\n### `SecurityAuditor`\n\n`gatepass.visit.SecurityAuditor`, shown to a person as \"Security auditor\".\n\nIt may invoke nothing: it observes. \"Who is in this picture\" is part of what a specification describes, so an actor with no grant is a statement rather than an unfinished line.\n\n\n---\n\nGenerated from gatepass v1 · model digest `f2e0f8ff51c077fa1c713d8151544379bafac36a5a927e71c685042d53ab6e61` · contract digest `slice-sha256/2:e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e`. Do not edit this file; change the specification and regenerate it with `ess generate`.\n"
 right: "<!--\ngenerated from gatepass v1\nmodel digest f2e0f8ff51c077fa1c713d8151544379bafac36a5a927e71c685042d53ab6e61\ncontract digest e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e\ndo not edit: regenerate with `ess generate`\n-->\n\n# Visits\n\nExpecting a visitor, letting them in, and letting them out again.\n\n`gatepass.visit` is one of gatepass's bounded contexts. [Back to the index](../index.md).\n\n## Types\n\n### `Badge`\n\n`gatepass.visit.Badge` is a record of three fields:\n\n- `serial` — `String`\n- `printed_at` — `Optional<Timestamp>`, which may be absent\n- `signature` — `Bytes`\n\n### `Building`\n\n`gatepass.visit.Building` is one of `North`, `South` and `Annex`.\n\nShown to a person as \"Building\".\n\n### `Deposit`\n\n`gatepass.visit.Deposit` is a record of two fields:\n\n- `amount` — `Decimal`\n- `currency` — `String`\n\nEvery value satisfies `amount >= 0`.\n\n### `EmployeeId`\n\n`gatepass.visit.EmployeeId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `Host`\n\n`gatepass.visit.Host` is one of two shapes, told apart by a `kind` field — tagged, so a decoder never has to guess which branch it is reading:\n\n- `contractor` — `gatepass.visit.VendorRef`\n- `employee` — `gatepass.visit.EmployeeId`\n\n### `VendorRef`\n\n`gatepass.visit.VendorRef` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `VisitId`\n\n`gatepass.visit.VisitId` wraps `Uuid` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `VisitorName`\n\n`gatepass.visit.VisitorName` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n## Entities\n\nAn entity is what this context is about: something with an identity that outlives any one request, a shape, and a lifecycle. The lifecycle is exhaustive — a move that is not drawn below is a move this specification does not permit, and that is the only way it says so. Every move is labelled with the command that takes it, because a move nothing can trigger is refused rather than drawn.\n\n### `Visit`\n\n`gatepass.visit.Visit`.\n\nAn instance is identified by `visit_id`, a `gatepass.visit.VisitId`. The name is part of the model and not a convention: a view projects the identity under that name, so a projection inventing its own would disagree with the view.\n\nIt holds:\n\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n- `host` — `gatepass.visit.Host`\n- `expected_minutes` — `Integer`\n- `expected_stay` — `Duration`\n- `deposit` — `gatepass.visit.Deposit`\n- `escorts` — `List<gatepass.visit.VisitorName>`\n- `notes` — `Map<String, String>`\n- `badge` — `Optional<gatepass.visit.Badge>`, which may be absent\n- `on_watchlist` — `Boolean`\n\nIt declares no relation to another entity, and no other entity names it.\n\nEvery instance satisfies `deposit.amount >= 0` and `expected_minutes > 0` — a predicate over this entity's own fields, checked against them rather than stored as a sentence, so an invariant reading something the entity does not have is refused instead of documented.\n\nIts state is a `gatepass.visit.Visit.State`, one of `Departed`, `Expected` and `OnSite`. That enum is synthesised from the lifecycle rather than declared beside it, so the states a view's filter compares and the states drawn below cannot disagree.\n\nAn instance is created in `Expected`. `Departed` is terminal, so an instance may rest there forever. That is declared rather than inferred from having no way out: an entity that cannot leave a state is either finished or stuck, and only its author knows which.\n\n```mermaid\nstateDiagram-v2\n    [*] --> Expected\n    Expected --> OnSite: arrive (AdmitVisitor)\n    OnSite --> Departed: depart (SignOutVisitor)\n    Departed --> [*]\n```\n\nEach move is taken by a declared command outcome, and a move nothing takes is refused as `missing_causation` rather than left as a state change nobody can trigger:\n\n- `arrive` — taken by `gatepass.visit.AdmitVisitor` on its `admitted` outcome\n- `depart` — taken by `gatepass.visit.SignOutVisitor` on its `signed-out` outcome\n\nAn instance is brought into existence by `gatepass.visit.RegisterVisit` on its `registered` outcome.\n\nIllegal transitions are illegal by absence: no rule forbids them, there is simply no arrow, because a rule would be a second place for the same truth to live. A diagram cannot show an absence, so the pairs it does not connect are listed here, derived from the same transitions — anything named below is a move this specification does not permit.\n\n- `Departed` may not become `Expected`\n- `Departed` may not become `OnSite`\n- `Expected` may not become `Departed`\n- `OnSite` may not become `Expected`\n\nTwo views project it: [`ExpectedVisits`](#expectedvisits) and [`VisitById`](#visitbyid).\n\n## Views\n\nA view is what the outside world is promised it can observe. Each one says which instances it contains and how soon it reflects a command that has already returned, because \"you can read this\" without \"how soon\" is the promise every flaky suite is built on.\n\n### `ExpectedVisits`\n\n`gatepass.visit.ExpectedVisits`, shown to a person as \"Expected visits\" and called `expected` on the wire.\n\nIt reads [`Visit`](#visit).\n\nIt contains the instances where `state == Expected` holds, and only those — so an instance a caller cannot find in here has been filtered out rather than lost.\n\nIt exposes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n- `deposit` — `gatepass.visit.Deposit`\n\nIt declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.\n\n**Read-your-writes**: it is current the moment the command that changed it returns. A caller that has just created an invoice and cannot see it in here has been told a lie about what it did.\n\nA generated scenario asserts it once, immediately after the command: a view promising this and not keeping the promise has to fail the suite rather than be retried until it passes.\n\n### `VisitById`\n\n`gatepass.visit.VisitById`, shown to a person as \"Visit by id\" and called `by-id` on the wire.\n\nIt reads [`Visit`](#visit).\n\nIt contains every instance of that entity: no filter narrows it, which is a decision somebody made and not a line somebody omitted.\n\nIt exposes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `visitor` — `gatepass.visit.VisitorName`\n- `host` — `gatepass.visit.Host`\n- `escorts` — `List<gatepass.visit.VisitorName>`\n- `notes` — `Map<String, String>`\n- `badge` — `Optional<gatepass.visit.Badge>`, which may be absent\n\nIt declares no order, so the rows come back in whatever order the implementation has, and two reads may disagree.\n\n**Eventual**: it catches up some time after the command returns, so a caller that reads it immediately may legitimately not see its own write yet. Nothing here says how long that takes, so nothing here lets a caller wait a fixed time and call it correct.\n\nA generated scenario therefore retries the assertion until the projection catches up, rather than asserting once and racing it. The repair everyone reaches for instead is a sleep, which turns the suite into a test of the machine it runs on.\n\n## Commands\n\n### `AdmitVisitor`\n\n`gatepass.visit.AdmitVisitor`, shown to a person as \"Admit the visitor\" and called `admit-visitor` on the wire.\n\nIt takes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `badge` — `gatepass.visit.Badge`\n\nIt has two outcomes.\n\n**`admitted`** — The visitor is on site, holding the badge that was printed. The default branch, taken when no other outcome's condition matched. It moves a `gatepass.visit.Visit` from `Expected` to `OnSite`, along the declared move `arrive`. The instance is the one named by the input field `visit_id`. It emits `gatepass.visit.VisitorAdmitted`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`wrong-state`** — The visit is not Expected, so nobody was admitted. Taken when the subject is resting in a state none of this command's moves start from — a `gatepass.visit.Visit` in `Departed` and `OnSite`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `gatepass.visit.VisitStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.\n\n### `RegisterVisit`\n\n`gatepass.visit.RegisterVisit`, shown to a person as \"Register a visit\" and called `register-visit` on the wire.\n\nIt takes:\n\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n- `host` — `gatepass.visit.Host`\n- `expected_minutes` — `Integer`\n- `expected_stay` — `Duration`\n- `deposit` — `gatepass.visit.Deposit`\n- `escorts` — `List<gatepass.visit.VisitorName>`\n- `notes` — `Map<String, String>`\n- `on_watchlist` — `Boolean`\n\nIt has two outcomes.\n\n**`registered`** — The visit is recorded, and the visitor is Expected. Taken when `expected_minutes > 0` holds of the input. It creates a `gatepass.visit.Visit`, which starts in `Expected`. The new instance's identity is published as `visit_id` on `gatepass.visit.VisitRegistered`. It emits `gatepass.visit.VisitRegistered`. A test reaches it by constructing an input that satisfies that condition.\n\n**`refused`** — The expected length was not positive, and nothing was recorded. The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It reports `gatepass.visit.InvalidVisitLength`, carrying `submitted`. It emits nothing. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n### `SignOutVisitor`\n\n`gatepass.visit.SignOutVisitor`, shown to a person as \"Sign the visitor out\" and called `sign-out-visitor` on the wire.\n\nIt takes:\n\n- `visit_id` — `gatepass.visit.VisitId`\n\nIt has two outcomes.\n\n**`signed-out`** — The visitor has left the building. The default branch, taken when no other outcome's condition matched. It moves a `gatepass.visit.Visit` from `OnSite` to `Departed`, along the declared move `depart`. The instance is the one named by the input field `visit_id`. It emits `gatepass.visit.VisitorDeparted`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`wrong-state`** — The visit is not OnSite, so nobody was signed out. Taken when the subject is resting in a state none of this command's moves start from — a `gatepass.visit.Visit` in `Departed` and `Expected`, which is what is left of the lifecycle once this command's own moves are taken away. The document lists none of it. No entity in this specification changes. It reports `gatepass.visit.VisitStateConflict`, carrying `state`. It emits nothing. A test reaches it by driving an instance into one of those states and then issuing the command, because no input selects this branch.\n\n## Events\n\n### `VisitRegistered`\n\n`gatepass.visit.VisitRegistered`.\n\nIt carries:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `visitor` — `gatepass.visit.VisitorName`\n- `building` — `gatepass.visit.Building`\n\nEmitted by `gatepass.visit.RegisterVisit` on its `registered` outcome.\n\nNothing in this system reacts to it.\n\n### `VisitorAdmitted`\n\n`gatepass.visit.VisitorAdmitted`.\n\nIt carries:\n\n- `visit_id` — `gatepass.visit.VisitId`\n- `badge` — `gatepass.visit.Badge`\n\nEmitted by `gatepass.visit.AdmitVisitor` on its `admitted` outcome.\n\nNothing in this system reacts to it.\n\n### `VisitorDeparted`\n\n`gatepass.visit.VisitorDeparted`.\n\nIt carries:\n\n- `visit_id` — `gatepass.visit.VisitId`\n\nEmitted by `gatepass.visit.SignOutVisitor` on its `signed-out` outcome.\n\nNothing in this system reacts to it.\n\n## Errors\n\n### `InvalidVisitLength`\n\nThe expected length of the visit is not a positive number of minutes.\n\nIt carries:\n\n- `submitted` — `Integer`\n\nReported by `gatepass.visit.RegisterVisit` on its `refused` outcome.\n\n### `VisitStateConflict`\n\nThe visit is not in a state this command acts from, so nothing moved.\n\nIt carries:\n\n- `state` — `gatepass.visit.Visit.State`\n\nReported by `gatepass.visit.AdmitVisitor` on its `wrong-state` outcome.\n\nReported by `gatepass.visit.SignOutVisitor` on its `wrong-state` outcome.\n\n## Actors\n\nAn actor is who may ask this context for something. Every grant below points at a command this specification declares — a grant is a resolved reference, so \"may invoke\" something nobody wrote is not a permission this model can express, and an authorisation that authorises nothing cannot ship quietly.\n\n### `Receptionist`\n\n`gatepass.visit.Receptionist`, shown to a person as \"Receptionist\".\n\nIt may invoke [`AdmitVisitor`](#admitvisitor), [`RegisterVisit`](#registervisit) and [`SignOutVisitor`](#signoutvisitor).\n\n### `SecurityAuditor`\n\n`gatepass.visit.SecurityAuditor`, shown to a person as \"Security auditor\".\n\nIt may invoke nothing: it observes. \"Who is in this picture\" is part of what a specification describes, so an actor with no grant is a statement rather than an unfinished line.\n\n\n---\n\nGenerated from gatepass v1 · model digest `f2e0f8ff51c077fa1c713d8151544379bafac36a5a927e71c685042d53ab6e61` · contract digest `e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e`. Do not edit this file; change the specification and regenerate it with `ess generate`.\n"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned stdout ----

thread 'the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned' (1122082) panicked at crates/generate/ess-gen/tests/corpus.rs:112:9:
assertion `left == right` failed: `docs/domains/oracle-dispatch.md` of `oracle-fixture` is not what is pinned. This is what every adopter's committed pages would gain in `git diff`; if the change is deliberate, regenerate the corpus in the commit that meant to make it and say why in the message
  left: "<!--\ngenerated from oracle v1\nmodel digest 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee\ncontract digest slice-sha256/2:e4417ff3378e2edc8b6ef43d12299606cac561b5a4510b27af3106da521d2511\ndo not edit: regenerate with `ess generate`\n-->\n\n# dispatch\n\nHanding an order to a carrier, which may refuse for reasons no input predicts.\n\n`oracle.dispatch` is one of oracle's bounded contexts. [Back to the index](../index.md).\n\n## Types\n\n### `HandoffId`\n\n`oracle.dispatch.HandoffId` wraps `Uuid` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `Label`\n\n`oracle.dispatch.Label` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `Recipient`\n\n`oracle.dispatch.Recipient` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n## Commands\n\n### `Handoff`\n\n`oracle.dispatch.Handoff`, shown to a person as \"Hand off\" and called `handoff` on the wire.\n\nIt takes:\n\n- `recipient` — `oracle.dispatch.Recipient`\n- `label` — `oracle.dispatch.Label`\n\nIt has two outcomes.\n\n**`accepted`** — The carrier took it. The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `oracle.dispatch.HandedOff`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`refused`** — The carrier refused; no input decides this, so a scenario injects it. Decided outside the input: the carrier has no capacity for this handoff. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `oracle.dispatch.NoCapacity`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.\n\n## Events\n\n### `HandedOff`\n\n`oracle.dispatch.HandedOff`.\n\nIt carries:\n\n- `handoff_id` — `oracle.dispatch.HandoffId`\n- `recipient` — `oracle.dispatch.Recipient`\n\nEmitted by `oracle.dispatch.Handoff` on its `accepted` outcome.\n\nNothing in this system reacts to it.\n\n### `HandoffEscalated`\n\n`oracle.dispatch.HandoffEscalated`, shown to a person as \"Handoff escalated\".\n\nIt carries:\n\n- `recipient` — `oracle.dispatch.Recipient`\n- `label` — `oracle.dispatch.Label`\n\nEmitted when binding `handoff-on-held` escalates: `oracle.dispatch.Handoff` failed and a person was told.\n\nNothing in this system reacts to it.\n\n## Errors\n\n### `NoCapacity`\n\nThe carrier had no capacity for this handoff.\n\nIt carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.\n\nReported by `oracle.dispatch.Handoff` on its `refused` outcome.\n\n## Type crossings\n\nTypes in this context that the specification permits to be used as another type, or the other way round. Nothing else crosses: two newtypes over the same primitive stay distinct until a line in the specification says otherwise.\n\n**`oracle.order.Email` may be used as `oracle.dispatch.Recipient`**, because:\n\n> An order's contact address is where the carrier's notice goes; the dispatch context validates it again on the way out, so the order context does not have to know how.\n\nEvery crossing in the system is on one page: [Type crossings](../crossings.md).\n\n\n---\n\nGenerated from oracle v1 · model digest `4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee` · contract digest `slice-sha256/2:e4417ff3378e2edc8b6ef43d12299606cac561b5a4510b27af3106da521d2511`. Do not edit this file; change the specification and regenerate it with `ess generate`.\n"
 right: "<!--\ngenerated from oracle v1\nmodel digest 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee\ncontract digest e4417ff3378e2edc8b6ef43d12299606cac561b5a4510b27af3106da521d2511\ndo not edit: regenerate with `ess generate`\n-->\n\n# dispatch\n\nHanding an order to a carrier, which may refuse for reasons no input predicts.\n\n`oracle.dispatch` is one of oracle's bounded contexts. [Back to the index](../index.md).\n\n## Types\n\n### `HandoffId`\n\n`oracle.dispatch.HandoffId` wraps `Uuid` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `Label`\n\n`oracle.dispatch.Label` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `Recipient`\n\n`oracle.dispatch.Recipient` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n## Commands\n\n### `Handoff`\n\n`oracle.dispatch.Handoff`, shown to a person as \"Hand off\" and called `handoff` on the wire.\n\nIt takes:\n\n- `recipient` — `oracle.dispatch.Recipient`\n- `label` — `oracle.dispatch.Label`\n\nIt has two outcomes.\n\n**`accepted`** — The carrier took it. The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `oracle.dispatch.HandedOff`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`refused`** — The carrier refused; no input decides this, so a scenario injects it. Decided outside the input: the carrier has no capacity for this handoff. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `oracle.dispatch.NoCapacity`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.\n\n## Events\n\n### `HandedOff`\n\n`oracle.dispatch.HandedOff`.\n\nIt carries:\n\n- `handoff_id` — `oracle.dispatch.HandoffId`\n- `recipient` — `oracle.dispatch.Recipient`\n\nEmitted by `oracle.dispatch.Handoff` on its `accepted` outcome.\n\nNothing in this system reacts to it.\n\n### `HandoffEscalated`\n\n`oracle.dispatch.HandoffEscalated`, shown to a person as \"Handoff escalated\".\n\nIt carries:\n\n- `recipient` — `oracle.dispatch.Recipient`\n- `label` — `oracle.dispatch.Label`\n\nEmitted when binding `handoff-on-held` escalates: `oracle.dispatch.Handoff` failed and a person was told.\n\nNothing in this system reacts to it.\n\n## Errors\n\n### `NoCapacity`\n\nThe carrier had no capacity for this handoff.\n\nIt carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.\n\nReported by `oracle.dispatch.Handoff` on its `refused` outcome.\n\n## Type crossings\n\nTypes in this context that the specification permits to be used as another type, or the other way round. Nothing else crosses: two newtypes over the same primitive stay distinct until a line in the specification says otherwise.\n\n**`oracle.order.Email` may be used as `oracle.dispatch.Recipient`**, because:\n\n> An order's contact address is where the carrier's notice goes; the dispatch context validates it again on the way out, so the order context does not have to know how.\n\nEvery crossing in the system is on one page: [Type crossings](../crossings.md).\n\n\n---\n\nGenerated from oracle v1 · model digest `4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee` · contract digest `e4417ff3378e2edc8b6ef43d12299606cac561b5a4510b27af3106da521d2511`. Do not edit this file; change the specification and regenerate it with `ess generate`.\n"

---- the_billing_documentation_is_byte_for_byte_what_is_pinned stdout ----

thread 'the_billing_documentation_is_byte_for_byte_what_is_pinned' (1122080) panicked at crates/generate/ess-gen/tests/corpus.rs:112:9:
assertion `left == right` failed: `docs/domains/billing-email.md` of `billing` is not what is pinned. This is what every adopter's committed pages would gain in `git diff`; if the change is deliberate, regenerate the corpus in the commit that meant to make it and say why in the message
  left: "<!--\ngenerated from billing v3\nmodel digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\ncontract digest slice-sha256/2:6885abd57db28f85bf636a59960c5e76be692eb6d69ce48089f10ca44cc9130f\ndo not edit: regenerate with `ess generate`\n-->\n\n# email\n\nSending the notifications other contexts ask for.\n\n`billing.email` is one of billing's bounded contexts. [Back to the index](../index.md).\n\n## Types\n\n### `EmailAddress`\n\n`billing.email.EmailAddress` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `MessageId`\n\n`billing.email.MessageId` wraps `Uuid` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `TemplateId`\n\n`billing.email.TemplateId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n## Commands\n\n### `SendEmail`\n\n`billing.email.SendEmail`.\n\nIt takes:\n\n- `recipient` — `billing.email.EmailAddress`\n- `template` — `billing.email.TemplateId`\n\nIt has two outcomes.\n\n**`sent`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `billing.email.EmailSent`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`failed`** — Decided outside the input: the provider rejects the recipient address. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `billing.email.Undeliverable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.\n\n## Events\n\n### `DeliveryEscalated`\n\n`billing.email.DeliveryEscalated`, shown to a person as \"Delivery escalated\".\n\nIt carries:\n\n- `recipient` — `billing.email.EmailAddress`\n- `template` — `billing.email.TemplateId`\n\nEmitted when binding `notify-on-invoice-created` escalates: `billing.email.SendEmail` failed and a person was told.\n\nNothing in this system reacts to it.\n\n### `EmailSent`\n\n`billing.email.EmailSent`.\n\nIt carries:\n\n- `message_id` — `billing.email.MessageId`\n- `recipient` — `billing.email.EmailAddress`\n\nEmitted by `billing.email.SendEmail` on its `sent` outcome.\n\nNothing in this system reacts to it.\n\n## Errors\n\n### `Undeliverable`\n\nThe address was rejected by the provider.\n\nIt carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.\n\nReported by `billing.email.SendEmail` on its `failed` outcome.\n\n## Type crossings\n\nTypes in this context that the specification permits to be used as another type, or the other way round. Nothing else crosses: two newtypes over the same primitive stay distinct until a line in the specification says otherwise.\n\n**`billing.invoice.Email` may be used as `billing.email.EmailAddress`**, because:\n\n> An invoice's customer email is a deliverable address; the email context validates it again on the way out, so the invoice context does not have to know how.\n\nEvery crossing in the system is on one page: [Type crossings](../crossings.md).\n\n\n---\n\nGenerated from billing v3 · model digest `56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942` · contract digest `slice-sha256/2:6885abd57db28f85bf636a59960c5e76be692eb6d69ce48089f10ca44cc9130f`. Do not edit this file; change the specification and regenerate it with `ess generate`.\n"
 right: "<!--\ngenerated from billing v3\nmodel digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\ncontract digest 1ccd0afca5f71d3c3c98894259751819e5313bcd73222af85bb80774dbe83e6f\ndo not edit: regenerate with `ess generate`\n-->\n\n# email\n\nSending the notifications other contexts ask for.\n\n`billing.email` is one of billing's bounded contexts. [Back to the index](../index.md).\n\n## Types\n\n### `EmailAddress`\n\n`billing.email.EmailAddress` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `MessageId`\n\n`billing.email.MessageId` wraps `Uuid` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n### `TemplateId`\n\n`billing.email.TemplateId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.\n\n## Commands\n\n### `SendEmail`\n\n`billing.email.SendEmail`.\n\nIt takes:\n\n- `recipient` — `billing.email.EmailAddress`\n- `template` — `billing.email.TemplateId`\n\nIt has two outcomes.\n\n**`sent`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `billing.email.EmailSent`. A test reaches it by constructing an input that satisfies no other outcome's condition.\n\n**`failed`** — Decided outside the input: the provider rejects the recipient address. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `billing.email.Undeliverable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.\n\n## Events\n\n### `DeliveryEscalated`\n\n`billing.email.DeliveryEscalated`, shown to a person as \"Delivery escalated\".\n\nIt carries:\n\n- `recipient` — `billing.email.EmailAddress`\n- `template` — `billing.email.TemplateId`\n\nEmitted when binding `notify-on-invoice-created` escalates: `billing.email.SendEmail` failed and a person was told.\n\nNothing in this system reacts to it.\n\n### `EmailSent`\n\n`billing.email.EmailSent`.\n\nIt carries:\n\n- `message_id` — `billing.email.MessageId`\n- `recipient` — `billing.email.EmailAddress`\n\nEmitted by `billing.email.SendEmail` on its `sent` outcome.\n\nNothing in this system reacts to it.\n\n## Errors\n\n### `Undeliverable`\n\nThe address was rejected by the provider.\n\nIt carries nothing beyond its name, so a caller can tell what went wrong and not which value caused it.\n\nReported by `billing.email.SendEmail` on its `failed` outcome.\n\n## Type crossings\n\nTypes in this context that the specification permits to be used as another type, or the other way round. Nothing else crosses: two newtypes over the same primitive stay distinct until a line in the specification says otherwise.\n\n**`billing.invoice.Email` may be used as `billing.email.EmailAddress`**, because:\n\n> An invoice's customer email is a deliverable address; the email context validates it again on the way out, so the invoice context does not have to know how.\n\nEvery crossing in the system is on one page: [Type crossings](../crossings.md).\n\n\n---\n\nGenerated from billing v3 · model digest `56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942` · contract digest `1ccd0afca5f71d3c3c98894259751819e5313bcd73222af85bb80774dbe83e6f`. Do not edit this file; change the specification and regenerate it with `ess generate`.\n"


failures:
    the_billing_documentation_is_byte_for_byte_what_is_pinned
    the_gatepass_documentation_is_byte_for_byte_what_is_pinned
    the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p ess-gen --test corpus`
     Running tests/determinism.rs (target/debug/deps/determinism-82fd72ce337b58c9)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-9d17c089cbb09f3a)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test every_page_says_which_specification_produced_it ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/openapi.rs (target/debug/deps/openapi-84c5c21bb15c13da)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test every_component_gets_one_document_named_after_it ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test a_command_is_only_ever_a_post ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/provenance.rs (target/debug/deps/provenance-97bf54f63a94402c)

running 16 tests
test a_text_without_both_digests_reads_as_nothing ... ok
test a_damaged_digest_reads_as_nothing ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... ok
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok
test review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp ... ok
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... ok
test review_conflicting_structured_and_comment_stamps_are_unreadable ... ok
test review_marker_looking_model_content_does_not_override_real_emitted_stamps ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/relations.rs (target/debug/deps/relations-9e1fbfdeb5e1e1b5)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... FAILED
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... FAILED

failures:

---- the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes stdout ----

thread 'the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes' (1122176) panicked at crates/generate/ess-gen/tests/relations.rs:172:5:
assertion `left == right` failed: `generated/openapi/invoice-service.yaml` is not what `openapi` writes for the billing example. The committed tree is what an adopter reads; regenerate it in the commit that changed the projection
  left: "# generated from billing v3\n# model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n# contract digest slice-sha256/2:c8fefbb1c7bb7c761f120e779652a3febf75e3f89339f148f30c39c72717c897\n# do not edit: regenerate with `ess generate`\nopenapi: 3.1.0\ninfo:\n  title: invoice-service\n  summary: Issues invoices and tracks payment.\n  description: |-\n    The HTTP surface of `invoice-service`, one of the components of `billing` v3.\n\n    Every path here is one semantic command, so the method is always POST and the path is the command's wire name under its domain's: a command is not a resource, and this document does not invent one. A status code is the outcome the specification declares — 202 for a branch that was taken, 422 for a refusal the input decides, 502 for a refusal decided outside the request — and the `outcome` property of every response body names the branch. Events emitted by a branch are published to consumers through the event transport; they are not returned here.\n  version: v3\n  x-ess-provenance:\n    system: billing\n    specification_version: v3\n    source_digest: 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n    contract_digest: slice-sha256/2:c8fefbb1c7bb7c761f120e779652a3febf75e3f89339f148f30c39c72717c897\ntags:\n- name: invoices\n  description: Issuing invoices and tracking whether they are paid.\npaths:\n  /invoices/commands/cancel-invoice:\n    post:\n      operationId: billing.invoice.CancelInvoice\n      summary: Cancel invoice\n      tags:\n      - invoices\n      requestBody:\n        description: The input `billing.invoice.CancelInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.CancelInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `cancelled`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CancelInvoice.cancelled.Response'\n        '409':\n          description: 'Outcome `wrong-state`: the input was acceptable and the subject is in a state this command does not act from. Resending the same request changes nothing until something else moves it.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CancelInvoice.wrong-state.Response'\n  /invoices/commands/create-invoice:\n    post:\n      operationId: billing.invoice.CreateInvoice\n      summary: Create invoice\n      tags:\n      - invoices\n      x-ess-may-invoke:\n      - billing.invoice.Customer\n      requestBody:\n        description: The input `billing.invoice.CreateInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.CreateInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `accepted`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CreateInvoice.accepted.Response'\n        '422':\n          description: 'Outcome `rejected`: the request was understood and refused on domain grounds. The body names the declared error and carries whatever that error declares.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CreateInvoice.rejected.Response'\n  /invoices/commands/issue-invoice:\n    post:\n      operationId: billing.invoice.IssueInvoice\n      summary: Issue invoice\n      tags:\n      - invoices\n      requestBody:\n        description: The input `billing.invoice.IssueInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.IssueInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `issued`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.IssueInvoice.issued.Response'\n        '409':\n          description: 'Outcome `wrong-state`: the input was acceptable and the subject is in a state this command does not act from. Resending the same request changes nothing until something else moves it.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.IssueInvoice.wrong-state.Response'\n  /invoices/commands/pay-invoice:\n    post:\n      operationId: billing.invoice.PayInvoice\n      summary: Pay invoice\n      tags:\n      - invoices\n      requestBody:\n        description: The input `billing.invoice.PayInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.PayInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `settled`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.PayInvoice.settled.Response'\n        '409':\n          description: 'Outcome `wrong-state`: the input was acceptable and the subject is in a state this command does not act from. Resending the same request changes nothing until something else moves it.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.PayInvoice.wrong-state.Response'\n        '422':\n          description: 'Outcome `rejected`: the request was understood and refused on domain grounds. The body names the declared error and carries whatever that error declares.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.PayInvoice.rejected.Response'\ncomponents:\n  schemas:\n    billing.invoice.AccountId:\n      title: AccountId\n      x-ess-name: billing.invoice.AccountId\n      x-ess-kind: newtype\n      type: string\n      format: uuid\n      pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n    billing.invoice.CancelInvoice.Input:\n      title: Cancel invoice input\n      x-ess-name: billing.invoice.CancelInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        invoice_id:\n          $ref: '#/components/schemas/billing.invoice.InvoiceId'\n      required:\n      - invoice_id\n      additionalProperties: false\n    billing.invoice.CancelInvoice.cancelled.Response:\n      additionalProperties: false\n      description: The invoice is cancelled, from Draft or from Issued. Taken when no other outcome's condition matched. A `billing.invoice.Invoice` has moved to `Cancelled`, along `cancel`. The instance is the one `invoice_id` names. Emits `InvoiceCancelled`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: cancelled\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.CancelInvoice.wrong-state.Response:\n      additionalProperties: false\n      description: The invoice is already Paid or already Cancelled, so nothing was cancelled. Taken when the subject is in a state none of this command's declared moves start from. Which states those are is the lifecycle's answer, not this command's.\n      properties:\n        error:\n          const: billing.invoice.InvoiceStateConflict\n          description: The invoice is not in a state this command acts from, so nothing moved.\n          type: string\n        outcome:\n          const: wrong-state\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvoiceStateConflict.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.CreateInvoice.Input:\n      title: Create invoice input\n      x-ess-name: billing.invoice.CreateInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        account_id:\n          $ref: '#/components/schemas/billing.invoice.AccountId'\n        customer_email:\n          $ref: '#/components/schemas/billing.invoice.Email'\n        amount:\n          $ref: '#/components/schemas/billing.invoice.Money'\n      required:\n      - account_id\n      - customer_email\n      - amount\n      additionalProperties: false\n    billing.invoice.CreateInvoice.accepted.Response:\n      additionalProperties: false\n      description: The invoice is created in Draft. Taken when `amount.amount > 0` holds of the input. A `billing.invoice.Invoice` now exists, in `Draft`. Its identity is published as `invoice_id` on `billing.invoice.InvoiceCreated`. Emits `InvoiceCreated`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: accepted\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.CreateInvoice.rejected.Response:\n      additionalProperties: false\n      description: The amount was not positive, and nothing was created. Taken when no other outcome's condition matched.\n      properties:\n        error:\n          const: billing.invoice.InvalidAmount\n          description: The requested amount is not positive.\n          type: string\n        outcome:\n          const: rejected\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvalidAmount.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.Email:\n      title: Email\n      x-ess-name: billing.invoice.Email\n      x-ess-kind: newtype\n      type: string\n    billing.invoice.InvalidAmount.Error:\n      title: InvalidAmount payload\n      description: The requested amount is not positive.\n      x-ess-name: billing.invoice.InvalidAmount\n      x-ess-kind: error-payload\n      type: object\n      properties:\n        submitted:\n          $ref: '#/components/schemas/billing.invoice.Money'\n      required:\n      - submitted\n      additionalProperties: false\n    billing.invoice.Invoice.State:\n      title: State\n      x-ess-name: billing.invoice.Invoice.State\n      x-ess-kind: enum\n      type: string\n      enum:\n      - Cancelled\n      - Draft\n      - Issued\n      - Paid\n    billing.invoice.InvoiceId:\n      title: InvoiceId\n      x-ess-name: billing.invoice.InvoiceId\n      x-ess-kind: newtype\n      type: string\n      format: uuid\n      pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n    billing.invoice.InvoiceStateConflict.Error:\n      title: InvoiceStateConflict payload\n      description: The invoice is not in a state this command acts from, so nothing moved.\n      x-ess-name: billing.invoice.InvoiceStateConflict\n      x-ess-kind: error-payload\n      type: object\n      properties:\n        state:\n          $ref: '#/components/schemas/billing.invoice.Invoice.State'\n      required:\n      - state\n      additionalProperties: false\n    billing.invoice.IssueInvoice.Input:\n      title: Issue invoice input\n      x-ess-name: billing.invoice.IssueInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        invoice_id:\n          $ref: '#/components/schemas/billing.invoice.InvoiceId'\n      required:\n      - invoice_id\n      additionalProperties: false\n    billing.invoice.IssueInvoice.issued.Response:\n      additionalProperties: false\n      description: The invoice leaves Draft and is now Issued. Taken when no other outcome's condition matched. A `billing.invoice.Invoice` has moved to `Issued`, along `issue`. The instance is the one `invoice_id` names. Emits `InvoiceIssued`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: issued\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.IssueInvoice.wrong-state.Response:\n      additionalProperties: false\n      description: The invoice is not in Draft, so it was not issued. Taken when the subject is in a state none of this command's declared moves start from. Which states those are is the lifecycle's answer, not this command's.\n      properties:\n        error:\n          const: billing.invoice.InvoiceStateConflict\n          description: The invoice is not in a state this command acts from, so nothing moved.\n          type: string\n        outcome:\n          const: wrong-state\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvoiceStateConflict.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.Money:\n      title: Money\n      x-ess-name: billing.invoice.Money\n      x-ess-kind: struct\n      type: object\n      properties:\n        amount:\n          type: string\n          format: decimal\n          pattern: ^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$\n        currency:\n          type: string\n      required:\n      - amount\n      - currency\n      additionalProperties: false\n      x-ess-invariants:\n      - amount >= 0\n    billing.invoice.PayInvoice.Input:\n      title: Pay invoice input\n      x-ess-name: billing.invoice.PayInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        invoice_id:\n          $ref: '#/components/schemas/billing.invoice.InvoiceId'\n        amount:\n          $ref: '#/components/schemas/billing.invoice.Money'\n      required:\n      - invoice_id\n      - amount\n      additionalProperties: false\n    billing.invoice.PayInvoice.rejected.Response:\n      additionalProperties: false\n      description: The payment was not positive, so the invoice did not move. Taken when no other outcome's condition matched.\n      properties:\n        error:\n          const: billing.invoice.InvalidAmount\n          description: The requested amount is not positive.\n          type: string\n        outcome:\n          const: rejected\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvalidAmount.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.PayInvoice.settled.Response:\n      additionalProperties: false\n      description: The payment is accepted and the invoice becomes Paid. Taken when `amount.amount > 0` holds of the input. A `billing.invoice.Invoice` has moved to `Paid`, along `settle`. The instance is the one `invoice_id` names. Emits `InvoicePaid`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: settled\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.PayInvoice.wrong-state.Response:\n      additionalProperties: false\n      description: The invoice is not Issued, so the payment did not settle it. Taken when the subject is in a state none of this command's declared moves start from. Which states those are is the lifecycle's answer, not this command's.\n      properties:\n        error:\n          const: billing.invoice.InvoiceStateConflict\n          description: The invoice is not in a state this command acts from, so nothing moved.\n          type: string\n        outcome:\n          const: wrong-state\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvoiceStateConflict.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\nx-ess-entities:\n  billing.invoice.Account:\n    title: Account\n    x-ess-name: billing.invoice.Account\n    x-ess-kind: entity\n    type: object\n    properties:\n      account_id:\n        $ref: '#/x-ess-entities/billing.invoice.AccountId'\n      display_name:\n        type: string\n      state:\n        $ref: '#/x-ess-entities/billing.invoice.Account.State'\n    required:\n    - account_id\n    - display_name\n    - state\n    additionalProperties: false\n  billing.invoice.Account.State:\n    title: State\n    x-ess-name: billing.invoice.Account.State\n    x-ess-kind: enum\n    type: string\n    enum:\n    - Active\n  billing.invoice.AccountId:\n    title: AccountId\n    x-ess-name: billing.invoice.AccountId\n    x-ess-kind: newtype\n    type: string\n    format: uuid\n    pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n  billing.invoice.Channel:\n    title: Delivery channel\n    x-ess-name: billing.invoice.Channel\n    x-ess-kind: enum\n    type: string\n    enum:\n    - Email\n    - Post\n    - Portal\n  billing.invoice.CompanyRef:\n    title: CompanyRef\n    x-ess-name: billing.invoice.CompanyRef\n    x-ess-kind: newtype\n    type: string\n  billing.invoice.Email:\n    title: Email\n    x-ess-name: billing.invoice.Email\n    x-ess-kind: newtype\n    type: string\n  billing.invoice.Invoice:\n    title: Invoice\n    x-ess-name: billing.invoice.Invoice\n    x-ess-kind: entity\n    type: object\n    properties:\n      invoice_id:\n        $ref: '#/x-ess-entities/billing.invoice.InvoiceId'\n      account_id:\n        $ref: '#/x-ess-entities/billing.invoice.AccountId'\n        x-ess-relation:\n          name: invoices\n          kind: owns\n          source: billing.invoice.Account\n          target: billing.invoice.Invoice\n          cardinality: many\n          via: account_id\n          $ref: '#/x-ess-entities/billing.invoice.Account'\n      total:\n        $ref: '#/x-ess-entities/billing.invoice.Money'\n      payee:\n        $ref: '#/x-ess-entities/billing.invoice.Payee'\n      channel:\n        $ref: '#/x-ess-entities/billing.invoice.Channel'\n      lines:\n        type: array\n        items:\n          $ref: '#/x-ess-entities/billing.invoice.LineItem'\n      note:\n        type: string\n      metadata:\n        type: object\n        additionalProperties:\n          type: string\n        x-ess-map-key: String\n      issued_at:\n        type: string\n        format: date-time\n      settlement_window:\n        type: string\n        format: duration\n      is_recurring:\n        type: boolean\n      signature:\n        type: string\n        pattern: ^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$\n        contentEncoding: base64\n      reminder_count:\n        type: integer\n      state:\n        $ref: '#/x-ess-entities/billing.invoice.Invoice.State'\n    required:\n    - invoice_id\n    - account_id\n    - total\n    - payee\n    - channel\n    - lines\n    - metadata\n    - settlement_window\n    - is_recurring\n    - signature\n    - reminder_count\n    - state\n    additionalProperties: false\n  billing.invoice.Invoice.State:\n    title: State\n    x-ess-name: billing.invoice.Invoice.State\n    x-ess-kind: enum\n    type: string\n    enum:\n    - Cancelled\n    - Draft\n    - Issued\n    - Paid\n  billing.invoice.InvoiceId:\n    title: InvoiceId\n    x-ess-name: billing.invoice.InvoiceId\n    x-ess-kind: newtype\n    type: string\n    format: uuid\n    pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n  billing.invoice.LineItem:\n    title: LineItem\n    x-ess-name: billing.invoice.LineItem\n    x-ess-kind: struct\n    type: object\n    properties:\n      description:\n        type: string\n      quantity:\n        type: integer\n      unit_price:\n        $ref: '#/x-ess-entities/billing.invoice.Money'\n    required:\n    - description\n    - quantity\n    - unit_price\n    additionalProperties: false\n  billing.invoice.Money:\n    title: Money\n    x-ess-name: billing.invoice.Money\n    x-ess-kind: struct\n    type: object\n    properties:\n      amount:\n        type: string\n        format: decimal\n        pattern: ^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$\n      currency:\n        type: string\n    required:\n    - amount\n    - currency\n    additionalProperties: false\n    x-ess-invariants:\n    - amount >= 0\n  billing.invoice.Payee:\n    title: Payee\n    x-ess-name: billing.invoice.Payee\n    x-ess-kind: union\n    oneOf:\n    - title: company\n      type: object\n      properties:\n        kind:\n          type: string\n          const: company\n        value:\n          $ref: '#/x-ess-entities/billing.invoice.CompanyRef'\n      required:\n      - kind\n      - value\n      additionalProperties: false\n    - title: person\n      type: object\n      properties:\n        kind:\n          type: string\n          const: person\n        value:\n          $ref: '#/x-ess-entities/billing.invoice.Email'\n      required:\n      - kind\n      - value\n      additionalProperties: false\n    x-ess-union-tag: kind\n"
 right: "# generated from billing v3\n# model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n# contract digest 4a8707523699f478d5d4e0068d6953a49cc0a36b9b026bab5ef7eeb6244f0fdd\n# do not edit: regenerate with `ess generate`\nopenapi: 3.1.0\ninfo:\n  title: invoice-service\n  summary: Issues invoices and tracks payment.\n  description: |-\n    The HTTP surface of `invoice-service`, one of the components of `billing` v3.\n\n    Every path here is one semantic command, so the method is always POST and the path is the command's wire name under its domain's: a command is not a resource, and this document does not invent one. A status code is the outcome the specification declares — 202 for a branch that was taken, 422 for a refusal the input decides, 502 for a refusal decided outside the request — and the `outcome` property of every response body names the branch. Events emitted by a branch are published to consumers through the event transport; they are not returned here.\n  version: v3\n  x-ess-provenance:\n    system: billing\n    specification_version: v3\n    source_digest: 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n    contract_digest: 4a8707523699f478d5d4e0068d6953a49cc0a36b9b026bab5ef7eeb6244f0fdd\ntags:\n- name: invoices\n  description: Issuing invoices and tracking whether they are paid.\npaths:\n  /invoices/commands/cancel-invoice:\n    post:\n      operationId: billing.invoice.CancelInvoice\n      summary: Cancel invoice\n      tags:\n      - invoices\n      requestBody:\n        description: The input `billing.invoice.CancelInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.CancelInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `cancelled`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CancelInvoice.cancelled.Response'\n        '409':\n          description: 'Outcome `wrong-state`: the input was acceptable and the subject is in a state this command does not act from. Resending the same request changes nothing until something else moves it.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CancelInvoice.wrong-state.Response'\n  /invoices/commands/create-invoice:\n    post:\n      operationId: billing.invoice.CreateInvoice\n      summary: Create invoice\n      tags:\n      - invoices\n      x-ess-may-invoke:\n      - billing.invoice.Customer\n      requestBody:\n        description: The input `billing.invoice.CreateInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.CreateInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `accepted`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CreateInvoice.accepted.Response'\n        '422':\n          description: 'Outcome `rejected`: the request was understood and refused on domain grounds. The body names the declared error and carries whatever that error declares.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.CreateInvoice.rejected.Response'\n  /invoices/commands/issue-invoice:\n    post:\n      operationId: billing.invoice.IssueInvoice\n      summary: Issue invoice\n      tags:\n      - invoices\n      requestBody:\n        description: The input `billing.invoice.IssueInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.IssueInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `issued`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.IssueInvoice.issued.Response'\n        '409':\n          description: 'Outcome `wrong-state`: the input was acceptable and the subject is in a state this command does not act from. Resending the same request changes nothing until something else moves it.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.IssueInvoice.wrong-state.Response'\n  /invoices/commands/pay-invoice:\n    post:\n      operationId: billing.invoice.PayInvoice\n      summary: Pay invoice\n      tags:\n      - invoices\n      requestBody:\n        description: The input `billing.invoice.PayInvoice` declares.\n        required: true\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/billing.invoice.PayInvoice.Input'\n      responses:\n        '202':\n          description: 'Outcome `settled`: the branch the specification declares for this input. Events this branch emits are published to consumers, not returned here.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.PayInvoice.settled.Response'\n        '409':\n          description: 'Outcome `wrong-state`: the input was acceptable and the subject is in a state this command does not act from. Resending the same request changes nothing until something else moves it.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.PayInvoice.wrong-state.Response'\n        '422':\n          description: 'Outcome `rejected`: the request was understood and refused on domain grounds. The body names the declared error and carries whatever that error declares.'\n          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/billing.invoice.PayInvoice.rejected.Response'\ncomponents:\n  schemas:\n    billing.invoice.AccountId:\n      title: AccountId\n      x-ess-name: billing.invoice.AccountId\n      x-ess-kind: newtype\n      type: string\n      format: uuid\n      pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n    billing.invoice.CancelInvoice.Input:\n      title: Cancel invoice input\n      x-ess-name: billing.invoice.CancelInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        invoice_id:\n          $ref: '#/components/schemas/billing.invoice.InvoiceId'\n      required:\n      - invoice_id\n      additionalProperties: false\n    billing.invoice.CancelInvoice.cancelled.Response:\n      additionalProperties: false\n      description: The invoice is cancelled, from Draft or from Issued. Taken when no other outcome's condition matched. A `billing.invoice.Invoice` has moved to `Cancelled`, along `cancel`. The instance is the one `invoice_id` names. Emits `InvoiceCancelled`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: cancelled\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.CancelInvoice.wrong-state.Response:\n      additionalProperties: false\n      description: The invoice is already Paid or already Cancelled, so nothing was cancelled. Taken when the subject is in a state none of this command's declared moves start from. Which states those are is the lifecycle's answer, not this command's.\n      properties:\n        error:\n          const: billing.invoice.InvoiceStateConflict\n          description: The invoice is not in a state this command acts from, so nothing moved.\n          type: string\n        outcome:\n          const: wrong-state\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvoiceStateConflict.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.CreateInvoice.Input:\n      title: Create invoice input\n      x-ess-name: billing.invoice.CreateInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        account_id:\n          $ref: '#/components/schemas/billing.invoice.AccountId'\n        customer_email:\n          $ref: '#/components/schemas/billing.invoice.Email'\n        amount:\n          $ref: '#/components/schemas/billing.invoice.Money'\n      required:\n      - account_id\n      - customer_email\n      - amount\n      additionalProperties: false\n    billing.invoice.CreateInvoice.accepted.Response:\n      additionalProperties: false\n      description: The invoice is created in Draft. Taken when `amount.amount > 0` holds of the input. A `billing.invoice.Invoice` now exists, in `Draft`. Its identity is published as `invoice_id` on `billing.invoice.InvoiceCreated`. Emits `InvoiceCreated`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: accepted\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.CreateInvoice.rejected.Response:\n      additionalProperties: false\n      description: The amount was not positive, and nothing was created. Taken when no other outcome's condition matched.\n      properties:\n        error:\n          const: billing.invoice.InvalidAmount\n          description: The requested amount is not positive.\n          type: string\n        outcome:\n          const: rejected\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvalidAmount.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.Email:\n      title: Email\n      x-ess-name: billing.invoice.Email\n      x-ess-kind: newtype\n      type: string\n    billing.invoice.InvalidAmount.Error:\n      title: InvalidAmount payload\n      description: The requested amount is not positive.\n      x-ess-name: billing.invoice.InvalidAmount\n      x-ess-kind: error-payload\n      type: object\n      properties:\n        submitted:\n          $ref: '#/components/schemas/billing.invoice.Money'\n      required:\n      - submitted\n      additionalProperties: false\n    billing.invoice.Invoice.State:\n      title: State\n      x-ess-name: billing.invoice.Invoice.State\n      x-ess-kind: enum\n      type: string\n      enum:\n      - Cancelled\n      - Draft\n      - Issued\n      - Paid\n    billing.invoice.InvoiceId:\n      title: InvoiceId\n      x-ess-name: billing.invoice.InvoiceId\n      x-ess-kind: newtype\n      type: string\n      format: uuid\n      pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n    billing.invoice.InvoiceStateConflict.Error:\n      title: InvoiceStateConflict payload\n      description: The invoice is not in a state this command acts from, so nothing moved.\n      x-ess-name: billing.invoice.InvoiceStateConflict\n      x-ess-kind: error-payload\n      type: object\n      properties:\n        state:\n          $ref: '#/components/schemas/billing.invoice.Invoice.State'\n      required:\n      - state\n      additionalProperties: false\n    billing.invoice.IssueInvoice.Input:\n      title: Issue invoice input\n      x-ess-name: billing.invoice.IssueInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        invoice_id:\n          $ref: '#/components/schemas/billing.invoice.InvoiceId'\n      required:\n      - invoice_id\n      additionalProperties: false\n    billing.invoice.IssueInvoice.issued.Response:\n      additionalProperties: false\n      description: The invoice leaves Draft and is now Issued. Taken when no other outcome's condition matched. A `billing.invoice.Invoice` has moved to `Issued`, along `issue`. The instance is the one `invoice_id` names. Emits `InvoiceIssued`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: issued\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.IssueInvoice.wrong-state.Response:\n      additionalProperties: false\n      description: The invoice is not in Draft, so it was not issued. Taken when the subject is in a state none of this command's declared moves start from. Which states those are is the lifecycle's answer, not this command's.\n      properties:\n        error:\n          const: billing.invoice.InvoiceStateConflict\n          description: The invoice is not in a state this command acts from, so nothing moved.\n          type: string\n        outcome:\n          const: wrong-state\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvoiceStateConflict.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.Money:\n      title: Money\n      x-ess-name: billing.invoice.Money\n      x-ess-kind: struct\n      type: object\n      properties:\n        amount:\n          type: string\n          format: decimal\n          pattern: ^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$\n        currency:\n          type: string\n      required:\n      - amount\n      - currency\n      additionalProperties: false\n      x-ess-invariants:\n      - amount >= 0\n    billing.invoice.PayInvoice.Input:\n      title: Pay invoice input\n      x-ess-name: billing.invoice.PayInvoice\n      x-ess-kind: command-input\n      type: object\n      properties:\n        invoice_id:\n          $ref: '#/components/schemas/billing.invoice.InvoiceId'\n        amount:\n          $ref: '#/components/schemas/billing.invoice.Money'\n      required:\n      - invoice_id\n      - amount\n      additionalProperties: false\n    billing.invoice.PayInvoice.rejected.Response:\n      additionalProperties: false\n      description: The payment was not positive, so the invoice did not move. Taken when no other outcome's condition matched.\n      properties:\n        error:\n          const: billing.invoice.InvalidAmount\n          description: The requested amount is not positive.\n          type: string\n        outcome:\n          const: rejected\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvalidAmount.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\n    billing.invoice.PayInvoice.settled.Response:\n      additionalProperties: false\n      description: The payment is accepted and the invoice becomes Paid. Taken when `amount.amount > 0` holds of the input. A `billing.invoice.Invoice` has moved to `Paid`, along `settle`. The instance is the one `invoice_id` names. Emits `InvoicePaid`, published to consumers rather than returned here.\n      properties:\n        outcome:\n          const: settled\n          description: Which declared outcome the command took.\n      required:\n      - outcome\n      type: object\n    billing.invoice.PayInvoice.wrong-state.Response:\n      additionalProperties: false\n      description: The invoice is not Issued, so the payment did not settle it. Taken when the subject is in a state none of this command's declared moves start from. Which states those are is the lifecycle's answer, not this command's.\n      properties:\n        error:\n          const: billing.invoice.InvoiceStateConflict\n          description: The invoice is not in a state this command acts from, so nothing moved.\n          type: string\n        outcome:\n          const: wrong-state\n          description: Which declared outcome the command took.\n        payload:\n          $ref: '#/components/schemas/billing.invoice.InvoiceStateConflict.Error'\n      required:\n      - outcome\n      - error\n      - payload\n      type: object\nx-ess-entities:\n  billing.invoice.Account:\n    title: Account\n    x-ess-name: billing.invoice.Account\n    x-ess-kind: entity\n    type: object\n    properties:\n      account_id:\n        $ref: '#/x-ess-entities/billing.invoice.AccountId'\n      display_name:\n        type: string\n      state:\n        $ref: '#/x-ess-entities/billing.invoice.Account.State'\n    required:\n    - account_id\n    - display_name\n    - state\n    additionalProperties: false\n  billing.invoice.Account.State:\n    title: State\n    x-ess-name: billing.invoice.Account.State\n    x-ess-kind: enum\n    type: string\n    enum:\n    - Active\n  billing.invoice.AccountId:\n    title: AccountId\n    x-ess-name: billing.invoice.AccountId\n    x-ess-kind: newtype\n    type: string\n    format: uuid\n    pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n  billing.invoice.Channel:\n    title: Delivery channel\n    x-ess-name: billing.invoice.Channel\n    x-ess-kind: enum\n    type: string\n    enum:\n    - Email\n    - Post\n    - Portal\n  billing.invoice.CompanyRef:\n    title: CompanyRef\n    x-ess-name: billing.invoice.CompanyRef\n    x-ess-kind: newtype\n    type: string\n  billing.invoice.Email:\n    title: Email\n    x-ess-name: billing.invoice.Email\n    x-ess-kind: newtype\n    type: string\n  billing.invoice.Invoice:\n    title: Invoice\n    x-ess-name: billing.invoice.Invoice\n    x-ess-kind: entity\n    type: object\n    properties:\n      invoice_id:\n        $ref: '#/x-ess-entities/billing.invoice.InvoiceId'\n      account_id:\n        $ref: '#/x-ess-entities/billing.invoice.AccountId'\n        x-ess-relation:\n          name: invoices\n          kind: owns\n          source: billing.invoice.Account\n          target: billing.invoice.Invoice\n          cardinality: many\n          via: account_id\n          $ref: '#/x-ess-entities/billing.invoice.Account'\n      total:\n        $ref: '#/x-ess-entities/billing.invoice.Money'\n      payee:\n        $ref: '#/x-ess-entities/billing.invoice.Payee'\n      channel:\n        $ref: '#/x-ess-entities/billing.invoice.Channel'\n      lines:\n        type: array\n        items:\n          $ref: '#/x-ess-entities/billing.invoice.LineItem'\n      note:\n        type: string\n      metadata:\n        type: object\n        additionalProperties:\n          type: string\n        x-ess-map-key: String\n      issued_at:\n        type: string\n        format: date-time\n      settlement_window:\n        type: string\n        format: duration\n      is_recurring:\n        type: boolean\n      signature:\n        type: string\n        pattern: ^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$\n        contentEncoding: base64\n      reminder_count:\n        type: integer\n      state:\n        $ref: '#/x-ess-entities/billing.invoice.Invoice.State'\n    required:\n    - invoice_id\n    - account_id\n    - total\n    - payee\n    - channel\n    - lines\n    - metadata\n    - settlement_window\n    - is_recurring\n    - signature\n    - reminder_count\n    - state\n    additionalProperties: false\n  billing.invoice.Invoice.State:\n    title: State\n    x-ess-name: billing.invoice.Invoice.State\n    x-ess-kind: enum\n    type: string\n    enum:\n    - Cancelled\n    - Draft\n    - Issued\n    - Paid\n  billing.invoice.InvoiceId:\n    title: InvoiceId\n    x-ess-name: billing.invoice.InvoiceId\n    x-ess-kind: newtype\n    type: string\n    format: uuid\n    pattern: ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\n  billing.invoice.LineItem:\n    title: LineItem\n    x-ess-name: billing.invoice.LineItem\n    x-ess-kind: struct\n    type: object\n    properties:\n      description:\n        type: string\n      quantity:\n        type: integer\n      unit_price:\n        $ref: '#/x-ess-entities/billing.invoice.Money'\n    required:\n    - description\n    - quantity\n    - unit_price\n    additionalProperties: false\n  billing.invoice.Money:\n    title: Money\n    x-ess-name: billing.invoice.Money\n    x-ess-kind: struct\n    type: object\n    properties:\n      amount:\n        type: string\n        format: decimal\n        pattern: ^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$\n      currency:\n        type: string\n    required:\n    - amount\n    - currency\n    additionalProperties: false\n    x-ess-invariants:\n    - amount >= 0\n  billing.invoice.Payee:\n    title: Payee\n    x-ess-name: billing.invoice.Payee\n    x-ess-kind: union\n    oneOf:\n    - title: company\n      type: object\n      properties:\n        kind:\n          type: string\n          const: company\n        value:\n          $ref: '#/x-ess-entities/billing.invoice.CompanyRef'\n      required:\n      - kind\n      - value\n      additionalProperties: false\n    - title: person\n      type: object\n      properties:\n        kind:\n          type: string\n          const: person\n        value:\n          $ref: '#/x-ess-entities/billing.invoice.Email'\n      required:\n      - kind\n      - value\n      additionalProperties: false\n    x-ess-union-tag: kind\n"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes stdout ----

thread 'the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes' (1122175) panicked at crates/generate/ess-gen/tests/relations.rs:172:5:
assertion `left == right` failed: `generated/schema/entities/billing.invoice.Account.schema.json` is not what `schema` writes for the billing example. The committed tree is what an adopter reads; regenerate it in the commit that changed the projection
  left: "{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": \"Account\",\n  \"x-ess-name\": \"billing.invoice.Account\",\n  \"x-ess-kind\": \"entity\",\n  \"type\": \"object\",\n  \"properties\": {\n    \"account_id\": {\n      \"$ref\": \"#/$defs/billing.invoice.AccountId\"\n    },\n    \"display_name\": {\n      \"type\": \"string\"\n    },\n    \"state\": {\n      \"$ref\": \"#/$defs/billing.invoice.Account.State\"\n    }\n  },\n  \"required\": [\n    \"account_id\",\n    \"display_name\",\n    \"state\"\n  ],\n  \"additionalProperties\": false,\n  \"x-ess-provenance\": {\n    \"system\": \"billing\",\n    \"specification_version\": \"v3\",\n    \"source_digest\": \"56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\",\n    \"contract_digest\": \"slice-sha256/2:c73c1fc72ab0e24b9537d2c17696c64ae7848c270d691f3d547cc66b4a54320a\",\n    \"regenerate\": \"ess generate\"\n  },\n  \"$defs\": {\n    \"billing.invoice.Account.State\": {\n      \"title\": \"State\",\n      \"x-ess-name\": \"billing.invoice.Account.State\",\n      \"x-ess-kind\": \"enum\",\n      \"type\": \"string\",\n      \"enum\": [\n        \"Active\"\n      ]\n    },\n    \"billing.invoice.AccountId\": {\n      \"title\": \"AccountId\",\n      \"x-ess-name\": \"billing.invoice.AccountId\",\n      \"x-ess-kind\": \"newtype\",\n      \"type\": \"string\",\n      \"format\": \"uuid\",\n      \"pattern\": \"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\"\n    }\n  }\n}\n"
 right: "{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": \"Account\",\n  \"x-ess-name\": \"billing.invoice.Account\",\n  \"x-ess-kind\": \"entity\",\n  \"type\": \"object\",\n  \"properties\": {\n    \"account_id\": {\n      \"$ref\": \"#/$defs/billing.invoice.AccountId\"\n    },\n    \"display_name\": {\n      \"type\": \"string\"\n    },\n    \"state\": {\n      \"$ref\": \"#/$defs/billing.invoice.Account.State\"\n    }\n  },\n  \"required\": [\n    \"account_id\",\n    \"display_name\",\n    \"state\"\n  ],\n  \"additionalProperties\": false,\n  \"x-ess-provenance\": {\n    \"system\": \"billing\",\n    \"specification_version\": \"v3\",\n    \"source_digest\": \"56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\",\n    \"contract_digest\": \"f7d09d13ef1fa3fe844ea2025e9ebc1c5b0399311a470aa9ba52baef33000b89\",\n    \"regenerate\": \"ess generate\"\n  },\n  \"$defs\": {\n    \"billing.invoice.Account.State\": {\n      \"title\": \"State\",\n      \"x-ess-name\": \"billing.invoice.Account.State\",\n      \"x-ess-kind\": \"enum\",\n      \"type\": \"string\",\n      \"enum\": [\n        \"Active\"\n      ]\n    },\n    \"billing.invoice.AccountId\": {\n      \"title\": \"AccountId\",\n      \"x-ess-name\": \"billing.invoice.AccountId\",\n      \"x-ess-kind\": \"newtype\",\n      \"type\": \"string\",\n      \"format\": \"uuid\",\n      \"pattern\": \"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$\"\n    }\n  }\n}\n"


failures:
    the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes
    the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p ess-gen --test relations`
     Running tests/schema.rs (target/debug/deps/schema-c872c2e57cbce1cf)

running 27 tests
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_schema_says_which_specification_it_came_from ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-38a9c8cb806bed12)

running 8 tests
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 9 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test a_placed_view_becomes_a_verb ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test the_binary_generates_its_own_completions ... ok
test every_placed_word_is_an_obligation ... ok
test a_string_field_offers_no_values ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test the_emission_is_deterministic ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/go.rs (target/debug/deps/go-408e408e0985c694)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 8 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/relations.rs (target/debug/deps/relations-0da2273dd262509e)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f193ff4f35681316)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test grants_are_refused_rather_than_owed ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test only_the_initial_state_can_be_constructed ... ok
test the_billing_plan_counts_are_pinned ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test the_plan_never_names_the_emission_language ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-1cb70fdbab20ef2a)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests ess_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 5 targets failed:
    `-p ess-diff --test artifacts`
    `-p ess-diff --test families`
    `-p ess-diff --test impact`
    `-p ess-gen --test corpus`
    `-p ess-gen --test relations`

```

### green-residual-order.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test families review_unclassified_transition_order
```

Exit: 0. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 3.80s
     Running tests/families.rs (target/debug/deps/families-18b80718065c44da)

running 1 test
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.32s


```

### format-apply.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt -p ess-compiler -p ess-diff -p ess-gen -p ess-synth
```

Exit: 0. Full preserved output:

```text

```

### format-apply-2.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt -p ess-compiler -p ess-diff -p ess-gen -p ess-synth
```

Exit: 0. Full preserved output:

```text

```

### format-apply-3.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt -p ess-compiler -p ess-diff -p ess-gen -p ess-synth
```

Exit: 0. Full preserved output:

```text

```

### clippy-1.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --all-targets -- -D warnings
```

Exit: 101. Full preserved output:

```text
    Checking cfg-if v1.0.4
    Checking memchr v2.8.3
    Checking itoa v1.0.18
    Checking foldhash v0.2.0
    Checking equivalent v1.0.2
    Checking allocator-api2 v0.2.21
    Checking typenum v1.20.1
    Checking dyn-clone v1.0.20
    Checking const-oid v0.10.2
    Checking ryu v1.0.23
    Checking unsafe-libyaml v0.2.11
    Checking cpufeatures v0.3.1
    Checking bitflags v2.13.1
    Checking serde_core v1.0.229
    Checking zmij v1.0.23
    Checking pulldown-cmark-escape v0.11.0
    Checking unicase v2.9.0
    Checking thiserror v2.0.20
    Checking libc v0.2.189
    Checking num-traits v0.2.19
    Checking regex-syntax v0.8.11
    Checking zerocopy v0.8.56
    Checking once_cell v1.21.4
    Checking aho-corasick v1.1.5
    Checking scopeguard v1.2.0
    Checking smallvec v1.16.0
    Checking ref-cast v1.0.27
    Checking hashbrown v0.17.1
    Checking borrow-or-share v0.2.4
    Checking bit-vec v0.8.0
    Checking percent-encoding v2.3.2
    Checking micromap v0.3.0
    Checking num-cmp v0.1.0
    Checking num-integer v0.1.47
    Checking pulldown-cmark v0.13.4
    Checking hybrid-array v0.4.14
    Checking num-complex v0.4.6
    Checking outref v0.5.2
    Checking getrandom v0.3.4
    Checking bytecount v0.6.9
    Checking parking_lot_core v0.9.12
    Checking lock_api v0.4.14
    Checking bit-set v0.8.0
    Checking vsimd v0.8.0
    Checking num-bigint v0.4.8
    Checking num-iter v0.1.46
    Checking unicode-general-category v1.1.0
    Checking strum v0.28.0
    Checking data-encoding v2.11.1
    Checking indexmap v2.14.1
    Checking unarray v0.1.4
    Checking regex-automata v0.4.18
    Checking jsonschema-regex v0.52.1
    Checking crypto-common v0.2.2
    Checking block-buffer v0.12.1
    Checking rand_core v0.9.5
    Checking ppv-lite86 v0.2.21
    Checking parking_lot v0.12.5
    Checking uuid-simd v0.8.0
    Checking num-rational v0.4.2
    Checking digest v0.11.3
    Checking num v0.4.3
    Checking fancy-regex v0.19.0
    Checking regex v1.13.1
    Checking sha2 v0.11.0
    Checking rand v0.9.5
    Checking rand_xorshift v0.4.0
    Checking fraction v0.17.0
    Checking rand_chacha v0.9.0
    Checking proptest v1.11.0
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking serde_yaml v0.9.34+deprecated
    Checking ahash v0.8.12
    Checking fluent-uri v0.4.1
    Checking email_address v0.2.9
    Checking schemars v0.8.22
    Checking jsonschema-value v0.52.1
    Checking referencing v0.52.1
    Checking jsonschema v0.52.1
    Checking ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-primitives)
    Checking ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-domain)
    Checking ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
error: this `if` statement can be collapsed
  --> crates/generate/ess-gen/src/stamp.rs:17:9
   |
17 | /         if text.starts_with("# ") && !rest.trim().is_empty() {
18 | |             if structured(rest)? != stamp {
19 | |                 return None;
20 | |             }
21 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#collapsible_if
   = note: `-D clippy::collapsible-if` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`
help: collapse nested if block
   |
17 ~         if text.starts_with("# ") && !rest.trim().is_empty()
18 ~             && structured(rest)? != stamp {
19 |                 return None;
20 ~             }
   |

error: item in documentation is missing backticks
   --> crates/generate/ess-gen/src/stamp.rs:104:5
    |
104 | /// serde_yaml's mapping visitor rejects duplicate keys, including in JSON input. JSON's generic
    |     ^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#doc_markdown
    = note: `-D clippy::doc-markdown` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::doc_markdown)]`
help: try
    |
104 - /// serde_yaml's mapping visitor rejects duplicate keys, including in JSON input. JSON's generic
104 + /// `serde_yaml`'s mapping visitor rejects duplicate keys, including in JSON input. JSON's generic
    |

error: could not compile `ess-gen` (lib) due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `ess-gen` (lib test) due to 2 previous errors

```

### clippy-2.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --all-targets -- -D warnings
```

Exit: 101. Full preserved output:

```text
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
    Checking ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
error: unnecessary hashes around raw string literal
   --> crates/generate/ess-gen/tests/provenance.rs:385:9
    |
385 | /         r#"
386 | | format: ess/1
387 | | system: probe
388 | | version: v1
...   |
415 | |         type: probe.core.Alpha
416 | | "#,
    | |__^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_raw_string_hashes
    = note: `-D clippy::needless-raw-string-hashes` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::needless_raw_string_hashes)]`
help: remove all the hashes around the string literal
    |
385 ~         r"
386 | format: ess/1
...
415 |         type: probe.core.Alpha
416 ~ ",
    |

error: could not compile `ess-gen` (test "provenance") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

```

### clippy-3.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --all-targets -- -D warnings
```

Exit: 101. Full preserved output:

```text
    Checking ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
error: these match arms have identical bodies
    --> crates/verify/ess-diff/src/change.rs:2063:13
     |
2063 | /             Self::OutcomeSetsChanged { outcome, .. }
2064 | |             | Self::OutcomeRefusesChanged { outcome, .. } => Some(outcome.clone()),
     | |__________________________________________________________________________________^
...
2071 | /             Self::OutcomeAdded { outcome }
2072 | |             | Self::OutcomeRemoved { outcome }
2073 | |             | Self::OutcomeConditionChanged { outcome, .. }
2074 | |             | Self::OutcomeSubjectChanged { outcome, .. }
...    |
2077 | |             | Self::OutcomeErrorChanged { outcome, .. }
2078 | |             | Self::OutcomeSummaryChanged { outcome, .. } => Some(outcome.clone()),
     | |__________________________________________________________________________________^
     |
     = help: if this is unintentional make the arms return different values
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#match_same_arms
     = note: `-D clippy::match-same-arms` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::match_same_arms)]`
help: otherwise merge the patterns into a single arm
     |
2063 ~             Self::InputAdded { field, .. }
2064 |             | Self::InputRemoved { field }
 ...
2068 |             | Self::InputSummaryChanged { field, .. } => Some(field.clone()),
2069 ~             Self::OutcomeSetsChanged { outcome, .. }
2070 ~             | Self::OutcomeRefusesChanged { outcome, .. } | Self::OutcomeAdded { outcome }
     |

error: adding items after statements is confusing, since items exist from the start of the scope
   --> crates/verify/ess-diff/src/delta.rs:437:9
    |
437 |         struct Changes<'a>(#[serde(serialize_with = "serialize_changes")] &'a [SemanticChange]);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#items_after_statements
    = note: `-D clippy::items-after-statements` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::items_after_statements)]`

error: this function has too many lines (105/100)
    --> crates/verify/ess-diff/src/diff.rs:1021:1
     |
1021 | / fn compare_entities(
1022 | |     was: &ResolvedEntity,
1023 | |     is: &ResolvedEntity,
1024 | |     name: &QualifiedName,
1025 | |     push: &mut impl FnMut(EntityChange),
1026 | | ) {
     | |_^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
     = note: `-D clippy::too-many-lines` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: this function has too many lines (102/100)
    --> crates/verify/ess-diff/src/diff.rs:1237:1
     |
1237 | / fn outcome_changes(
1238 | |     was: &[ResolvedOutcome],
1239 | |     is: &[ResolvedOutcome],
1240 | |     push: &mut impl FnMut(CommandChange),
1241 | | ) {
     | |_^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines

error: this function has too many lines (141/100)
    --> crates/verify/ess-diff/src/diff.rs:1673:1
     |
1673 | fn residual(ir: &EssIr) -> serde_json::Value {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines

error: could not compile `ess-diff` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `ess-diff` (lib) due to 5 previous errors

```

### generate-billing.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo run --offline --quiet --locked -p ess-cli --bin ess -- generate --format json --path examples/billing
```

Exit: 0. Full preserved output:

```text

```

### docs-gatepass.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess generate --kind docs --format json --path examples/gatepass
```

Exit: 0. Full preserved output:

```text

```

### docs-oracle.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess generate --kind docs --format json --path examples/oracle-fixture
```

Exit: 0. Full preserved output:

```text

```

### http-rust.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess synthesize --target rust --format json --path examples/gatepass
```

Exit: 0. Full preserved output:

```text

```

### http-go.log

Command (working directory: assigned tree):

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess synthesize --target go --format json --path examples/gatepass
```

Exit: 0. Full preserved output:

```text

```
