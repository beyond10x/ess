---
title: Write a specification
sidebar_position: 1
description: Author an ESS document — the layout, the constructs the model insists on, and the validation errors that teach the model fastest.
---

# Write a specification

This guide covers authoring an Executable System Specification. The normative example is
`examples/billing/` in the repository — deliberately the smallest system that exercises the
core of the model. Concepts are covered in [ESS](../concepts/ess.md); these pages are about writing
one.

## Pages in this guide

1. [Lay out and validate a specification](specify/layout-and-validation.md) — the files, where
   generated output goes, reading refusals, checking what resolved, and names.
2. [Refusals, fields and invariants](specify/fields-and-invariants.md) — what the model insists on:
   declared outcomes, invariants, enums, instants, text, JSON and Binary64 fields.
3. [Guards and outcome selection](specify/guards-and-predicates.md) — choosing an outcome from input,
   held state or stored fields.
4. [Commands and outcomes](specify/commands-and-outcomes.md) — wrong-state answers, unknown
   instances, existence, filters, deletion and event value sources.
5. [Values, credentials and views](specify/values-and-views.md) — value expressions, the caller's
   credential, and view consistency, paging and aggregates.
6. [Components and bindings](specify/bindings-and-components.md) — the layers above the domains
   and bindings between components.
7. [Conversions and wire names](specify/wire-names.md) — conversions between contexts, wire
   spellings and error wire codes.

## Where each section went

This guide used to be a single page. Each of its sections is listed here under its old anchor, so
an older link still finds it.

- <a id="layout"></a>[Layout](specify/layout-and-validation.md#layout)
- <a id="keep-sources-and-generated-output-together"></a>[Keep sources and generated output together](specify/layout-and-validation.md#keep-sources-and-generated-output-together)
- <a id="name-the-ess-release-the-specification-is-maintained-with"></a>[Name the `ess` release the specification is maintained with](specify/layout-and-validation.md#name-the-ess-release-the-specification-is-maintained-with)
- <a id="validate-early-read-the-refusals"></a>[Validate early, read the refusals](specify/layout-and-validation.md#validate-early-read-the-refusals)
- <a id="what-the-model-insists-on"></a>[What the model insists on](specify/fields-and-invariants.md#what-the-model-insists-on)
- <a id="a-command-that-can-be-refused-says-so"></a>[A command that can be refused says so](specify/fields-and-invariants.md#a-command-that-can-be-refused-says-so)
- <a id="an-invariant-reads-only-what-every-creation-sets"></a>[An invariant reads only what every creation sets](specify/fields-and-invariants.md#an-invariant-reads-only-what-every-creation-sets)
- <a id="cover-every-declared-enum-value"></a>[Cover every declared enum value](specify/fields-and-invariants.md#cover-every-declared-enum-value)
- <a id="order-two-instants"></a>[Order two instants](specify/fields-and-invariants.md#order-two-instants)
- <a id="say-which-characters-a-text-may-hold-and-how-long-it-may-be"></a>[Say which characters a text may hold, and how long it may be](specify/fields-and-invariants.md#say-which-characters-a-text-may-hold-and-how-long-it-may-be)
- <a id="say-what-a-text-starts-with"></a>[Say what a text starts with](specify/fields-and-invariants.md#say-what-a-text-starts-with)
- <a id="carry-any-json-value"></a>[Carry any JSON value](specify/fields-and-invariants.md#carry-any-json-value)
- <a id="carry-finite-binary-floating-point-values"></a>[Carry finite binary floating-point values](specify/fields-and-invariants.md#carry-finite-binary-floating-point-values)
- <a id="select-an-outcome-from-the-held-subject-state"></a>[Select an outcome from the held subject state](specify/guards-and-predicates.md#select-an-outcome-from-the-held-subject-state)
- <a id="guard-an-outcome-by-the-subjects-stored-fields"></a>[Guard an outcome by the subject's stored fields](specify/guards-and-predicates.md#guard-an-outcome-by-the-subjects-stored-fields)
- <a id="an-outcome-the-input-cannot-decide-says-that-too"></a>[An outcome the input cannot decide says that too](specify/guards-and-predicates.md#an-outcome-the-input-cannot-decide-says-that-too)
- <a id="one-outcome-for-many-commands"></a>[One outcome for many commands](specify/guards-and-predicates.md#one-outcome-for-many-commands)
- <a id="illegal-lifecycle-moves-are-illegal-by-absence"></a>[Illegal lifecycle moves are illegal by absence](specify/guards-and-predicates.md#illegal-lifecycle-moves-are-illegal-by-absence)
- <a id="a-command-says-what-it-answers-when-invoked-in-the-wrong-state"></a>[A command says what it answers when invoked in the wrong state](specify/commands-and-outcomes.md#a-command-says-what-it-answers-when-invoked-in-the-wrong-state)
- <a id="an-unknown-instance-can-have-its-own-outcome"></a>[An unknown instance can have its own outcome](specify/commands-and-outcomes.md#an-unknown-instance-can-have-its-own-outcome)
- <a id="a-request-with-no-input-can-have-its-own-outcome"></a>[A request with no input can have its own outcome](specify/commands-and-outcomes.md#a-request-with-no-input-can-have-its-own-outcome)
- <a id="an-outcome-can-be-selected-by-whether-the-record-exists"></a>[An outcome can be selected by whether the record exists](specify/commands-and-outcomes.md#an-outcome-can-be-selected-by-whether-the-record-exists)
- <a id="an-outcome-can-change-every-record-a-filter-selects"></a>[An outcome can change every record a filter selects](specify/commands-and-outcomes.md#an-outcome-can-change-every-record-a-filter-selects)
- <a id="an-outcome-can-delete-its-subject"></a>[An outcome can delete its subject](specify/commands-and-outcomes.md#an-outcome-can-delete-its-subject)
- <a id="a-creation-can-land-in-a-declared-state"></a>[A creation can land in a declared state](specify/commands-and-outcomes.md#a-creation-can-land-in-a-declared-state)
- <a id="an-accepted-request-can-change-nothing"></a>[An accepted request can change nothing](specify/commands-and-outcomes.md#an-accepted-request-can-change-nothing)
- <a id="a-system-can-run-inside-ambient-preconditions"></a>[A system can run inside ambient preconditions](specify/commands-and-outcomes.md#a-system-can-run-inside-ambient-preconditions)
- <a id="an-events-values-need-a-declared-source"></a>[An event's values need a declared source](specify/commands-and-outcomes.md#an-events-values-need-a-declared-source)
- <a id="value-expressions"></a>[Value expressions](specify/values-and-views.md#value-expressions)
- <a id="read-the-callers-credential"></a>[Read the caller's credential](specify/values-and-views.md#read-the-callers-credential)
- <a id="an-input-refused-when-absent-is-present-afterwards"></a>[An input refused when absent is present afterwards](specify/values-and-views.md#an-input-refused-when-absent-is-present-afterwards)
- <a id="a-view-declares-its-consistency"></a>[A view declares its consistency](specify/values-and-views.md#a-view-declares-its-consistency)
- <a id="a-view-can-be-paged"></a>[A view can be paged](specify/values-and-views.md#a-view-can-be-paged)
- <a id="aggregate-views"></a>[Aggregate views](specify/values-and-views.md#aggregate-views)
- <a id="a-binding-says-what-happens-when-it-fails"></a>[A binding says what happens when it fails](specify/bindings-and-components.md#a-binding-says-what-happens-when-it-fails)
- <a id="bound-a-retry"></a>[Bound a retry](specify/bindings-and-components.md#bound-a-retry)
- <a id="read-a-field-inside-an-event-envelope"></a>[Read a field inside an event envelope](specify/bindings-and-components.md#read-a-field-inside-an-event-envelope)
- <a id="select-ordered-records-in-a-binding"></a>[Select ordered records in a binding](specify/bindings-and-components.md#select-ordered-records-in-a-binding)
- <a id="declare-a-periodic-host-cause"></a>[Declare a periodic host cause](specify/bindings-and-components.md#declare-a-periodic-host-cause)
- <a id="read-the-channel-an-event-arrived-on"></a>[Read the channel an event arrived on](specify/bindings-and-components.md#read-the-channel-an-event-arrived-on)
- <a id="preserve-clock-reading-provenance"></a>[Preserve clock-reading provenance](specify/bindings-and-components.md#preserve-clock-reading-provenance)
- <a id="crossing-contexts-takes-a-declared-conversion"></a>[Crossing contexts takes a declared conversion](specify/wire-names.md#crossing-contexts-takes-a-declared-conversion)
- <a id="an-enum-variant-can-carry-its-own-wire-spelling"></a>[An enum variant can carry its own wire spelling](specify/wire-names.md#an-enum-variant-can-carry-its-own-wire-spelling)
- <a id="a-field-can-carry-its-own-wire-name"></a>[A field can carry its own wire name](specify/wire-names.md#a-field-can-carry-its-own-wire-name)
- <a id="say-whether-an-absent-optional-is-sent-as-null"></a>[Say whether an absent Optional is sent as null](specify/wire-names.md#say-whether-an-absent-optional-is-sent-as-null)
- <a id="three-layers-above-the-domains"></a>[Three layers above the domains](specify/bindings-and-components.md#three-layers-above-the-domains)
- <a id="check-what-you-just-wrote-resolved"></a>[Check what you just wrote resolved](specify/layout-and-validation.md#check-what-you-just-wrote-resolved)
- <a id="names"></a>[Names](specify/layout-and-validation.md#names)

## Next

* [Verify an implementation](./verify-conformance.md) — generate the suite this specification
  obliges, run it, and turn the result into evidence.
* [The billing example](https://github.com/beyond10x/ess/tree/main/examples/billing) — the billing
  example's source next to its generated output.
