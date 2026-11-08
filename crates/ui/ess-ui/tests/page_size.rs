//! A server-paged read names the params that carry its page and its page size
//! (story:a-server-paged-collection-has-a-page-size-control).

use ess_yaml::Value;

fn read(text: &str) -> ess_ui::Reads {
    ess_yaml::from_str::<ess_ui::Reads>(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

#[test]
fn a_read_takes_a_pager() {
    let read = read(
        "{view: t.All, params: {page: state.page, size: state.size}, paging: server, pager: {page: page, size: size, sizes: [25, 50, 100]}}",
    );
    let pager = read
        .pager
        .as_ref()
        .expect("the typed read carries its pager");
    assert_eq!(pager.page.as_deref(), Some("page"));
    assert_eq!(pager.size.as_deref(), Some("size"));
    assert_eq!(pager.sizes, vec![25, 50, 100]);
    assert_eq!(read.state_path("page"), Some("state.page"));
    assert_eq!(read.state_path("size"), Some("state.size"));

    let other = self::read(
        "{view: t.All, params: {p: state.paging.at, q: params.id, r: 'state.p == 1'}, paging: server, pager: {page: p}}",
    );
    assert_eq!(other.state_path("p"), Some("state.paging.at"));
    assert_eq!(
        other.state_path("q"),
        None,
        "a param bound to params is no state path"
    );
    assert_eq!(
        other.state_path("r"),
        None,
        "an expression is no state path"
    );
    assert_eq!(other.state_path("missing"), None);
    assert_eq!(
        other.pager.as_ref().and_then(|pager| pager.size.as_deref()),
        None
    );

    let refused = ess_yaml::from_str::<ess_ui::Reads>(
        "{view: t.All, params: {page: state.page}, paging: server, pager: {page: page, rows: 10}}",
    );
    assert!(
        refused.is_err(),
        "a pager refuses a key it does not declare"
    );
}

#[test]
fn pager_sizes_default() {
    let read = read(
        "{view: t.All, params: {page: state.page, size: state.size}, paging: server, pager: {page: page, size: size}}",
    );
    let pager = read.pager.expect("the pager loads");
    assert_eq!(pager.sizes, vec![25, 50, 100]);

    let schema: Value = ess_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML");
    let declared = &schema["constructs"]["Reads"]["fields"]["pager"]["sizes_default"];
    let declared: Vec<i64> = declared
        .as_sequence()
        .unwrap_or_else(|| panic!("the schema states the pager's default sizes: {declared:?}"))
        .iter()
        .map(|size| size.as_i64().expect("a size is an integer"))
        .collect();
    assert_eq!(
        pager.sizes, declared,
        "the loader's default sizes are the schema's"
    );
}
