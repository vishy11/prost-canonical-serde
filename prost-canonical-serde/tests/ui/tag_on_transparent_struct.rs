extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(tag = "type")]
#[prost_canonical_serde(transparent)]
struct TaggedTransparent {
    #[prost(string, tag = "1")]
    value: String,
}

fn main() {}
