//! A client-paged read offers page sizes with a pager that names no params
//! (story:a-client-paged-collection-offers-sizes-and-a-range).

fn read(text: &str) -> ess_ui::Reads {
    ess_yaml::from_str::<ess_ui::Reads>(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

#[test]
fn a_pager_may_name_sizes_alone() {
    let read = read("{view: t.All, paging: client, pager: {sizes: [10, 25]}}");
    let pager = read
        .pager
        .as_ref()
        .expect("the typed read carries its pager");
    assert_eq!(pager.page, None, "`page` is optional");
    assert_eq!(pager.size, None);
    assert_eq!(pager.sizes, vec![10, 25]);

    let empty = self::read("{view: t.All, paging: client, pager: {}}");
    assert_eq!(
        empty.pager.map(|pager| pager.sizes),
        Some(ess_ui::Pager::DEFAULT_SIZES.to_vec()),
        "a pager with no key offers the default sizes"
    );
}

#[test]
fn a_server_pager_keeps_its_page() {
    let read =
        read("{view: t.All, params: {page: state.page}, paging: server, pager: {page: page}}");
    assert_eq!(
        read.pager.and_then(|pager| pager.page).as_deref(),
        Some("page"),
        "an existing document keeps its meaning"
    );
}
