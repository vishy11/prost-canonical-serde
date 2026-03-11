extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(transparent)]
struct Transparent {
    #[prost(int32, tag = "1")]
    #[prost_canonical_serde(proto_name = "left", json_name = "left")]
    left: i32,
    #[prost(int32, tag = "2")]
    #[prost_canonical_serde(proto_name = "right", json_name = "right")]
    right: i32,
}

fn main() {}
