extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(untagged)]
enum UntaggedEnum {
    #[prost(string, tag = "1")]
    Value(String),
}

fn main() {}
