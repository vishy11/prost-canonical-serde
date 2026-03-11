use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[prost_canonical_serde(rename_all = "title_case")]
struct InvalidRenameAll {
    #[prost(string, tag = "1")]
    value: String,
}

fn main() {}
