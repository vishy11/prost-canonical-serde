extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct Inner {
    #[prost(int32, tag = "1")]
    #[prost_canonical_serde(proto_name = "value", json_name = "value")]
    value: i32,
}

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct InvalidFlattenName {
    #[prost(message, optional, tag = "1")]
    #[prost_canonical_serde(flatten, proto_name = "inner", json_name = "inner")]
    inner: Option<Inner>,
}

fn main() {}
