//! A binding whose routes carry `request.members`, `select`, `lists` or `cors`, or which declares
//! `cors` itself, is refused by both generated servers, naming the route, the key and the server,
//! until they read them: a server written from it would decode a flat body, serve the first of the
//! routes sharing a method and path, split no comma-joined list and answer no preflight
//! (`docs/design/http-presentation-binding.md`, "Several operations on one route", "Request
//! encoding", "Cross-origin reads").

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

/// `shop.items.Search` writes its `words` comma-joined.
const LISTS: &str = "routes:
  - command: shop.items.Search
    method: get
    path: /search
    request: {encoding: query}
    lists: {words: comma}
";

/// `shop.items.Order` admits no cross-origin read: the one form a server could honour by doing
/// nothing, refused all the same, since a server that answers no preflight states no policy.
const CORS_NONE: &str = "routes:
  - {command: shop.items.Order, method: post, path: /orders, cors: none}
";

/// The binding's own policy, over `shop.items.Order` and every derived route.
const CORS_BINDING: &str = "cors: {origins: [https://app.example]}
routes:
  - {command: shop.items.Order, method: post, path: /orders}
";

fn assert_lists_and_cors_refused_by_name(target: Target, server: &str) {
    for (routes, operation, key, route) in [
        (
            LISTS,
            "command `shop.items.Search`",
            "`lists`",
            "`get /search`",
        ),
        (
            CORS_NONE,
            "command `shop.items.Order`",
            "`cors`",
            "`post /orders`",
        ),
        (
            CORS_BINDING,
            "command `shop.items.Order`",
            "the binding's `cors`",
            "`post /orders`",
        ),
    ] {
        let text = refused(target, routes);
        assert!(
            text.lines().any(|line| line.contains(operation)
                && line.contains(route)
                && line.contains(key)
                && line.contains(server)
                && line.contains("does not read it yet")),
            "{text}"
        );
    }
}

#[test]
fn the_rust_server_refuses_a_binding_with_lists_or_cors_by_name() {
    assert_lists_and_cors_refused_by_name(Target::Rust, "generated Rust server");
}

#[test]
fn the_go_server_refuses_a_binding_with_lists_or_cors_by_name() {
    assert_lists_and_cors_refused_by_name(Target::Go, "generated Go server");
}
