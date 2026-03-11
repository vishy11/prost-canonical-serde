extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(deny_unknown_fields)]
struct Inner {
    #[prost(string, tag = "1")]
    value: String,
}

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct Outer {
    #[prost(message, optional, tag = "1")]
    #[prost_canonical_serde(flatten)]
    inner: Option<Inner>,
}

fn main() {}
