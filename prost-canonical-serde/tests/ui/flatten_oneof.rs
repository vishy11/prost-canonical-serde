extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct InvalidFlattenOneof {
    #[prost(oneof = "invalid_flatten_oneof::Choice", tags = "1")]
    #[prost_canonical_serde(flatten)]
    choice: Option<invalid_flatten_oneof::Choice>,
}

mod invalid_flatten_oneof {
    use super::*;

    #[derive(CanonicalSerialize, CanonicalDeserialize)]
    pub enum Choice {
        #[prost(int32, tag = "1")]
        Value(i32),
    }
}

fn main() {}
