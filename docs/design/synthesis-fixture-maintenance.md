# Maintaining synthesis fixtures

The checked-in synthesis snapshots are produced by `ess generate synthesize`. Run it from the
repository root, using the candidate's CLI (`cargo run --locked --bin ess -- ...`):

| Specification | Target | Output |
|---|---|---|
| `examples/billing` | `rust` | `generated/rust/billing` |
| `examples/gatepass` | `rust` | `generated/rust/gatepass` |
| `examples/billing` | `go` | `generated/go/billing` |
| `examples/gatepass` | `go` | `generated/go/gatepass` |
| `examples/billing` | `web` | `generated/web/billing` |

For example, after enrollment:

```console
cargo run --locked --bin ess -- generate synthesize --path examples/billing --target rust --out generated/rust/billing
```

A fresh checkout has the artifact payloads but no local ownership ledger. If generation reports
an unowned destination, first generate a reference into a **new directory** from a managed checkout
of the revision that produced the existing snapshots. The reference must contain the generator's
settled `.ess-output` state, and its payload must match the existing files exactly. In the working
candidate, enroll that reference before regenerating:

```console
cargo run --locked --bin ess -- generate output adopt --ownership-root generated/rust/billing --from <settled-reference-directory> --owner synthesis
```

Repeat for each output tree. A mismatching reference is refused; establish the correct source
revision rather than replacing the destination by copying files. Keep the reference while the
refresh is being verified. The old `cargo xtask synth` command is no longer available.

The local `.ess-output` directories are ignored by Git. The feasibility test excludes only that
ordinary directory at each snapshot root; every artifact path and byte still must match synthesis.
Run the affected `ess-synth` tests and the repository projection checks before publishing a refresh.
