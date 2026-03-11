extern crate alloc;

use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};

trait MySerTrait {}
trait MyDeTrait {}

#[derive(CanonicalSerialize, CanonicalDeserialize)]
#[serde(bound(serialize = "T: MySerTrait", deserialize = "T: MyDeTrait"))]
struct UnsupportedBound<T> {
    #[prost(string, tag = "1")]
    value: String,
    marker: core::marker::PhantomData<T>,
}

fn main() {}
