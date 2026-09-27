# Concurrent history conformance

Status: proposed, 2026-09-27. Nothing is implemented. The plan is `epic:concurrent-history-conformance`
in `.engineering/planning/` and the stories that decompose it. The history document is specified in
`models/concurrent-history/`.

## The gap

On `origin/main` `be44a3365` (0.36.0), every ESS check that executes an adopter's implementation
drives it with one client, one command at a time, and no injected fault:

| check | where | what limits it |
|---|---|---|
| synthesized conformance suite | `ess verify conform synthesize/run` | scenarios run step by step, single client |
| fault matrix | `crates/verify/ess-conformance/src/faulty.rs` | built-in fixtures only |
| specification mutation audit, 9 classes | `ess verify conform mutate` (0.34.0, #114) | `--target` is `billing`, `oracle-fixture` or `interpreted`, so an adopter's implementation cannot be audited (#153) |
| seeded random-walk explorer with shrinking | `src/ts/explore.ts` (1182 lines), `src/go/explore.go` | sequential: every command is awaited before the next (`explore.ts:717`); drops every command with an `external:` outcome (#156: 59 of 89 excluded, 3 outcomes reached); no ambient precondition (#152) |
| Rust spec interpreter | `src/interpret.rs` (152 lines) | a seam: every scenario returns `Unsupported` |
| compiler property tests, specification-surface fuzzing | `ess-compiler/tests/adversarial.rs`, `fuzz/` | test `ess`, not the adopter's software |
| semantic diff, impact | `ess verify diff`, `ess verify impact` | revision against revision |

Lost updates, events applied twice on redelivery, a retry that creates a second entity, a stale read
under a declared `read_your_writes` or `Current` view, and two transitions out of one state that
both succeed all pass every one of these.

## The feature

Run 2–4 clients against the adopter's target at once, record every call's invoke and return, and
search for a sequential order of the recorded operations that the specification's own model
accepts. Hold each view to the consistency it declares. Inject only faults the specification
declares: a second delivery for `delivery: at_least_once`, a client retry for `replays`, and a
delayed or unanswered `external:` branch. Every new bug class is proven by a planted fault that the
new check catches and no earlier check catches.

## Decisions

1. **One model, one checker, in Rust.** The Rust interpreter (`epic:model-driven-interpretation`) is
   the model. The checker and shrinking live in `ess verify conform check-history`. The Go and
   TypeScript explorers are thin clients: they drive the target, write `ess-history/1`, and call
   `ess`. Adopters already install `ess` to generate those packages. An earlier draft put a separate
   checker in the Go package so Go adopters could have it before the interpreter exists. It was
   dropped because it would have meant three copies of the checker kept in agreement by a
   differential test.
2. **No new dependency.** No crate, Go module, npm package or external tool is added to `ess` or to
   any emitted package. Published tools are read as algorithm references; no code is copied.
3. **The history is specified; the checker is not.** `models/concurrent-history/` declares
   `concurrent.history.History` and `concurrent.history.Operation`, so the three languages share
   one typed source. The checker is a search algorithm, not behaviour a caller observes, and ESS
   does not synthesize behaviour ([the linker never chooses](linker-never-chooses.md)).
4. **An unanswered operation finishes after every other one.** A timed-out call may still take
   effect, so its return instant is read as later than every other operation's. Treating it as
   failed produces false violations.
5. **`Unknown` is never a pass.** A search that exhausts its budget reports `Unknown` and exits 3.
6. **No target restart.** No specification, realization or `ConformanceTarget` method declares
   restart support, so injecting one would be an undeclared fault.

## Prior art

Web research, 2026-09-27. Versions and dates are read from the named page or registry on that
date.

| source | what it is | used for |
|---|---|---|
| [Porcupine](https://github.com/anishathalye/porcupine), Go, MIT, v1.3.1 (pkg.go.dev, 2026-09-21), 1252 stars | linearizability checker; Wing–Gong–Lowe search with P-compositionality; `NondeterministicModel`; `Ok`/`Illegal`/`Unknown`; HTML visualizer; used by etcd and S2 | the algorithm and the lane view, as reference |
| [porcupine-rs](https://lib.rs/crates/porcupine-rs) 0.3.0, MIT, 232 downloads (crates.io) | Rust port | not used: no nondeterministic models, and ESS steps are nondeterministic (external branches, generated identities, eventual views). Its release date conflicts between docs.rs (2026-09-25) and crates.io (2026-05-10) |
| [S2: linearizability testing with deterministic simulation](https://s2.dev/blog/linearizability) | turmoil + Porcupine on a production stream store | decision 4 |
| Hughes et al., PULSE (ICFP 2009); [quickcheck-state-machine](https://github.com/advancedtelematic/quickcheck-state-machine) | parallel state-machine testing: a sequential prefix, a parallel suffix, linearizability against the sequential model | the runner's shape and shrinking |
| [proptest-state-machine](https://docs.rs/proptest-state-machine) | Rust state-machine testing | sequential only; confirms the gap |
| [lincheck](https://github.com/SmnTin/lincheck) 0.2.1 (2023-08-03) | proptest + loom for in-process data structures | not applicable: the adopter's target is out of process |
| [Jepsen Elle](https://github.com/jepsen-io/elle), Clojure | black-box transactional anomaly checker | session anomaly vocabulary only; JVM |
| [AWS P and PObserve](https://queue.acm.org/detail.cfm?id=3712057); [TLA+ trace validation](https://arxiv.org/abs/2404.16075) (SEFM 2024) | checking structured production logs against a formal specification | the later recorded-history milestone |
| [Stateright](https://github.com/stateright/stateright) 0.31.0; [quint-connect](https://github.com/informalsystems/quint-connect) 0.1.2 | embedded model checker for Rust actors; Quint traces replayed against Rust | not adopted: each needs its own model language, and ESS already has its model in the IR |
| turmoil, madsim, mad-turmoil, Antithesis | deterministic simulation of a runtime | not adopted: ESS does not own the adopter's scheduler |

## Effort and gain

Ranked on 2026-09-27, before decisions 1 and 2. Effort and gain are estimates, not measurements.

| # | item | effort | gain |
|---|---|---|---|
| 1 | an unanswered operation finishes after every other one (decision 4) | very low | high |
| 2 | concurrent mode in the Go explorer with a checker | low | high |
| 3 | an HTML view of a failing history | low | medium |
| 4 | declared fault injection: duplicate delivery, client retry | medium | high |
| 5 | #152 ambient precondition and #156 external branches | medium | high |
| 6 | concurrent mode in the TypeScript explorer | medium | medium |
| 7 | `read_your_writes` and `eventual` view checks | medium | medium |
| 8 | the Rust checker, `check-history` | medium–high | medium |
| 9 | the Rust interpreter | high | high, indirect |
| 10 | validation of recorded production histories | high | high, later |
| 11 | client lanes on the ESS web page | medium | low |
| 12 | deterministic simulation of the adopter's runtime | high | low for ESS |
| 13 | TLA+/Quint/Stateright export, Elle | high | low |

Decision 1 moved items 2 and 3 behind items 8 and 9: the Go explorer no longer carries its own
checker, so its concurrent mode needs the Rust interpreter first.

## Plan

| story | depends on |
|---|---|
| `story:concurrent-history-format` | — |
| `story:linearizability-checker-over-the-interpreter` | the format; `story:interpreted-command-execution` |
| `story:concurrent-explorer-runner` | the format, the checker; `story:outcome-shapes-beyond-ess-14` (#152) |
| `story:session-and-eventual-view-checks` | the checker |
| `story:declared-fault-injection` | the runner, the session checks; `story:external-mutation-explorer-and-toolchain` (#156) |
| `story:concurrent-history-lanes` | the checker, the runner |
| `story:recorded-history-validation` (later milestone) | the checker, the session checks |

Four plan critics (acceptance, design, scope, parallel safety) reviewed the set twice; their
verdicts are the `review-result:concurrent-history-*` records.

## Not covered

- A technique row in the `ess:hardening` skill catalogue (`beyond10x/agentplugins`), once the
  runner ships.
- TLA+, Alloy or Quint export.
