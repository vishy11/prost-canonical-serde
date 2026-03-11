# prost-canonical-serde

Canonical JSON encoding for Prost-generated protobuf bindings.

## Why this exists

Protobuf has a canonical JSON mapping that differs from plain Serde JSON (for
example, `int64`/`uint64` are encoded as strings and `bytes` use base64). Prost
provides efficient Rust bindings, but it does not implement canonical JSON on
its own. This project fills that gap by generating `serde::Serialize` and
`serde::Deserialize` implementations that follow the protobuf canonical JSON
spec, while keeping the normal `serde_json` API surface.

## Highlights

- Seamless Prost integration: derive macros and build helpers work with
  prost-generated message types.
- Well-known types support: `prost-types` (Timestamp, Duration, Any, Struct,
  etc.) are handled with their canonical JSON mappings.
- `no_std` friendly: the core crate works without `std` (alloc required).
- High conformance: validated against the upstream protobuf conformance test
  suite. Remaining non-conformance aligns with limitations in Prost itself
  (for example, unknown field preservation and MessageSet support).

## Quick start

Use the derive macros for prost-generated types and keep using `serde_json`.
See the crate documentation on
[docs.rs](https://docs.rs/prost-canonical-serde/latest/prost_canonical_serde/)
for a full end-to-end example with a `.proto`, `build.rs`, and a runnable usage
snippet.

## Additional attributes

`prost_canonical_serde` also supports a small set of serde-style attributes on
top of the generated protobuf metadata.

### `#[prost_canonical_serde(transparent)]`

Apply this to a single-field named struct to serialize and deserialize it as the
inner field's canonical protobuf JSON representation.

```rust
#[derive(prost_canonical_serde::CanonicalSerialize, prost_canonical_serde::CanonicalDeserialize)]
#[prost_canonical_serde(transparent)]
struct Count {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(proto_name = "count", json_name = "count")]
    count: i64,
}
```

### `#[prost_canonical_serde(flatten)]`

Apply this to a message field, including `Option<Message>`, to merge that
message's canonical JSON fields into the parent object.

```rust
#[derive(prost_canonical_serde::CanonicalSerialize, prost_canonical_serde::CanonicalDeserialize)]
struct Metadata {
    #[prost(int64, tag = "1")]
    #[prost_canonical_serde(proto_name = "count", json_name = "count")]
    count: i64,
}

#[derive(prost_canonical_serde::CanonicalSerialize, prost_canonical_serde::CanonicalDeserialize)]
struct Envelope {
    #[prost(string, tag = "1")]
    #[prost_canonical_serde(proto_name = "name", json_name = "name")]
    name: String,
    #[prost(message, optional, tag = "2")]
    #[prost_canonical_serde(flatten)]
    metadata: Option<Metadata>,
}
```

### Current limits

- `transparent` is only supported on single-field named structs.
- `flatten` is only supported on message fields and `Option<Message>`.
- `flatten` cannot be used on `oneof` fields or together with `proto_name` /
  `json_name`.
- If flattened fields collide with outer keys or with each other, deserialization
  prefers non-flattened fields first and then the first matching flattened field
  in declaration order. Serialization emits fields in declaration order, so
  colliding flattened keys can produce duplicate JSON object keys and should be
  avoided.

## License

Apache-2.0. See `LICENSE`.
