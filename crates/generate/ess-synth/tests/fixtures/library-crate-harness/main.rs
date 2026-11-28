//! Reads each argument pair `<declaration> <document>` with the synthesized crate's own decoder,
//! writes what it read back with the matching encoder, reads that again and holds the two values
//! equal. Prints `ok <written>` or `refused <path> | <expectation>`, one line per pair.

use shop_model::{json, wire};

macro_rules! round_trip {
    ($document:expr, $decode:path, $encode:path) => {
        match $decode($document, "$") {
            Ok(value) => {
                let mut written = String::new();
                $encode(&value, &mut written);
                let reread = json::parse(&written).expect("the encoder writes JSON");
                let again = $decode(&reread, "$").expect("what the encoder writes decodes");
                assert_eq!(again, value, "decode(encode(v)) == v");
                format!("ok {written}")
            }
            Err(refusal) => format!("refused {} | {}", refusal.at, refusal.expected),
        }
    };
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    for pair in arguments.chunks(2) {
        let [declaration, document] = pair else {
            panic!("arguments come in pairs");
        };
        let document = json::parse(document).expect("the document parses");
        let line = match declaration.as_str() {
            "item_id" => round_trip!(
                &document,
                wire::decode_shop_catalog_item_id,
                wire::encode_shop_catalog_item_id
            ),
            "mode" => round_trip!(
                &document,
                wire::decode_shop_catalog_mode,
                wire::encode_shop_catalog_mode
            ),
            "limit" => round_trip!(
                &document,
                wire::decode_shop_catalog_limit,
                wire::encode_shop_catalog_limit
            ),
            "entry" => round_trip!(
                &document,
                wire::decode_shop_catalog_entry,
                wire::encode_shop_catalog_entry
            ),
            "sku" => round_trip!(
                &document,
                wire::decode_shop_catalog_sku,
                wire::encode_shop_catalog_sku
            ),
            "bundle" => round_trip!(
                &document,
                wire::decode_shop_catalog_bundle,
                wire::encode_shop_catalog_bundle
            ),
            "note" => round_trip!(
                &document,
                wire::decode_shop_catalog_note,
                wire::encode_shop_catalog_note
            ),
            "dep" => round_trip!(
                &document,
                wire::decode_shop_catalog_dep,
                wire::encode_shop_catalog_dep
            ),
            "catalog" => round_trip!(
                &document,
                wire::decode_entity_shop_catalog_catalog,
                wire::encode_entity_shop_catalog_catalog
            ),
            "shelf" => round_trip!(
                &document,
                wire::decode_entity_shop_catalog_shelf,
                wire::encode_entity_shop_catalog_shelf
            ),
            other => panic!("no declaration `{other}`"),
        };
        println!("{line}");
    }
}
