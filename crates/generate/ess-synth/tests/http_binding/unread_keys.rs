//! A binding whose routes carry `request.members` or `select` is refused by both generated
//! servers, naming the route, the key and the server, until they read them: a server written from
//! it would decode a flat body and serve the first of the routes sharing a method and path
//! (`docs/design/http-presentation-binding.md`, "Several operations on one route").

use ess_synth::{synthesize_bound, BindingRefusal, OutputLayout, SynthesisFailure, Target};

use super::shared::{binding, ir};

/// `shop.items.Order` reads its `count` from `data.count`.
const MEMBERS: &str = "routes:
  - command: shop.items.Order
    method: post
    path: /orders/{item_id}
    parameters: {item_id: input.item_id}
    request: {members: {count: data.count}}
";

/// `shop.items.Order` and `shop.items.StrictOrder` share `post /orders`, told apart by `kind`.
const SELECT: &str = "routes:
  - {command: shop.items.Order, method: post, path: /orders, select: {kind: order}}
  - {command: shop.items.StrictOrder, method: post, path: /orders, select: {kind: strict}}
";

/// What synthesizing `target` from `routes` refuses, as printed.
fn refused(target: Target, routes: &str) -> String {
    let ir = ir();
    let bound = binding(&ir, routes);
    match synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound)) {
        Err(SynthesisFailure::Binding(BindingRefusal::Unservable(refused))) => refused.to_string(),
        Err(other) => panic!("refused for another reason: {other}"),
        Ok(_) => panic!("a server was written from a binding whose keys it does not read"),
    }
}

fn assert_refused_by_name(target: Target, server: &str) {
    for (routes, key, route) in [
        (MEMBERS, "`request.members`", "`post /orders/{item_id}`"),
        (SELECT, "`select`", "`post /orders`"),
    ] {
        let text = refused(target, routes);
        assert!(
            text.contains("command `shop.items.Order`")
                && text.contains(route)
                && text.contains(key)
                && text.contains(server)
                && text.contains("does not read it yet"),
            "{text}"
        );
    }
}

#[test]
fn the_rust_server_refuses_a_binding_with_members_or_select_by_name() {
    assert_refused_by_name(Target::Rust, "generated Rust server");
}

#[test]
fn the_go_server_refuses_a_binding_with_members_or_select_by_name() {
    assert_refused_by_name(Target::Go, "generated Go server");
}
