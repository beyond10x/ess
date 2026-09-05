# Review boundaries wave 3 — 2026-09-05

**Approved under standing authorization; implementing.** This wave restores persisted delivery validation, makes TypeScript declarations feasible in one namespace, and writes the binding conformance migration design. Each story serves vision:O2.

Skill version 0.7.0. Integration branch wave/review-boundaries-3, coordinator managed record wt-752828a285ba. Published base 98ea8abeeaf80846f525b5def8b531c139ed7071.

## Authorization and selection

This is an interactive session with standing implementation and publication authorization in approval-record:review-remediation-standing-implementation and approval-record:review-remediation-standing-publication. The user also explicitly requested managed cleanup. No approval bypass is inferred and no further permission is needed for these authorized waves. Release versions/tags remain separately authorized.

Authorized commits: opening scope/page/store commit; three unit implementations and their review tests/corrections where required; serial integration merges; closing store/evidence commit; merge to main; publication of the green wave and cleanup records. No unrelated work is granted by this selection.

The store was read afresh after wave 2 closed. All three selected stories have no dependencies or blockers. Independent scoping is recorded in their bodies and typed entries. The full draft/proposed computation below preserves every returned wave, collision, unassessed entry and cycle. The selected proposed set is one computed wave with no collision, unassessed entry or cycle.

Prioritize P0 persisted delivery documents reaching analysis/execution. TypeScript naming is an independent single-package defect with a real local compiler available. The design-only conformance unit resolves decisions needed before its two dependent implementation stories can migrate counts/coverage. The latter's new file path is inferred, but its reserved document-only edit surface is high-confidence. TypeScript's package/source scope is cited and its new compiler-test lane is inferred. Delivery owns two cited packages because the read invariant must survive the CLI boundary.

InfraIr validation competes for the CLI package. Semantic diff needs the version/provenance decision and Atlas coordination recorded in its refined scope; it is deferred from this set. Other ready stories are left for the next fresh selection. No design completion alone closes its dependent implementation findings or the full review.

Native aep-drive:story-scoper, aep-drive:implementor and aep-drive:adversary types are unavailable in this harness. Collaboration threads use their exact plugin charters with inherited model and no override. Completed threads must be reused because the four-thread cap counts them. The prior containment review's two automatic dispatch rejections and local completion are recorded in wave 2; each new unit retains its required review and no green outcome is assumed.

## Preflight and resources

Primary ESS is clean on main and matches advertised remote 98ea8abeeaf80846f525b5def8b531c139ed7071. Both previous unit worktrees, build directories and unit branches are gone; only their archived raw evidence remains under the reusable coordinator target. Unrelated review/feature/release trees remain untouched. The original architecture review/outlook worktree and PDF are preserved.

Free capacity measured 141,610,594,304 bytes; the hard reserve is 8,589,934,592 bytes. Current integration target is 1,890,048 KiB and website/node_modules is 661,020 KiB. Prior measured units were 183,580 KiB for diagnostics and 1,108,376 KiB for the CLI/generator unit. N=3 is a coordinator resource estimate: two Rust implementations plus one document-only unit fit comfortably at this capacity, within three worker slots and the default model budget of four. A numeric budget was previously requested but not supplied. Token counts and complete agent duration are unavailable from this harness.

Each tree keeps its own target and uses /usr/bin/sccache, CARGO_INCREMENTAL=0 and dev/test debug=0; CARGO_TARGET_DIR is forbidden. Logs/reports use target/review-boundaries-3, while the additionally assigned temporary-fixture root is target itself to keep compiler socket paths short. CARGO_CACHE_RUSTC_INFO=0 prevents replay of a cached failed compiler metadata query observed in wave 2. No shared target or new external scratch is planned. Recheck disk at dispatch and return.

At dispatch, the long delivery-unit temporary path prevented sccache from creating its startup notification socket. The coordinator started a dedicated foreground server using the documented SCCACHE_START_SERVER=1 and SCCACHE_NO_DAEMON=1 mode. Its explicit socket is /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock. Rust clients add SCCACHE_SERVER_UDS pointing there; each retains its own target and TMPDIR. The server was queried successfully before routing this supplement. Its log is target/review-boundaries-3/cache-server.log in the coordinator, which owns shutdown after all wave gates. This shares only compiler-cache access and bypasses no gate. The initial startup failure executed no delivery tests.

Current Atlas authority is managed wt-90ec680c6073 at clean exact remote 7b00adf3b1004e0cdd8dd12aa4fa8cc8435a0432. Its last organization fence is red on separately recorded sibling issues; this wave changes no Atlas source or public documentation allowlist. Bot commits and publishing use the current verified authority.


## Cache idle restart

The original foreground cache session63151 was observed complete with exit0 after its default idle timeout. Its stale socket was present but had no listener; the first delivery-adversary isolated attempt executed zero tests and failed at rustc -vV startup. Coordinator restarted the same exact socket in foreground session44216 with SCCACHE_IDLE_TIMEOUT=0, keeping all previous resource restrictions. The new server answered --show-stats with exit0 before review resumed. Its log is coordinator target/review-boundaries-3/cache-server-restart.log. No default server was stopped, no cache was purged and no target is shared. Coordinator will stop this exact task-owned socket and observe foreground termination after the final gates.

## Unit records

Worktree root: /home/timo/.local/state/worktree/trees/b10x/ess. Every unit branches from the same opening commit and owns its recorded triple.

| Story | Branch | Managed worktree under root | Build/temp | Logs/reports | Stage |
| --- | --- | --- | --- | --- | --- |
| review-persisted-delivery-validation | impl/review-persisted-delivery-validation | review-persisted-delivery-validation | target/ | target/review-boundaries-3/ | implementation returned; reviewed; merged at 170fdfa1f3061af33f4de558d22de4711ab6194d |
| review-typescript-root-collision | impl/review-typescript-root-collision | review-typescript-root-collision | target/ | target/review-boundaries-3/ | reviewed; merged at 0d267e25739ca495ad1a229393181ad1b75182f3 |
| review-conformance-format-design | impl/review-conformance-format-design | review-conformance-format-design | target/ | target/review-boundaries-3/ | corrected and personally verified at 6acd811a8f53a3e5ab56ec7a68a13fd0a727ba2b; integration pending |

All units were provisioned at exact opening commit 45832cc885377b2d61845ee33af14f0293d99e67, branched as above, and assigned active leases <story>-wave3. Full briefs and the compiler-cache resource supplement live in each assigned scratch root. Implementor threads are impl_containment (delivery), impl_diagnostic (TypeScript), and scope_conformance_design (design).

File-backed briefs bind exact base, scope, acceptance, gates and reporting. Coordinator owns all AEP mutations, commits, shared files, gate evidence, publication and lifecycle. Implementation agents leave uncommitted assigned files. Reviews preserve original assertions and reports; maximum two full passes per unit. No story moves terminal before the complete integrated gate.

Delivery package checks cover ess-deployment and ess-cli, with real compiler-generated valid fixtures and local fake executors only. TypeScript checks cover schema-contract plus an explicit compiler lane using the installed TypeScript 6.0.3; a selected lane must fail if the compiler is unavailable, and no default-CI coverage claim is made until its wiring is established. Conformance design uses source-backed producer/reader and migration matrices; it creates no new writer, makes no compatibility execution claim, and needs no meaningless prose test. The integrated coordinator runs all eight offline gate steps and site build, plus any selected compiler-specific lane.

## Implementation handoffs

TypeScript implementation 7a73ce9bf741a98f82c7141f4a80340451a704ed is bot-authored and clean before review. Default package cases rose 9 to 16; the separately selected real TypeScript 6.0.3 lane executes 3. Both pass, as do package formatting and default/feature Clippy. Four unique new Rust cases were first observed red; the compiler exposed additional strict-module keywords after the initial mechanism. Missing compiler selects and fails all three cases. The new successful-output fixture passed on unchanged production first. Full original implementation report and patch are preserved under docs/reviews/2026-09-05-review-boundaries-3-typescript-implementation.*. Scope stays in schema-contract; inferred manifest/test paths are now cited from the diff. The design implementor thread is assigned its test-only adversary pass through a file-backed brief.

Conformance design implementation 5148543b57c855cb4ccca92e2368e566801e9c36 is bot-authored and clean before review. Its 246-line design includes 36 source-backed matrix rows; no executable compatibility case was run or claimed. Original report and patch are preserved under docs/reviews/2026-09-05-review-boundaries-3-conformance-design-implementation.*. The only tracked edit is the reserved design file, now cited in scope. The first review brief includes a coordinator-observed impact-format citation mismatch to check; the document has not passed review.

TypeScript adversary pass 1 is preserved verbatim in review-result:review-boundaries-3-typescript-adversary-pass-1. Six cases were authored before isolated execution and all passed: default package 16 to 19, real compiler lane 3 to 6. Package formatting and feature-enabled strict Clippy pass. The report finds nothing and contains an explicit empty findings list. Its test-only changes are committed on the unit, preserving every original assertion; its original test patch is in docs/reviews/2026-09-05-review-boundaries-3-typescript-adversary-tests.patch.

Delivery implementation f1baa9051be7d6cfc48ec1dcd302d0c87ac21a15 is bot-authored and clean before review. Its default package totals rose 53 to 67 (deployment 7 to 18, CLI 46 to 49); both test, format and strict Clippy package gates pass. Meaningful initial cases executed 11 with 9 failures; compiler/setup failures are explicitly separate. The full original report and patch are preserved under docs/reviews/2026-09-05-review-boundaries-3-delivery-implementation.*. Its first adversary brief is ready.

Conformance design adversary pass1 is preserved verbatim in review-result:review-boundaries-3-conformance-design-adversary-pass-1. It found two introduced blockers and one introduced warning: refusal/scenario coexistence, undefined suite5/report1 pairing, and an incorrect current impact label. Original implementor corrected all three at dc746cfab0e628ad28911cbee32c1031a9efb6dd; the sole tracked design now has 58 matrix rows and 10 pairing rows. The correction report and patch are preserved separately under docs/reviews/2026-09-05-review-boundaries-3-conformance-design-correction-1.*. A review_outcome records fixed; a different nonimplementor thread is running the second and last full source-backed design attack. No runtime compatibility results are claimed.

Delivery adversary pass 1 is preserved verbatim in review-result:review-boundaries-3-delivery-adversary-pass-1. Six new cases passed in isolation; package totals rose 67 to 73, deployment 21 plus CLI 52. Both package formatting and strict Clippy pass. Nothing was found; no second attack is justified by an unresolved concern. The 209 added test lines preserve existing assertions and are bot-committed at 4289e3eb636a97b73f8261dd6d68f3027afe9f65; the original patch is preserved in docs/reviews/2026-09-05-review-boundaries-3-delivery-adversary-tests.patch. The cache startup attempt executed zero cases and is recorded separately.

No handoff is an integrated acceptance or completed story. The coordinator also records read-only preparation for the future infrastructure and Rust feasibility stories; these draft refinements authorize no additional implementation in this wave.

## Full draft computation

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:create-only-command-cannot-refuse",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-diff"
            },
            {
              "confidence": "inferred",
              "path": "docs/design"
            }
          ]
        },
        {
          "id": "story:fuzz-the-specification-surface",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "fuzz"
            }
          ]
        },
        {
          "id": "story:review-composition-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-composition"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-conformance-format-design",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-conformance-coverage.md"
            }
          ]
        },
        {
          "id": "story:review-execution-recovery-design",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-execution-recovery.md"
            }
          ]
        },
        {
          "id": "story:review-infra-ir-invariants",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-analyze"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-compiler"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-project"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-spec"
            }
          ]
        },
        {
          "id": "story:review-typescript-root-collision",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/Cargo.toml"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/typescript.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/tests/typescript_typecheck.rs"
            }
          ]
        }
      ]
    },
    {
      "wave": 2,
      "artifacts": [
        {
          "id": "story:java-conformance-target",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:review-format-catalog",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-observation-completeness",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/infra/ess-kubernetes"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-analyze"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-domain"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-project"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-spec"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-observation-completeness.md"
            }
          ]
        },
        {
          "id": "story:review-rust-target-feasibility",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            }
          ]
        },
        {
          "id": "story:review-semantic-diff-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-diff"
            },
            {
              "confidence": "cited",
              "path": "docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md"
            },
            {
              "confidence": "inferred",
              "path": "generated/asyncapi"
            },
            {
              "confidence": "inferred",
              "path": "generated/docs"
            },
            {
              "confidence": "inferred",
              "path": "generated/openapi"
            },
            {
              "confidence": "inferred",
              "path": "generated/schema"
            },
            {
              "confidence": "inferred",
              "path": "generated/site"
            }
          ]
        }
      ]
    },
    {
      "wave": 3,
      "artifacts": [
        {
          "id": "story:review-browser-replay-fidelity",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-replay-subset.md"
            }
          ]
        },
        {
          "id": "story:review-consumer-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-consumer-coverage.md"
            }
          ]
        },
        {
          "id": "story:review-glossary-boundaries",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-concept-boundaries.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-openapi-semantic-accounting",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-openapi"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-openapi-accounting.md"
            }
          ]
        },
        {
          "id": "story:review-typed-diagnostics",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-typed-diagnostics.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 4,
      "artifacts": [
        {
          "id": "story:review-expression-typechecking",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:review-output-ownership",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-output-ownership.md"
            }
          ]
        },
        {
          "id": "story:review-public-support-claims",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "cited",
              "path": "website/docs"
            }
          ]
        }
      ]
    },
    {
      "wave": 5,
      "artifacts": [
        {
          "id": "story:review-persisted-delivery-validation",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment"
            }
          ]
        },
        {
          "id": "story:review-schema-resource-identity",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-schema-resource-identity.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:the-generated-go-runtime-is-gofmt-clean",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        }
      ]
    },
    {
      "wave": 6,
      "artifacts": [
        {
          "id": "story:review-primitive-semantics",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-primitives"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-primitive-semantics.md"
            }
          ]
        },
        {
          "id": "story:scenarios-directory-compiles-nothing",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            }
          ]
        }
      ]
    },
    {
      "wave": 7,
      "artifacts": [
        {
          "id": "story:a-skipped-scenario-is-not-a-failed-one",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "inferred",
              "path": "website/docs/guides/verify-conformance.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 8,
      "artifacts": [
        {
          "id": "story:review-authored-discovery",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-authored-discovery.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 9,
      "artifacts": [
        {
          "id": "story:review-cache-origin",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            }
          ]
        }
      ]
    },
    {
      "wave": 10,
      "artifacts": [
        {
          "id": "story:review-delivery-trust-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": ".github/actions/release-component"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-delivery-trust.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        }
      ]
    },
    {
      "wave": 11,
      "artifacts": [
        {
          "id": "story:review-conformance-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-conformance-coverage.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:create-only-command-cannot-refuse",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:create-only-command-cannot-refuse",
      "path": "docs/design",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:java-conformance-target",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:java-conformance-target",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-authored-discovery",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-composition-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:java-conformance-target",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/specify/ess-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/specify/ess-domain",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-gen",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/generate/ess-gen",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/verify/ess-diff",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-authored-discovery",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-conformance-format-design",
      "path": "docs/design/review-conformance-coverage.md",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:review-public-support-claims",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:review-public-support-claims",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/generate/ess-deployment",
      "confidence": "cited"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/specify/ess-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-glossary-boundaries",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-glossary-boundaries",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-analyze",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-project",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-spec",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-output-ownership",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-ownership",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-persisted-delivery-validation",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/generate/ess-synth",
      "confidence": "cited"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/generate/ess-gen",
      "confidence": "inferred"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-semantic-diff-coverage",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    }
  ],
  "unassessed": [],
  "cycles": []
}
```

## Full proposed computation before selection

```json
{
  "waves": [],
  "collisions": [],
  "unassessed": [],
  "cycles": []
}
```

## Selected proposed computation

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:review-conformance-format-design",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-conformance-coverage.md"
            }
          ]
        },
        {
          "id": "story:review-persisted-delivery-validation",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment"
            }
          ]
        },
        {
          "id": "story:review-typescript-root-collision",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/Cargo.toml"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract/src/typescript.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/schema-contract/tests/typescript_typecheck.rs"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [],
  "unassessed": [],
  "cycles": []
}
```

## Conformance final review and bounded correction

The immutable second pass is review-result:review-boundaries-3-conformance-design-adversary-pass-2 at dc746cfab0e628ad28911cbee32c1031a9efb6dd. Findings fell from3 to2: carried0, new2, resolved3. The new source-backed issues are identical refusal multiplicity and unsigned/signed timestamp ambiguity; no earlier correction regressed. Under standing authorization the coordinator bound a sorted refusal list preserving every occurrence and exact u64 timestamps for new report/run contracts with checked adapters and frozen legacy behavior. Original implementor receives only these correction classes. Coordinator will personally inspect the diff and preservation of earlier assertions; no third full attack. These are document-only findings, with no runtime case claim.

Exact CLI comparison:

```json
{
  "artifact": "story:review-conformance-format-design",
  "reviews": 10,
  "from": "review-result:review-boundaries-3-conformance-design-adversary-pass-1",
  "from_reviewer": "unattributed",
  "to": "review-result:review-boundaries-3-conformance-design-adversary-pass-2",
  "to_reviewer": "unattributed",
  "carried": [],
  "new": [
    {
      "file": "docs/design/review-conformance-coverage.md",
      "line": 116,
      "category": "acceptance",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "The design forbids duplicate refusal records and omitted refusals without defining how to preserve repeated identical refusals that current generated and authored producers emit."
    },
    {
      "file": "docs/design/review-conformance-coverage.md",
      "line": 167,
      "category": "contract-drift",
      "severity": "warning",
      "verdict": "CONFIRMED",
      "origin": "introduced",
      "message": "The report timestamp is described as an existing signed value although Rust uses u64 and Go uses int64, while the shared new writer rules require unsigned integer fields, leaving the v2 timestamp range and conversion contract contradictory."
    }
  ],
  "resolved": [
    {
      "file": "docs/design/review-conformance-coverage.md",
      "line": 95,
      "category": "acceptance",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "The refusal-ID disjointness rule cannot preserve existing synthesis results that retain a runnable scenario beside a refusal for an unimplemented check, or an accepted authored scenario beside a duplicate-source refusal."
    },
    {
      "file": "docs/design/review-conformance-coverage.md",
      "line": 167,
      "category": "boundary",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "Independent suite-v5 and report-v1 selections have no specified writer outcome even though the frozen report-v1 reader rejects suite major 5, leaving the advertised opt-in rollout without a complete version-pairing contract."
    },
    {
      "file": "docs/design/review-conformance-coverage.md",
      "line": 31,
      "category": "contract-drift",
      "severity": "warning",
      "verdict": "CONFIRMED",
      "origin": "introduced",
      "message": "The impact compatibility rules name impact/1 although the cited ESS baseline writes ess-impact/2, so the frozen contract and its migration target are identified incorrectly."
    }
  ]
}
```

## Personal verification after the final design correction

Original implementor committed the bounded correction at6acd811a8f53a3e5ab56ec7a68a13fd0a727ba2b with exact bot author and committer. Coordinator read the full bounded diff and affected timestamp/coverage sections against the source-backed findings. No assertion was dropped or relaxed: a direct comparison verified all68 prior matrix/pairing rows unchanged, with17 new rows. Final design has75 matrix rows and10 pairing rows. Repeated producer refusals now retain multiplicity without extra execution results; new timestamp wire/adaptation rules are exact u64 with checked conversion and frozen legacy behavior. The second review has a fixed outcome, and its exact correction SHA is recorded here. This is personal bounded verification, not a third full adversary pass or runtime compatibility evidence. Both correction reports and patches are preserved separately.
