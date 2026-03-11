extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};
use serde_json::json;

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(transparent)]
struct TransparentCount {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(proto_name = "count", json_name = "count")]
    count: i64,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct FlattenedInner {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(proto_name = "count", json_name = "count")]
    count: i64,
    #[prost(string, tag = "2")]
    #[prost_canonical_serde(proto_name = "display_name", json_name = "displayName")]
    display_name: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct FlattenedOuter {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(proto_name = "name", json_name = "name")]
    name: String,
    #[prost(message, optional, tag = "2")]
    #[prost_canonical_serde(flatten)]
    inner: Option<FlattenedInner>,
}

#[test]
fn transparent_uses_inner_canonical_representation() {
    let value = TransparentCount { count: 42 };
    let json = serde_json::to_value(&value).expect("serialize transparent");
    assert_eq!(json, json!("42"));

    let roundtrip: TransparentCount =
        serde_json::from_value(json!("42")).expect("deserialize transparent");
    assert_eq!(roundtrip, value);
}

#[test]
fn flatten_merges_nested_fields_into_parent_object() {
    let value = FlattenedOuter {
        name: "demo".to_string(),
        inner: Some(FlattenedInner {
            count: 42,
            display_name: "inner".to_string(),
        }),
    };

    let json = serde_json::to_value(&value).expect("serialize flatten");
    assert_eq!(
        json,
        json!({
            "name": "demo",
            "count": "42",
            "displayName": "inner"
        })
    );

    let roundtrip: FlattenedOuter =
        serde_json::from_value(json).expect("deserialize flattened value");
    assert_eq!(roundtrip, value);
}

#[test]
fn flatten_accepts_proto_field_names_on_deserialize() {
    let value: FlattenedOuter = serde_json::from_value(json!({
        "name": "demo",
        "count": "7",
        "display_name": "proto name"
    }))
    .expect("deserialize flattened proto names");

    assert_eq!(
        value,
        FlattenedOuter {
            name: "demo".to_string(),
            inner: Some(FlattenedInner {
                count: 7,
                display_name: "proto name".to_string(),
            }),
        }
    );
}
