extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct InvalidFieldTransparent {
    #[prost(int32, tag = "1")]
    #[prost_canonical_serde(transparent)]
    value: i32,
}

fn main() {}
