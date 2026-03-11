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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize)]
#[repr(i32)]
enum RenamedEnum {
    #[prost_canonical_serde(rename(serialize = "WIRE_READY", deserialize = "wire-ready"))]
    Ready = 0,
    Idle = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration, CanonicalSerialize, CanonicalDeserialize)]
#[repr(i32)]
enum AliasedEnum {
    #[prost_canonical_serde(alias = "wire-ready")]
    #[prost_canonical_serde(alias = "wire_ready")]
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

#[test]
fn variant_rename_applies_to_oneof_serialize_and_deserialize_names() {
    let value = RenamedChoice::Value("demo".to_string());

    assert_eq!(
        serde_json::to_value(&value).expect("serialize renamed oneof variant"),
        json!({ "wireValue": "demo" })
    );

    let roundtrip: RenamedChoice =
        serde_json::from_value(json!({ "wire_value": "demo" }))
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
        serde_json::from_value(json!("wire-ready"))
            .expect("deserialize renamed protobuf enum");

    assert_eq!(roundtrip, RenamedEnum::Ready);
}

#[test]
fn variant_alias_applies_to_oneof_deserialize_names() {
    let roundtrip: AliasedChoice =
        serde_json::from_value(json!({ "wire_value": "demo" }))
            .expect("deserialize aliased oneof variant");
    assert_eq!(roundtrip, AliasedChoice::Value("demo".to_string()));

    let roundtrip: AliasedChoice =
        serde_json::from_value(json!({ "wire-value": "demo" }))
            .expect("deserialize second aliased oneof variant");
    assert_eq!(roundtrip, AliasedChoice::Value("demo".to_string()));
}

#[test]
fn variant_alias_applies_to_protobuf_enum_deserialize_names() {
    let roundtrip: AliasedEnum =
        serde_json::from_value(json!("wire-ready"))
            .expect("deserialize aliased protobuf enum");
    assert_eq!(roundtrip, AliasedEnum::Ready);

    let roundtrip: AliasedEnum =
        serde_json::from_value(json!("wire_ready"))
            .expect("deserialize second aliased protobuf enum");
    assert_eq!(roundtrip, AliasedEnum::Ready);
}
