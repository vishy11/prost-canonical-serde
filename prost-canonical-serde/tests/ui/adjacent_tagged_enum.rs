extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentTaggedEnum {
    #[prost(string, tag = "1")]
    Value(String),
}

fn main() {}
