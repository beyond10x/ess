//! The additive read predicate is preserved for every renderer, not sent as a parameter.
#[test]
fn read_filter_loads_without_becoming_a_parameter() {
    let read = serde_yaml::from_str::<ess_ui::Reads>(
        "view: orders.All\nfilter: row.partner_id == params.id\nparams: {region: params.region}\n",
    );
    assert!(
        read.is_ok(),
        "a read must accept its client predicate: {read:?}"
    );
    let read = read.unwrap();
    assert_eq!(read.params.len(), 1);
    assert!(read.params.contains_key("region"));
}
