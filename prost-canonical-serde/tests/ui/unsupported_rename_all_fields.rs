use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(rename_all_fields = "snake_case")]
enum UnsupportedRenameAllFields {
    #[prost(string, tag = "1")]
    Value(String),
}

fn main() {}
