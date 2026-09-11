use projection_types::core::{Arrived, Body, Choice, Wrapped, Wrapper};

#[test]
fn native_presence_and_nested_optional_identity() {
    let body = Body { status: "ready".to_owned(), nested: Some(None) };
    let mut event = Arrived {
        data: body.clone(), partial: None, choice: Choice::Gone("gone".to_owned()),
        wrapped: Wrapped(Wrapper { body: body.clone() }),
    };
    let input = projection_system::project(&event);
    assert_eq!(input.text, "ready");
    assert_eq!(input.partial, None);
    assert_eq!(input.choice, None);
    assert_eq!(input.nested, Some(None));
    assert_eq!(input.deeper, Some(Some(None)));
    assert_eq!(input.through_wrapper, "ready");
    assert_eq!(input.whole, body);
    event.partial = Some(body.clone());
    event.choice = Choice::Ready(body);
    event.data.nested = None;
    let input = projection_system::project(&event);
    assert_eq!(input.partial.as_deref(), Some("ready"));
    assert_eq!(input.choice.as_deref(), Some("ready"));
    assert_eq!(input.nested, None);
    assert_eq!(input.deeper, Some(None));
}
