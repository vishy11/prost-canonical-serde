extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};
use serde_json::json;

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct RenamedFieldMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(rename = "wireCount")]
    count: i64,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct SplitRenamedFieldMessage {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(rename(serialize = "wireName", deserialize = "wire_name"))]
    display_name: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct AliasedFieldMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(alias = "wire_count")]
    #[prost_canonical_serde(alias = "wire-count")]
    count: i64,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct RenamedAliasedFieldMessage {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(rename = "wireName")]
    #[prost_canonical_serde(alias = "wire_name")]
    #[prost_canonical_serde(alias = "wire-name")]
    name: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct FieldPathDefaultMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(default = "default_count")]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(default = "container_default_message")]
struct FieldDefaultOverridesContainerMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(default)]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

fn default_count() -> i64 {
    7
}

fn container_default_message() -> FieldDefaultOverridesContainerMessage {
    FieldDefaultOverridesContainerMessage {
        count: 42,
        note: "fallback".to_string(),
    }
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct SkippedFieldMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(skip)]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct SkipSerializingFieldMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(skip_serializing)]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct SkipDeserializingFieldMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(skip_deserializing)]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct SkipDeserializingFieldDefaultMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(skip_deserializing)]
    #[prost_canonical_serde(default = "default_count")]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(default = "container_skip_default_message")]
struct SkipDeserializingIgnoresContainerDefaultMessage {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(skip_deserializing)]
    count: i64,

    #[prost(string, tag = "2")]
    note: String,
}

fn container_skip_default_message() -> SkipDeserializingIgnoresContainerDefaultMessage {
    SkipDeserializingIgnoresContainerDefaultMessage {
        count: 42,
        note: "fallback".to_string(),
    }
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

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct CollisionInner {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(proto_name = "count", json_name = "count")]
    count: i64,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct OuterFieldCollision {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(proto_name = "count", json_name = "count")]
    count: i64,

    #[prost(message, optional, tag = "2")]
    #[prost_canonical_serde(flatten)]
    inner: Option<CollisionInner>,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct FirstFlattenCollision {
    #[prost(message, optional, tag = "1")]
    #[prost_canonical_serde(flatten)]
    first: Option<CollisionInner>,

    #[prost(message, optional, tag = "2")]
    #[prost_canonical_serde(flatten)]
    second: Option<CollisionInner>,
}

#[test]
fn field_rename_applies_to_serialize_and_deserialize_names() {
    let value = RenamedFieldMessage { count: 42 };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize renamed field"),
        json!({ "wireCount": "42" })
    );

    let roundtrip: RenamedFieldMessage =
        serde_json::from_value(json!({ "wireCount": "42" })).expect("deserialize renamed field");
    assert_eq!(roundtrip, value);

    let roundtrip: RenamedFieldMessage = serde_json::from_value(json!({ "count": "42" }))
        .expect("deserialize field from proto name alias");
    assert_eq!(roundtrip, value);
}

#[test]
fn field_rename_supports_independent_serialize_and_deserialize_names() {
    let value = SplitRenamedFieldMessage {
        display_name: "demo".to_string(),
    };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize split renamed field"),
        json!({ "wireName": "demo" })
    );

    let roundtrip: SplitRenamedFieldMessage =
        serde_json::from_value(json!({ "wire_name": "demo" }))
            .expect("deserialize split renamed field");
    assert_eq!(roundtrip, value);

    let roundtrip: SplitRenamedFieldMessage =
        serde_json::from_value(json!({ "display_name": "demo" }))
            .expect("deserialize split renamed field from proto alias");
    assert_eq!(roundtrip, value);
}

#[test]
fn field_alias_allows_multiple_deserialize_names() {
    let value: AliasedFieldMessage =
        serde_json::from_value(json!({ "wire_count": "42" })).expect("deserialize aliased field");
    assert_eq!(value, AliasedFieldMessage { count: 42 });

    let value: AliasedFieldMessage = serde_json::from_value(json!({ "wire-count": "42" }))
        .expect("deserialize second aliased field");
    assert_eq!(value, AliasedFieldMessage { count: 42 });

    assert_eq!(
        serde_json::to_value(&AliasedFieldMessage { count: 42 }).expect("serialize aliased field"),
        json!({ "count": "42" })
    );
}

#[test]
fn field_alias_works_with_explicit_rename() {
    let value = RenamedAliasedFieldMessage {
        name: "demo".to_string(),
    };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize renamed aliased field"),
        json!({ "wireName": "demo" })
    );

    let roundtrip: RenamedAliasedFieldMessage =
        serde_json::from_value(json!({ "wire_name": "demo" }))
            .expect("deserialize renamed aliased field");
    assert_eq!(roundtrip, value);

    let roundtrip: RenamedAliasedFieldMessage =
        serde_json::from_value(json!({ "wire-name": "demo" }))
            .expect("deserialize second renamed aliased field");
    assert_eq!(roundtrip, value);

    let roundtrip: RenamedAliasedFieldMessage = serde_json::from_value(json!({ "name": "demo" }))
        .expect("deserialize renamed aliased field from proto alias");
    assert_eq!(roundtrip, value);
}

#[test]
fn field_default_path_fills_missing_field() {
    let value: FieldPathDefaultMessage =
        serde_json::from_value(json!({})).expect("missing field should use field default path");

    assert_eq!(
        value,
        FieldPathDefaultMessage {
            count: 7,
            note: String::new(),
        }
    );
}

#[test]
fn field_default_path_preserves_present_field() {
    let value: FieldPathDefaultMessage =
        serde_json::from_value(json!({ "count": "9", "note": "demo" }))
            .expect("present field should override field default path");

    assert_eq!(
        value,
        FieldPathDefaultMessage {
            count: 9,
            note: "demo".to_string(),
        }
    );
}

#[test]
fn field_default_overrides_container_default_for_missing_field() {
    let value: FieldDefaultOverridesContainerMessage =
        serde_json::from_value(json!({})).expect("field default should override container default");

    assert_eq!(
        value,
        FieldDefaultOverridesContainerMessage {
            count: 0,
            note: "fallback".to_string(),
        }
    );
}

#[test]
fn field_default_still_preserves_present_field_with_container_default() {
    let value: FieldDefaultOverridesContainerMessage =
        serde_json::from_value(json!({ "count": "9" }))
            .expect("present field should override field default and container default");

    assert_eq!(
        value,
        FieldDefaultOverridesContainerMessage {
            count: 9,
            note: "fallback".to_string(),
        }
    );
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

#[test]
fn flatten_collision_deserialize_prefers_non_flattened_field() {
    let value: OuterFieldCollision = serde_json::from_value(json!({
        "count": "7"
    }))
    .expect("deserialize collision with outer field");

    assert_eq!(
        value,
        OuterFieldCollision {
            count: 7,
            inner: None,
        }
    );
}

#[test]
fn flatten_collision_deserialize_prefers_first_flattened_field() {
    let value: FirstFlattenCollision = serde_json::from_value(json!({
        "count": "9"
    }))
    .expect("deserialize collision between flattened fields");

    assert_eq!(
        value,
        FirstFlattenCollision {
            first: Some(CollisionInner { count: 9 }),
            second: None,
        }
    );
}

#[test]
fn flatten_collision_serialize_emits_keys_in_declaration_order() {
    let json = serde_json::to_string(&OuterFieldCollision {
        count: 1,
        inner: Some(CollisionInner { count: 2 }),
    })
    .expect("serialize outer collision");

    assert_eq!(json, r#"{"count":"1","count":"2"}"#);
}

#[test]
fn field_skip_omits_serialize_and_defaults_on_deserialize() {
    let value = SkippedFieldMessage {
        count: 42,
        note: "demo".to_string(),
    };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize skipped field"),
        json!({ "note": "demo" })
    );

    let roundtrip: SkippedFieldMessage =
        serde_json::from_value(json!({ "count": "9", "note": "demo" }))
            .expect("deserialize skipped field");
    assert_eq!(
        roundtrip,
        SkippedFieldMessage {
            count: 0,
            note: "demo".to_string(),
        }
    );
}

#[test]
fn field_skip_serializing_omits_serialize_but_keeps_deserialize() {
    let value = SkipSerializingFieldMessage {
        count: 42,
        note: "demo".to_string(),
    };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize skip_serializing field"),
        json!({ "note": "demo" })
    );

    let roundtrip: SkipSerializingFieldMessage =
        serde_json::from_value(json!({ "count": "9", "note": "demo" }))
            .expect("deserialize skip_serializing field");
    assert_eq!(
        roundtrip,
        SkipSerializingFieldMessage {
            count: 9,
            note: "demo".to_string(),
        }
    );
}

#[test]
fn field_skip_deserializing_keeps_serialize_and_defaults_on_deserialize() {
    let value = SkipDeserializingFieldMessage {
        count: 42,
        note: "demo".to_string(),
    };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize skip_deserializing field"),
        json!({ "count": "42", "note": "demo" })
    );

    let roundtrip: SkipDeserializingFieldMessage =
        serde_json::from_value(json!({ "count": "9", "note": "demo" }))
            .expect("deserialize skip_deserializing field");
    assert_eq!(
        roundtrip,
        SkipDeserializingFieldMessage {
            count: 0,
            note: "demo".to_string(),
        }
    );
}

#[test]
fn field_skip_deserializing_honors_field_default_path() {
    let value: SkipDeserializingFieldDefaultMessage =
        serde_json::from_value(json!({ "count": "9", "note": "demo" }))
            .expect("deserialize skip_deserializing field with default path");

    assert_eq!(
        value,
        SkipDeserializingFieldDefaultMessage {
            count: 7,
            note: "demo".to_string(),
        }
    );
}

#[test]
fn field_skip_deserializing_uses_type_default_instead_of_container_default() {
    let value: SkipDeserializingIgnoresContainerDefaultMessage = serde_json::from_value(json!({}))
        .expect("deserialize skip_deserializing field with container default");

    assert_eq!(
        value,
        SkipDeserializingIgnoresContainerDefaultMessage {
            count: 0,
            note: "fallback".to_string(),
        }
    );
}
