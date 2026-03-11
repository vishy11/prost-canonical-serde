extern crate alloc;
extern crate serde as renamed_serde;

use std::cell::RefCell;
use std::fmt;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};
use serde::de::{self, Deserialize, Deserializer, IntoDeserializer, MapAccess, Visitor, value};
use serde::forward_to_deserialize_any;
use serde::ser::Error as _;
use serde::ser::{self, Impossible, SerializeStruct, Serializer};
use serde_json::json;

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(rename = "wire_message")]
struct RenamedMessage {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(proto_name = "value", json_name = "value")]
    value: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(crate = "renamed_serde")]
struct CustomSerdePathMessage {
    #[prost(string, tag = "1")]
    value: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(deny_unknown_fields)]
struct StrictMessage {
    #[prost(string, tag = "1")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(expecting = "a message object")]
struct ExpectingMessage {
    #[prost(string, tag = "1")]
    note: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(default)]
struct ContainerDefaultMessage {
    #[prost(int64, tag = "1")]
    count: i64,
    #[prost(string, tag = "2")]
    note: String,
}

impl Default for ContainerDefaultMessage {
    fn default() -> Self {
        Self {
            count: 42,
            note: "fallback".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(default = "custom_default_message")]
struct PathDefaultMessage {
    #[prost(int64, tag = "1")]
    count: i64,
    #[prost(string, tag = "2")]
    note: String,
}

fn custom_default_message() -> PathDefaultMessage {
    PathDefaultMessage {
        count: 7,
        note: "path".to_string(),
    }
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct FromWireMessage {
    #[prost(int64, tag = "1")]
    count: i64,
}

#[derive(Debug, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
struct IntoWireMessage {
    #[prost(int64, tag = "1")]
    count: i64,
}

#[derive(Debug, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(into = "IntoWireMessage")]
struct IntoConvertedMessage {
    #[prost(int64, tag = "1")]
    count: i64,
}

impl From<IntoConvertedMessage> for IntoWireMessage {
    fn from(value: IntoConvertedMessage) -> Self {
        Self {
            count: value.count + 1,
        }
    }
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(from = "FromWireMessage")]
struct FromConvertedMessage {
    #[prost(int64, tag = "1")]
    count: i64,
}

impl From<FromWireMessage> for FromConvertedMessage {
    fn from(value: FromWireMessage) -> Self {
        Self {
            count: value.count + 1,
        }
    }
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
enum TryFromWireChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(try_from = "TryFromWireChoice")]
enum TryFromConvertedChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

struct TryFromChoiceError(&'static str);

impl fmt::Display for TryFromChoiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl TryFrom<TryFromWireChoice> for TryFromConvertedChoice {
    type Error = TryFromChoiceError;

    fn try_from(value: TryFromWireChoice) -> Result<Self, Self::Error> {
        match value {
            TryFromWireChoice::Value(text) if text.is_empty() => Err(TryFromChoiceError("empty values are not allowed")),
            TryFromWireChoice::Value(text) => Ok(Self::Value(text.to_ascii_uppercase())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
enum IntoWireChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

#[derive(Debug, Clone, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(into = "IntoWireChoice")]
enum IntoConvertedChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

impl From<IntoConvertedChoice> for IntoWireChoice {
    fn from(value: IntoConvertedChoice) -> Self {
        match value {
            IntoConvertedChoice::Value(text) => Self::Value(text.to_ascii_uppercase()),
        }
    }
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(
    rename(serialize = "wire_tag_ser", deserialize = "wire_tag_de"),
    tag = "type"
)]
struct TaggedMessage {
    #[prost(int64, tag = "1")]
    count: i64,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(rename(serialize = "wire_box_ser", deserialize = "wire_box_de"))]
#[prost_canonical_serde(transparent)]
struct RenamedTransparent {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(proto_name = "value", json_name = "value")]
    value: String,
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(rename(serialize = "wire_choice_ser", deserialize = "wire_choice_de"))]
enum RenamedChoice {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(proto_name = "value_choice", json_name = "valueChoice")]
    ValueChoice(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(crate = "renamed_serde")]
enum CustomSerdePathChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(deny_unknown_fields)]
enum StrictChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

#[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
#[serde(expecting = "a choice object")]
enum ExpectingChoice {
    #[prost(string, tag = "1")]
    Value(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaptureError(String);

impl fmt::Display for CaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CaptureError {}

impl ser::Error for CaptureError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self(msg.to_string())
    }
}

struct NameCaptureSerializer;

struct NameCaptureStruct {
    name: String,
}

impl SerializeStruct for NameCaptureStruct {
    type Ok = String;
    type Error = CaptureError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.name)
    }
}

impl Serializer for NameCaptureSerializer {
    type Ok = String;
    type Error = CaptureError;
    type SerializeSeq = Impossible<String, CaptureError>;
    type SerializeTuple = Impossible<String, CaptureError>;
    type SerializeTupleStruct = Impossible<String, CaptureError>;
    type SerializeTupleVariant = Impossible<String, CaptureError>;
    type SerializeMap = Impossible<String, CaptureError>;
    type SerializeStruct = NameCaptureStruct;
    type SerializeStructVariant = Impossible<String, CaptureError>;

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(NameCaptureStruct {
            name: name.to_string(),
        })
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        Ok(name.to_string())
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        Err(CaptureError::custom("unexpected scalar serialization"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(CaptureError::custom("unexpected sequence serialization"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(CaptureError::custom("unexpected sequence serialization"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(CaptureError::custom("unexpected sequence serialization"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(CaptureError::custom("unexpected sequence serialization"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(CaptureError::custom("unexpected map serialization"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(CaptureError::custom("unexpected variant serialization"))
    }
}

struct SingleFieldMap {
    key: Option<&'static str>,
    value: &'static str,
}

struct StringValueDeserializer(&'static str);

impl<'de> Deserializer<'de> for StringValueDeserializer {
    type Error = value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.0)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.0)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0.to_string())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes byte_buf unit unit_struct
        newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any
    }
}

impl SingleFieldMap {
    fn new(key: &'static str, value: &'static str) -> Self {
        Self {
            key: Some(key),
            value,
        }
    }
}

impl<'de> MapAccess<'de> for SingleFieldMap {
    type Error = value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.key.take() {
            Some(key) => seed.deserialize(key.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(StringValueDeserializer(self.value))
    }
}

struct StructNameDeserializer<'a> {
    seen: &'a RefCell<Option<String>>,
    key: &'static str,
    value: &'static str,
}

impl<'a> StructNameDeserializer<'a> {
    fn new(seen: &'a RefCell<Option<String>>, key: &'static str, value: &'static str) -> Self {
        Self { seen, key, value }
    }
}

impl<'de> Deserializer<'de> for StructNameDeserializer<'_> {
    type Error = value::Error;

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        *self.seen.borrow_mut() = Some(name.to_string());
        visitor.visit_map(SingleFieldMap::new(self.key, self.value))
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::custom("expected deserialize_struct"))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option
        unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier ignored_any
    }
}

struct NewtypeNameDeserializer<'a> {
    seen: &'a RefCell<Option<String>>,
    value: &'static str,
}

impl<'a> NewtypeNameDeserializer<'a> {
    fn new(seen: &'a RefCell<Option<String>>, value: &'static str) -> Self {
        Self { seen, value }
    }
}

impl<'de> Deserializer<'de> for NewtypeNameDeserializer<'_> {
    type Error = value::Error;

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        *self.seen.borrow_mut() = Some(name.to_string());
        visitor.visit_newtype_struct(StringValueDeserializer(self.value))
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::custom("expected deserialize_newtype_struct"))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option
        unit unit_struct struct seq tuple tuple_struct map enum identifier ignored_any
    }
}

#[test]
fn container_rename_applies_to_struct_serialize_and_deserialize_names() {
    let serialize_name = serde::Serialize::serialize(
        &RenamedMessage {
            value: "demo".to_string(),
        },
        NameCaptureSerializer,
    )
    .expect("serialize renamed message");
    assert_eq!(serialize_name, "wire_message");

    let seen = RefCell::new(None);
    let roundtrip =
        RenamedMessage::deserialize(StructNameDeserializer::new(&seen, "value", "demo"))
            .expect("deserialize renamed message");
    assert_eq!(seen.into_inner(), Some("wire_message".to_string()));
    assert_eq!(
        roundtrip,
        RenamedMessage {
            value: "demo".to_string(),
        }
    );
}

#[test]
fn container_serde_crate_path_is_accepted_for_structs() {
    let value = CustomSerdePathMessage {
        value: "demo".to_string(),
    };
    let json = serde_json::to_value(&value).expect("serialize with custom serde path");
    assert_eq!(json, json!({ "value": "demo" }));

    let roundtrip: CustomSerdePathMessage =
        serde_json::from_value(json!({ "value": "demo" }))
            .expect("deserialize with custom serde path");
    assert_eq!(roundtrip, value);
}

#[test]
fn container_rename_supports_independent_newtype_names() {
    let serialize_name = serde::Serialize::serialize(
        &RenamedTransparent {
            value: "demo".to_string(),
        },
        NameCaptureSerializer,
    )
    .expect("serialize renamed transparent");
    assert_eq!(serialize_name, "wire_box_ser");

    let seen = RefCell::new(None);
    let roundtrip = RenamedTransparent::deserialize(NewtypeNameDeserializer::new(&seen, "demo"))
        .expect("deserialize renamed transparent");
    assert_eq!(seen.into_inner(), Some("wire_box_de".to_string()));
    assert_eq!(
        roundtrip,
        RenamedTransparent {
            value: "demo".to_string(),
        }
    );
}

#[test]
fn container_rename_applies_to_oneof_enums() {
    let serialize_name = serde::Serialize::serialize(
        &RenamedChoice::ValueChoice("demo".to_string()),
        NameCaptureSerializer,
    )
    .expect("serialize renamed oneof");
    assert_eq!(serialize_name, "wire_choice_ser");

    let seen = RefCell::new(None);
    let roundtrip =
        RenamedChoice::deserialize(StructNameDeserializer::new(&seen, "valueChoice", "demo"))
            .expect("deserialize renamed oneof");
    assert_eq!(seen.into_inner(), Some("wire_choice_de".to_string()));
    assert_eq!(roundtrip, RenamedChoice::ValueChoice("demo".to_string()));
}

#[test]
fn container_serde_crate_path_is_accepted_for_oneofs() {
    let value = CustomSerdePathChoice::Value("demo".to_string());
    let json = serde_json::to_value(&value).expect("serialize oneof with custom serde path");
    assert_eq!(json, json!({ "value": "demo" }));

    let roundtrip: CustomSerdePathChoice =
        serde_json::from_value(json!({ "value": "demo" }))
            .expect("deserialize oneof with custom serde path");
    assert_eq!(roundtrip, value);
}

#[test]
fn deny_unknown_fields_rejects_unknown_struct_keys() {
    let err = serde_json::from_value::<StrictMessage>(json!({
        "note": "demo",
        "extra": "nope"
    }))
    .expect_err("unknown fields should be rejected");

    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn container_expecting_customizes_struct_errors() {
    let err = serde_json::from_value::<ExpectingMessage>(json!("demo"))
        .expect_err("invalid type should use custom expecting text");

    assert!(err.to_string().contains("expected a message object"));
}

#[test]
fn deny_unknown_fields_rejects_unknown_oneof_keys() {
    let err = serde_json::from_value::<StrictChoice>(json!({
        "extra": "nope"
    }))
    .expect_err("unknown oneof fields should be rejected");

    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn container_expecting_customizes_oneof_errors() {
    let err = serde_json::from_value::<ExpectingChoice>(json!("demo"))
        .expect_err("invalid type should use custom expecting text");

    assert!(err.to_string().contains("expected a choice object"));
}

#[test]
fn unknown_fields_are_ignored_without_deny_unknown_fields() {
    let value: RenamedMessage = serde_json::from_value(json!({
        "value": "demo",
        "extra": "ignored"
    }))
    .expect("unknown fields should be ignored by default");

    assert_eq!(
        value,
        RenamedMessage {
            value: "demo".to_string(),
        }
    );
}

#[test]
fn container_default_fills_missing_fields_from_default_impl() {
    let value: ContainerDefaultMessage =
        serde_json::from_value(json!({})).expect("missing fields should use container default");

    assert_eq!(
        value,
        ContainerDefaultMessage {
            count: 42,
            note: "fallback".to_string(),
        }
    );
}

#[test]
fn container_default_preserves_present_fields() {
    let value: ContainerDefaultMessage = serde_json::from_value(json!({
        "count": "9"
    }))
    .expect("present fields should override container default");

    assert_eq!(
        value,
        ContainerDefaultMessage {
            count: 9,
            note: "fallback".to_string(),
        }
    );
}

#[test]
fn container_default_path_fills_missing_fields() {
    let value: PathDefaultMessage =
        serde_json::from_value(json!({})).expect("missing fields should use default path");

    assert_eq!(
        value,
        PathDefaultMessage {
            count: 7,
            note: "path".to_string(),
        }
    );
}

#[test]
fn container_from_deserializes_through_intermediate_type() {
    let value: FromConvertedMessage =
        serde_json::from_value(json!({ "count": "7" })).expect("deserialize via from");

    assert_eq!(value, FromConvertedMessage { count: 8 });
}

#[test]
fn container_into_serializes_through_intermediate_type() {
    let value = serde_json::to_value(&IntoConvertedMessage { count: 7 })
        .expect("serialize via into");

    assert_eq!(value, json!({ "count": "8" }));
}

#[test]
fn container_try_from_deserializes_through_intermediate_type() {
    let value: TryFromConvertedChoice =
        serde_json::from_value(json!({ "value": "demo" })).expect("deserialize via try_from");

    assert_eq!(value, TryFromConvertedChoice::Value("DEMO".to_string()));
}

#[test]
fn container_into_serializes_oneof_through_intermediate_type() {
    let value = serde_json::to_value(&IntoConvertedChoice::Value("demo".to_string()))
        .expect("serialize oneof via into");

    assert_eq!(value, json!({ "value": "DEMO" }));
}

#[test]
fn container_try_from_propagates_conversion_errors() {
    let err = serde_json::from_value::<TryFromConvertedChoice>(json!({
        "value": ""
    }))
    .expect_err("failed conversion should be reported");

    assert!(err.to_string().contains("empty values are not allowed"));
}

#[test]
fn tagged_struct_serializes_tag_first_with_renamed_value() {
    let json = serde_json::to_string(&TaggedMessage { count: 7 }).expect("serialize tagged struct");
    assert_eq!(json, r#"{"type":"wire_tag_ser","count":"7"}"#);
}

#[test]
fn tagged_struct_requires_matching_tag_on_deserialize() {
    let value: TaggedMessage = serde_json::from_value(json!({
        "type": "wire_tag_de",
        "count": "7"
    }))
    .expect("deserialize tagged struct");

    assert_eq!(value, TaggedMessage { count: 7 });
}

#[test]
fn tagged_struct_rejects_missing_tag_on_deserialize() {
    let err = serde_json::from_value::<TaggedMessage>(json!({
        "count": "7"
    }))
    .expect_err("missing tag should be rejected");

    assert!(err.to_string().contains("missing field"));
}

#[test]
fn tagged_struct_rejects_wrong_tag_on_deserialize() {
    let err = serde_json::from_value::<TaggedMessage>(json!({
        "type": "wire_tag_ser",
        "count": "7"
    }))
    .expect_err("wrong tag should be rejected");

    assert!(err.to_string().contains("invalid struct tag"));
}

macro_rules! rename_all_case_tests {
    ($struct_name:ident, $enum_name:ident, $struct_test:ident, $enum_test:ident, [$($serde_attr:tt)+], $serialize_key:literal, $deserialize_key:literal) => {
        #[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
        #[serde($($serde_attr)+)]
        struct $struct_name {
            #[prost(string, tag = "1")]
            http_status_code: String,
        }

        #[derive(Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
        #[serde($($serde_attr)+)]
        enum $enum_name {
            #[prost(string, tag = "1")]
            HttpStatusCode(String),
        }

        #[test]
        fn $struct_test() {
            let value = $struct_name {
                http_status_code: "demo".to_string(),
            };
            assert_eq!(
                serde_json::to_value(&value).expect("serialize rename_all struct"),
                json!({ $serialize_key: "demo" })
            );

            let roundtrip: $struct_name =
                serde_json::from_value(json!({ $deserialize_key: "demo" }))
                    .expect("deserialize rename_all struct");
            assert_eq!(roundtrip, value);
        }

        #[test]
        fn $enum_test() {
            let value = $enum_name::HttpStatusCode("demo".to_string());
            assert_eq!(
                serde_json::to_value(&value).expect("serialize rename_all oneof"),
                json!({ $serialize_key: "demo" })
            );

            let roundtrip: $enum_name =
                serde_json::from_value(json!({ $deserialize_key: "demo" }))
                    .expect("deserialize rename_all oneof");
            assert_eq!(roundtrip, value);
        }
    };
}

rename_all_case_tests!(
    LowercaseFields,
    LowercaseChoice,
    rename_all_lowercase_struct_fields,
    rename_all_lowercase_oneof_variants,
    [rename_all = "lowercase"],
    "httpstatuscode",
    "httpstatuscode"
);

rename_all_case_tests!(
    UppercaseFields,
    UppercaseChoice,
    rename_all_uppercase_struct_fields,
    rename_all_uppercase_oneof_variants,
    [rename_all = "UPPERCASE"],
    "HTTPSTATUSCODE",
    "HTTPSTATUSCODE"
);

rename_all_case_tests!(
    PascalCaseFields,
    PascalCaseChoice,
    rename_all_pascal_case_struct_fields,
    rename_all_pascal_case_oneof_variants,
    [rename_all = "PascalCase"],
    "HttpStatusCode",
    "HttpStatusCode"
);

rename_all_case_tests!(
    CamelCaseFields,
    CamelCaseChoice,
    rename_all_camel_case_struct_fields,
    rename_all_camel_case_oneof_variants,
    [rename_all = "camelCase"],
    "httpStatusCode",
    "httpStatusCode"
);

rename_all_case_tests!(
    SnakeCaseFields,
    SnakeCaseChoice,
    rename_all_snake_case_struct_fields,
    rename_all_snake_case_oneof_variants,
    [rename_all = "snake_case"],
    "http_status_code",
    "http_status_code"
);

rename_all_case_tests!(
    ScreamingSnakeCaseFields,
    ScreamingSnakeCaseChoice,
    rename_all_screaming_snake_case_struct_fields,
    rename_all_screaming_snake_case_oneof_variants,
    [rename_all = "SCREAMING_SNAKE_CASE"],
    "HTTP_STATUS_CODE",
    "HTTP_STATUS_CODE"
);

rename_all_case_tests!(
    KebabCaseFields,
    KebabCaseChoice,
    rename_all_kebab_case_struct_fields,
    rename_all_kebab_case_oneof_variants,
    [rename_all = "kebab-case"],
    "http-status-code",
    "http-status-code"
);

rename_all_case_tests!(
    ScreamingKebabCaseFields,
    ScreamingKebabCaseChoice,
    rename_all_screaming_kebab_case_struct_fields,
    rename_all_screaming_kebab_case_oneof_variants,
    [rename_all = "SCREAMING-KEBAB-CASE"],
    "HTTP-STATUS-CODE",
    "HTTP-STATUS-CODE"
);

rename_all_case_tests!(
    SplitRenameAllFields,
    SplitRenameAllChoice,
    rename_all_split_struct_fields,
    rename_all_split_oneof_variants,
    [rename_all(
        serialize = "camelCase",
        deserialize = "snake_case"
    )],
    "httpStatusCode",
    "http_status_code"
);
