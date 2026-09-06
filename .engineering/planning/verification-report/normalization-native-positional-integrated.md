---
format: aep.planning-md/1
id: verification-report:normalization-native-positional-integrated
kind: verification-report
status: draft
title: Integrated native codecs and positional normalization verification
relations:
- verifies: story:binary64-structural-codecs
- verifies: story:normalize-positional-array-input
- verifies: story:normalization-followup-publication
revision: 1
---
## Outcome

Native model Binary64 codecs and normalization recipe6 are integrated with independent tests and shared public guidance. Structural source291f229256ce4fa78a17a01b085f56a9e5ab6990 plus independent testsb809e43dc492b8817fc5a860b0948e90bdab8649 merged at213d4b8a8338763346cad2bc92cee826430726c9. Positional sourceeb2e5d60e9e803993417df39563bc744dbcd36fc plus independent tests3b500c7fcda959d99eccac402f7c8aa99311f8da merged at6601c849f69b2a791f7795921e346a4f3b6b220f. Final source and documentation commit1f8e319cf153c348a6c434c6e74939f4aa587125 passed the complete workspace and site gates below.

## Independent reviews

- review-result:binary64-structural-codecs-adversary-pass1: exact public body SHA-256655cd7c22e2d87ef622d5463d511af41c3b76fed25121b67acc820f4a2a7a91d,12→16 focused cases, zero red and no product findings. Root verified all969 original tracked hashes/modes and exactly four added tests. Six separately selected native Rust/Go cases passed. Outcome no-op.
- review-result:normalization-positional-adversary-pass1: exact public body SHA-256ac0fb35bc0bdba0ece1f2102f9d77c460dacdb1d74d2f0d6d65f9f3db1190e67,30→38 focused outer cases, no product findings. Root verified all979 tracked hashes/modes and exactly four added tests. Nineteen independently authored literal vectors ran in generated Rust with both number modes and Go with race checking, plus metadata, UTF-8, retained-input and CLI controls. Outcome no-op.
- review-result:normalization-followup-publication-docs-pass1: exact public body SHA-256fb2cb9361cfc4f61a826d3bc9496e0b415ae15a34f052b5f48541e443ffbf992. Its one introduced API-wording warning was corrected by naming reference Plan, generated Rust/Go and CLI text entrypoints. Outcome fixed; this review ran no native tests.

All public review bodies preserve their declared path-only normalization and retain exact private companions and raw logs. Each immutable report was recorded before its outcome/correction. Initial test-harness compilation/fixture/style errors remain in those reports; no production correction is attributed to them. Positional's attempted cross-stage re-decoding fixture was refused during admission, and the corrected test asserts that reachable refusal. No assertion, frozen source or legacy byte witness was relaxed.

The coordinator's first added-file mode comparison used a Git-mode string against a filesystem integer; correcting that scratch verifier, without source changes, passed all979+4 comparisons. Earlier AEP writes inherited the configured human actor. They were performed by the coordinator agent and do not represent human review or approval; subsequent writes explicitly select agent:normalization-coordinator. Existing journals and immutable records were not rewritten.

## Qualification and compatibility

Structural implementation qualification reported127→137 selected cases, with nineteen literal bit vectors and native wire/layout controls. Positional default package qualification rose116→130 and CLI15→17; native Rust default and arbitrary-precision each executed99 positional,3 mixed and70 Binary64 vectors, while Go executed93 positional,3 mixed and70 Binary64 vectors. Six decoded-value API cases are explicitly unavailable in Go. These are scoped observations with their own baselines, not quantities to add to the workspace count.

Complete same-generator format-1 through format-5 Rust/Go maps retain all173 raw file bytes, including the capture index. Ten frozen runtime templates remain exact. Existing non-Binary64 structural maps retain their digest02312f45fadac5e16b54aa1fce13d8f68cd8aaf14336a2180b85cd923811c79f. Generator-version fields and declaration headers remain truthful; no cross-release byte equality is implied. A future actual version bump must separately qualify only those producer slots and preserve these raw baselines.

## Integrated gates

| Command | Exit | Seconds | Finished UTC |
| --- | --- | --- | --- |
| task check | 0 | 149.710 | 2026-09-06T13:19:25.977027+00:00 |
| task site-build | 0 | 16.439 | 2026-09-06T13:19:42.417443+00:00 |

The literal workspace gate passed1,980 Rust cases across170 result groups, with zero failed or ignored. ESS_GO_COMPILER was absent, matching ordinary CI discovery. Formatting, strict workspace Clippy, tests, rustdoc, examples, generated projection/schema drift, release consistency and action checks completed. The site gate executed the WASM/browser-lab checks and production documentation build. Existing npm audit/install-script notices were not represented as a clean dependency security audit.

## Resource and delivery boundary

Both implementors and both adversaries released the serial compiler lane with all processes terminal. All unit verification files are retained in assigned recovery scratch or committed test sources. Harness token/tool-count telemetry was unavailable; no fabricated cost is supplied. The initial source scopes were confirmed through AEP, retaining their historical inferred classifications and explicit corrections.

The combined gate reduced free filesystem space to about10.04GB. Before the next unit receives its6GiB allowance above an8GiB reserve, the coordinator will archive wanted ignored verification artifacts and retire only the four completed, remotely reachable ESS unit trees through the worktree manager. Integration and current authority trees remain needed for the ongoing sequence. This is resource-driven cleanup of completed published units, not a claim that final cleanup is finished.

This checkpoint is main-source publication, not a new binary release or completed adopter cutover. Released ESS remains0.19.0. TypeScript normalization, final released-tool adoption and the remaining explicitly source-bound IVR mapping are still pending. Whole-system Binary64 synthesis/conformance and unsupported structural refinements retain their documented boundaries.
