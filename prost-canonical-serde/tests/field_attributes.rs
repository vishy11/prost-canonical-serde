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

#[test]
fn field_rename_applies_to_serialize_and_deserialize_names() {
    let value = RenamedFieldMessage { count: 42 };

    assert_eq!(
        serde_json::to_value(&value).expect("serialize renamed field"),
        json!({ "wireCount": "42" })
    );

    let roundtrip: RenamedFieldMessage =
        serde_json::from_value(json!({ "wireCount": "42" }))
            .expect("deserialize renamed field");
    assert_eq!(roundtrip, value);

    let roundtrip: RenamedFieldMessage =
        serde_json::from_value(json!({ "count": "42" }))
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
        serde_json::from_value(json!({ "wire_count": "42" }))
            .expect("deserialize aliased field");
    assert_eq!(value, AliasedFieldMessage { count: 42 });

    let value: AliasedFieldMessage =
        serde_json::from_value(json!({ "wire-count": "42" }))
            .expect("deserialize second aliased field");
    assert_eq!(value, AliasedFieldMessage { count: 42 });

    assert_eq!(
        serde_json::to_value(&AliasedFieldMessage { count: 42 })
            .expect("serialize aliased field"),
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

    let roundtrip: RenamedAliasedFieldMessage =
        serde_json::from_value(json!({ "name": "demo" }))
            .expect("deserialize renamed aliased field from proto alias");
    assert_eq!(roundtrip, value);
}

#[test]
fn field_default_path_fills_missing_field() {
    let value: FieldPathDefaultMessage =
        serde_json::from_value(json!({}))
            .expect("missing field should use field default path");

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
        serde_json::from_value(json!({}))
            .expect("field default should override container default");

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
