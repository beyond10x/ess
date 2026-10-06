---
format: aep.planning-md/3
id: story:served-codec-name-collision
kind: story
status: implemented
title: Rust served synthesis accepts distinct commands whose codec names flatten identically
refs:
- provider: github
  reference: beyond10x/ess#415
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:43:43Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:43:44Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:44Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Rust served synthesis accepts distinct commands whose codec names flatten identically (beyond10x/ess#415).

## Origin

Opened on GitHub as beyond10x/ess#415; added to the bundle on 2026-10-04 under the operator goal that every open issue is solved and merged through the integration branch. Issue text (first part):

> A valid IR containing distinct commands `renewal.input.AB` and `renewal.input.A_B` is refused for the Rust network surface because codec identifiers flatten to the same string. Explicit code aliases distinguish payload types but do not change codec allocation. Found during #412 boundary tests; existing target refusal is preserved, no successful served support is claimed.
> 
> Owning source, unchanged at base2f554561bef25125a93a1fb1d6517d50cb24ed20:
> - `crates/generate/ess-synth/src/rust/wire.rs:53–55`: ident uses value_ident(type_fragment(canonical)).
> - `crates/generate/ess-synth/src/rust/feasibility.rs:1015–1030`: encode/decode/outcome codec inventory.
> 
> Actual structured refusal is `WireCollision` for these three identifiers:
> ```
> decode_command_renewal_input_a_b
> encode_command_renewal_input_a_b
> encode_outcome_renewal_input_a_b
> ```
> Each is allocated twice for the two distinct canonical commands. The #412 fixture additionally includes AB2 and code aliases, so a repair must handle suffix occupation deterministically. Validated compiled IR digestfac40dca6e1ba406f8bb3bba13246f0cabc00504f07aed21fb71e7a57b3a9f6d, contractdigesta5caccc38deb594d792bb5f6907bcc33bad058cf84cc8b1caf2f97fcae1f82ad.
> 
> Reproducer: `collision_alias_and_input_local_contracts_compile_in_both_targets` in #412 owned branch external_request_binding.rs SHA25659be63088f38a633e423011ec135ebabeb296a19267c579acf3eccfe7383b537. Actual log SHA2565e5c4444bc5c9559e66e8c160a934f490ae132be42cbea4dd9781d7e938c8350; 0passed2fai
