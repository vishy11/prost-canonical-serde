extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct InvalidFlattenScalar {
    #[prost(int32, tag = "1")]
    #[prost_canonical_serde(flatten)]
    value: i32,
}

fn main() {}
