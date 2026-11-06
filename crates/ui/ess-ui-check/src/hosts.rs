//! Host slots: the names a host registers (`--hosts`) are each declared by a slot.
//!
//! The declaration refusals (no `reason`, a `host` written twice, `host` beside `component` or
//! `primitive`) are the loader's and reach the report as `host_slots` through `classify`.

use std::collections::BTreeSet;

use ess_ui::{Document, NodePath};

use crate::{Options, Sink};

pub(crate) fn run(document: &Document, options: &Options, sink: &mut Sink) {
    let Some(registered) = &options.hosts else {
        return;
    };
    let declared: BTreeSet<&str> = document
        .host_slots()
        .into_iter()
        .map(|(_, slot)| slot.host.as_str())
        .collect();
    let mut reported = BTreeSet::new();
    for name in registered {
        if !declared.contains(name.as_str()) && reported.insert(name.as_str()) {
            sink.push(
                "hosts_registered",
                &NodePath::root(),
                format!(
                    "the host registers `{name}`, and no host slot of the document declares it: \
                     declare the slot, or stop registering the component"
                ),
            );
        }
    }
}
