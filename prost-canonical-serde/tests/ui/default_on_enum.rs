extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(default)]
enum DefaultOnEnum {
    #[prost(string, tag = "1")]
    Value(String),
}

fn main() {}
