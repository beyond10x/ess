//! Adversary case for beyond10x/ess#425: `IssueInvoice` now records the caller's `issued_at`, and
//! `OutstandingInvoices` declares `order_by: issued_at desc`.
//!
//! A `Timestamp` travels as text, and `ess_primitives::Rfc3339Instant` says what ordering one means:
//! "a guard that orders two of them must order the instants and not the spellings:
//! `2020-01-01T12:00:00+01:00` is before `2020-01-01T11:30:00Z`". Before #425 every `issued_at` here
//! came from one counter in one spelling, so ordering the text happened to order the instants. Now
//! the caller writes it, and any RFC 3339 offset is a valid `Timestamp`.

use billing_realization::invoice::{InvoiceRealization, SharedInvoices};
use billing_types::invoice::obligations::{
    CreateInvoiceBehavior, IssueInvoiceBehavior, OutstandingInvoicesQuery,
};
use billing_types::invoice::{
    AccountId, CreateInvoice, CreateInvoiceOutcome, Email, InvoiceId, IssueInvoice, Money,
};
use billing_types::primitives::{Decimal, Timestamp, Uuid};
use ess_primitives::Rfc3339Instant;

fn create(realization: &mut InvoiceRealization) -> InvoiceId {
    let outcome = realization
        .create_invoice(CreateInvoice {
            account_id: AccountId(Uuid("00000000-0000-4000-8000-000000000001".to_owned())),
            customer_email: Email("a@example.com".to_owned()),
            amount: Money {
                amount: Decimal("1".to_owned()),
                currency: "EUR".to_owned(),
            },
        })
        .expect("the honest realization satisfies the obligation");
    let CreateInvoiceOutcome::Accepted { invoice_created } = outcome else {
        panic!("a positive amount is accepted, got {outcome:?}");
    };
    invoice_created.invoice_id
}

#[test]
fn realization_ranks_outstanding_invoices_by_instant_not_by_spelling() {
    // 08:00:01Z, written with an offset, and 09:00:03Z: the first is the earlier instant and the
    // lexicographically greater text.
    let earlier_text = "2026-01-05T10:00:01+02:00";
    let later_text = "2026-01-05T09:00:03Z";
    assert!(
        Rfc3339Instant::parse_rfc3339(earlier_text).expect("a valid Timestamp")
            < Rfc3339Instant::parse_rfc3339(later_text).expect("a valid Timestamp"),
        "fixture: `{earlier_text}` names the earlier instant"
    );

    let mut realization = InvoiceRealization::over(SharedInvoices::new());
    let earlier = create(&mut realization);
    let later = create(&mut realization);
    for (id, at) in [(&earlier, earlier_text), (&later, later_text)] {
        realization
            .issue_invoice(IssueInvoice {
                invoice_id: id.clone(),
                issued_at: Timestamp(at.to_owned()),
            })
            .expect("the obligation is satisfied");
    }
    let rows = realization
        .outstanding_invoices()
        .expect("the obligation is satisfied");
    let order: Vec<_> = rows.iter().map(|row| row.issued_at.clone()).collect();
    assert_eq!(
        rows.iter().map(|row| row.invoice_id.clone()).collect::<Vec<_>>(),
        vec![later, earlier],
        "`order_by: issued_at desc` ranks the later instant first; the realization answered {order:?}"
    );
}
