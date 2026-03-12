extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};
use serde_json::json;

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
enum RenamedChoice {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(rename(serialize = "wireValue", deserialize = "wire_value"))]
    Value(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
enum AliasedChoice {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(alias = "wire_value")]
    #[prost_canonical_serde(alias = "wire-value")]
    Value(String),

    #[prost(string, tag = "2")]
    Other(String),
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize,
)]
#[repr(i32)]
enum RenamedEnum {
    #[prost_canonical_serde(rename(serialize = "WIRE_READY", deserialize = "wire-ready"))]
    Ready = 0,
    Idle = 1,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize,
)]
#[repr(i32)]
enum AliasedEnum {
    #[prost_canonical_serde(alias = "wire-ready")]
    #[prost_canonical_serde(alias = "wire_ready")]
    Ready = 0,
    Idle = 1,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
enum SkippedChoice {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(skip)]
    Value(String),

    #[prost(string, tag = "2")]
    Other(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(deny_unknown_fields)]
enum SkipDeserializeChoice {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(skip_deserializing)]
    Value(String),

    #[prost(string, tag = "2")]
    Other(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
enum SkipSerializeChoice {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(skip_serializing)]
    Value(String),

    #[prost(string, tag = "2")]
    Other(String),
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize,
)]
#[repr(i32)]
enum SkippedEnum {
    #[prost_canonical_serde(skip)]
    Ready = 0,
    Idle = 1,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize,
)]
#[repr(i32)]
enum SkipSerializeEnum {
    #[prost_canonical_serde(skip_serializing)]
    Ready = 0,
    Idle = 1,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize,
)]
#[repr(i32)]
enum SkipDeserializeEnum {
    #[prost_canonical_serde(skip_deserializing)]
    Ready = 0,
    Idle = 1,
}

impl RenamedEnum {
    fn as_str_name(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Idle => "IDLE",
        }
    }

    fn from_str_name(value: &str) -> Option<Self> {
        match value {
            "READY" => Some(Self::Ready),
            "IDLE" => Some(Self::Idle),
            _ => None,
        }
    }
}

impl AliasedEnum {
    fn as_str_name(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Idle => "IDLE",
        }
    }

    fn from_str_name(value: &str) -> Option<Self> {
        match value {
            "READY" => Some(Self::Ready),
            "IDLE" => Some(Self::Idle),
            _ => None,
        }
    }
}

impl SkippedEnum {
    fn as_str_name(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Idle => "IDLE",
        }
    }

    fn from_str_name(value: &str) -> Option<Self> {
        match value {
            "READY" => Some(Self::Ready),
            "IDLE" => Some(Self::Idle),
            _ => None,
        }
    }
}

impl SkipSerializeEnum {
    fn as_str_name(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Idle => "IDLE",
        }
    }

    fn from_str_name(value: &str) -> Option<Self> {
        match value {
            "READY" => Some(Self::Ready),
            "IDLE" => Some(Self::Idle),
            _ => None,
        }
    }
}

impl SkipDeserializeEnum {
    fn as_str_name(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Idle => "IDLE",
        }
    }

    fn from_str_name(value: &str) -> Option<Self> {
        match value {
            "READY" => Some(Self::Ready),
            "IDLE" => Some(Self::Idle),
            _ => None,
        }
    }
}

#[test]
fn variant_rename_applies_to_oneof_serialize_and_deserialize_names() {
    let value = RenamedChoice::Value("demo".to_string());

    assert_eq!(
        serde_json::to_value(&value).expect("serialize renamed oneof variant"),
        json!({ "wireValue": "demo" })
    );

    let roundtrip: RenamedChoice = serde_json::from_value(json!({ "wire_value": "demo" }))
        .expect("deserialize renamed oneof variant");

    assert_eq!(roundtrip, value);
}

#[test]
fn variant_rename_applies_to_protobuf_enum_serialize_and_deserialize_names() {
    assert_eq!(
        serde_json::to_value(RenamedEnum::Ready).expect("serialize renamed protobuf enum"),
        json!("WIRE_READY")
    );

    let roundtrip: RenamedEnum =
        serde_json::from_value(json!("wire-ready")).expect("deserialize renamed protobuf enum");

    assert_eq!(roundtrip, RenamedEnum::Ready);
}

#[test]
fn variant_alias_applies_to_oneof_deserialize_names() {
    let roundtrip: AliasedChoice = serde_json::from_value(json!({ "wire_value": "demo" }))
        .expect("deserialize aliased oneof variant");
    assert_eq!(roundtrip, AliasedChoice::Value("demo".to_string()));

    let roundtrip: AliasedChoice = serde_json::from_value(json!({ "wire-value": "demo" }))
        .expect("deserialize second aliased oneof variant");
    assert_eq!(roundtrip, AliasedChoice::Value("demo".to_string()));
}

#[test]
fn variant_alias_applies_to_protobuf_enum_deserialize_names() {
    let roundtrip: AliasedEnum =
        serde_json::from_value(json!("wire-ready")).expect("deserialize aliased protobuf enum");
    assert_eq!(roundtrip, AliasedEnum::Ready);

    let roundtrip: AliasedEnum = serde_json::from_value(json!("wire_ready"))
        .expect("deserialize second aliased protobuf enum");
    assert_eq!(roundtrip, AliasedEnum::Ready);
}

#[test]
fn variant_skip_rejects_oneof_serialize_and_deserialize() {
    let err = serde_json::to_value(SkippedChoice::Value("demo".to_string()))
        .expect_err("skip variant should fail serialization");
    assert!(
        err.to_string()
            .contains("skipped variant cannot be serialized")
    );

    let err = serde_json::from_value::<SkippedChoice>(json!({ "value": "demo" }))
        .expect_err("skip variant should not deserialize");
    assert!(err.to_string().contains("expected oneof field"));
}

#[test]
fn variant_skip_serializing_only_rejects_oneof_serialize() {
    let err = serde_json::to_value(SkipSerializeChoice::Value("demo".to_string()))
        .expect_err("skip_serializing variant should fail serialization");
    assert!(
        err.to_string()
            .contains("skipped variant cannot be serialized")
    );

    let roundtrip: SkipSerializeChoice = serde_json::from_value(json!({ "value": "demo" }))
        .expect("skip_serializing variant should still deserialize");
    assert_eq!(roundtrip, SkipSerializeChoice::Value("demo".to_string()));
}

#[test]
fn variant_skip_deserializing_only_rejects_oneof_deserialize() {
    assert_eq!(
        serde_json::to_value(SkipDeserializeChoice::Value("demo".to_string()))
            .expect("skip_deserializing variant should still serialize"),
        json!({ "value": "demo" })
    );

    let err = serde_json::from_value::<SkipDeserializeChoice>(json!({ "value": "demo" }))
        .expect_err("skip_deserializing variant should not deserialize");
    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn variant_skip_rejects_protobuf_enum_serialize_and_deserialize() {
    let err = serde_json::to_value(SkippedEnum::Ready)
        .expect_err("skip variant should fail enum serialization");
    assert!(
        err.to_string()
            .contains("skipped enum variant cannot be serialized")
    );

    let err = serde_json::from_value::<SkippedEnum>(json!("READY"))
        .expect_err("skip variant should not deserialize from string");
    assert!(err.to_string().contains("invalid enum string"));

    let err = serde_json::from_value::<SkippedEnum>(json!(0))
        .expect_err("skip variant should not deserialize from number");
    assert!(
        err.to_string()
            .contains("skipped enum variant cannot be deserialized")
    );
}

#[test]
fn variant_skip_serializing_only_rejects_protobuf_enum_serialize() {
    let err = serde_json::to_value(SkipSerializeEnum::Ready)
        .expect_err("skip_serializing variant should fail enum serialization");
    assert!(
        err.to_string()
            .contains("skipped enum variant cannot be serialized")
    );

    let roundtrip: SkipSerializeEnum = serde_json::from_value(json!("READY"))
        .expect("skip_serializing variant should still deserialize");
    assert_eq!(roundtrip, SkipSerializeEnum::Ready);
}

#[test]
fn variant_skip_deserializing_only_rejects_protobuf_enum_deserialize() {
    assert_eq!(
        serde_json::to_value(SkipDeserializeEnum::Ready)
            .expect("skip_deserializing variant should still serialize"),
        json!("READY")
    );

    let err = serde_json::from_value::<SkipDeserializeEnum>(json!("READY"))
        .expect_err("skip_deserializing variant should not deserialize from string");
    assert!(err.to_string().contains("invalid enum string"));

    let err = serde_json::from_value::<SkipDeserializeEnum>(json!(0))
        .expect_err("skip_deserializing variant should not deserialize from number");
    assert!(
        err.to_string()
            .contains("skipped enum variant cannot be deserialized")
    );
}
