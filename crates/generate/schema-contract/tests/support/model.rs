use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::Plan;

pub fn plan() -> Plan {
    let text = include_str!("../../../ess-gen/tests/fixtures/model-types.yaml");
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, text.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(text).unwrap())]).unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let selected =
        ModelTypes::select(&ir, &BTreeSet::from(["sample.data.Record".to_owned()])).unwrap();
    Plan::from_model(&selected).unwrap()
}
