//! Canonical JSON serialization for prost-generated messages.
//!
//! This crate lets you keep using `serde_json` normally while producing and
//! consuming protobuf canonical JSON. You add derives to the prost-generated
//! types, and the generated `serde::Serialize`/`serde::Deserialize` impls use
//! the canonical protobuf JSON rules.
//!
//! Well-known types from `prost-types` (such as `Timestamp`, `Duration`, and
//! `Any`) are supported directly with their canonical JSON mappings.
//!
//! # End-to-end example
//! A minimal setup that generates types, derives canonical serde impls, and
//! uses `serde_json` directly.
//!
//! ## example.proto
//! ```proto
#![doc = include_str!("../docs/proto/example.proto")]
//! ```
//!
//! ## build.rs
//! ```rust,ignore
#![doc = include_str!("../docs/example_build.rs")]
//! ```
//!
//! ## usage
//! ```rust
#![doc = include_str!("../docs/usage.rs")]
//! ```
//!
//! # Additional attributes
//!
//! The derive also supports:
//! - `#[prost_canonical_serde(transparent)]` on single-field named structs
//! - `#[prost_canonical_serde(flatten)]` on message fields and `Option<Message>`
//!
//! ## Limits
//!
//! - `transparent` is only supported on single-field named structs.
//! - `flatten` is only supported on message fields and `Option<Message>`.
//! - `flatten` cannot be combined with `proto_name` / `json_name` and is not
//!   supported on `oneof` fields.
//! - If flattened keys collide, deserialization prefers non-flattened fields
//!   first and then the first matching flattened field in declaration order.
//!   Serialization emits fields in declaration order, so colliding keys can
//!   produce duplicate JSON object keys and should be avoided.
//!
//! The derive macros generate canonical protobuf JSON serde implementations, so
//! you should not need to use the adapters in this crate directly.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod canonical;

pub use canonical::{
    BufferedValue, Canonical, CanonicalEnum, CanonicalEnumMap, CanonicalEnumMapRef,
    CanonicalEnumOption, CanonicalEnumSeq, CanonicalEnumValue, CanonicalEnumVec, CanonicalError,
    CanonicalMap, CanonicalMapKey, CanonicalMapRef, CanonicalMapType, CanonicalOption,
    CanonicalSeq, CanonicalValue, CanonicalVec,
};

pub use prost_canonical_serde_derive::{CanonicalDeserialize, CanonicalSerialize};

extern crate self as prost_canonical_serde;

/// Serializes a value using protobuf canonical JSON rules.
pub trait CanonicalSerialize {
    /// Serializes this value in canonical protobuf JSON form.
    ///
    /// # Errors
    /// Returns any serializer error raised while writing JSON.
    fn serialize_canonical<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

/// Deserializes a value using protobuf canonical JSON rules.
pub trait CanonicalDeserialize: Sized {
    /// Deserializes this value from canonical protobuf JSON form.
    ///
    /// # Errors
    /// Returns any deserializer error raised while reading JSON.
    fn deserialize_canonical<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;
}

/// Internal helper trait implemented by prost-generated enums.
#[doc(hidden)]
pub trait ProstEnum: Sized {
    fn from_i32(value: i32) -> Option<Self>;
    fn from_str_name(value: &str) -> Option<Self>;
    fn as_str_name(&self) -> &'static str;
    fn as_i32(&self) -> i32;

    fn is_variant_skipped_for_serialization(value: i32) -> bool {
        let _ = value;
        false
    }

    fn is_variant_skipped_for_deserialization(value: i32) -> bool {
        let _ = value;
        false
    }
}

/// Internal helper trait implemented by prost-generated oneof enums.
#[doc(hidden)]
pub trait ProstOneof: Sized {
    fn serialize_field<S>(&self, map: &mut S) -> Result<(), S::Error>
    where
        S: SerializeObject;

    fn try_deserialize<'de, A>(key: &str, map: &mut A) -> Result<OneofMatch<Self>, A::Error>
    where
        A: serde::de::MapAccess<'de>;

    fn matches_field_name(key: &str) -> bool;
}

/// Internal helper trait implemented by prost-generated messages.
#[doc(hidden)]
pub trait ProstMessage: Sized {
    const DENY_UNKNOWN_FIELDS: bool;

    fn serialize_fields<S>(&self, map: &mut S) -> Result<(), S::Error>
    where
        S: SerializeObject;

    fn matches_field_name(key: &str) -> bool;
}

/// Internal helper used to indicate oneof match outcomes.
#[doc(hidden)]
pub enum OneofMatch<T> {
    /// The key did not match any oneof field.
    NoMatch,
    /// The key matched a oneof field; `None` means "skip value".
    Matched(Option<T>),
}

/// Internal helper that abstracts object-like serializers.
#[doc(hidden)]
pub trait SerializeObject {
    type Error: serde::ser::Error;

    fn serialize_entry<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize + ?Sized;
}

/// Internal adapter for serializers that encode anonymous maps.
#[doc(hidden)]
pub struct MapObjectSerializer<S>(S);

impl<S> MapObjectSerializer<S> {
    #[must_use]
    pub fn new(serializer: S) -> Self {
        Self(serializer)
    }
}

impl<S> MapObjectSerializer<S>
where
    S: serde::ser::SerializeMap,
{
    pub fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S> SerializeObject for MapObjectSerializer<S>
where
    S: serde::ser::SerializeMap,
{
    type Error = S::Error;

    fn serialize_entry<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        self.0.serialize_entry(key, value)
    }
}

/// Internal adapter for serializers that encode named structs.
#[doc(hidden)]
pub struct StructObjectSerializer<S>(S);

impl<S> StructObjectSerializer<S> {
    #[must_use]
    pub fn new(serializer: S) -> Self {
        Self(serializer)
    }
}

impl<S> StructObjectSerializer<S>
where
    S: serde::ser::SerializeStruct,
{
    pub fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S> SerializeObject for StructObjectSerializer<S>
where
    S: serde::ser::SerializeStruct,
{
    type Error = S::Error;

    fn serialize_entry<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize + ?Sized,
    {
        self.0.serialize_field(key, value)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use prost_canonical_serde_example::{KitchenSink, Nested, Status, kitchen_sink};
    use std::collections::HashMap;
    use std::string::String;
    use std::vec;

    fn sample_message() -> KitchenSink {
        let mut string_to_int = HashMap::new();
        string_to_int.insert(String::from("alpha"), 1);
        string_to_int.insert(String::from("beta"), 2);

        let mut int_to_string = HashMap::new();
        int_to_string.insert(7, String::from("seven"));

        KitchenSink {
            int32_field: 123,
            int64_field: 9_007_199_254_740_993,
            uint64_field: u64::MAX,
            bool_field: true,
            string_field: String::from("hello"),
            bytes_field: vec![0, 1, 2, 255],
            float_field: 12.5,
            double_field: -3.25,
            status: Status::Active as i32,
            nested: Some(Nested {
                id: 42,
                note: String::from("primary"),
            }),
            repeated_nested: vec![
                Nested {
                    id: 7,
                    note: String::from("first"),
                },
                Nested {
                    id: 8,
                    note: String::from("second"),
                },
            ],
            string_to_int,
            int_to_string,
            choice: Some(kitchen_sink::Choice::Name(String::from("choice name"))),
            timestamp: Some(prost_types::Timestamp {
                seconds: 1_640_995_200,
                nanos: 123_000_000,
            }),
            optional_int32: None,
        }
    }

    #[test]
    fn kitchen_sink_canonical_json_roundtrip() {
        let message = sample_message();
        let json = serde_json::to_string(&message).expect("serialize canonical");
        let decoded: KitchenSink = serde_json::from_str(&json).expect("deserialize canonical");
        assert_eq!(message, decoded);
    }
}
