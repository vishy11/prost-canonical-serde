//! Proc macros that derive canonical JSON serde implementations for prost types.
//!
//! These derives implement `serde::Serialize` and `serde::Deserialize` using
//! canonical protobuf JSON rules, so callers can keep using `serde_json`
//! normally.
//!
//! # Example
//! ```rust,ignore
//! use prost_canonical_serde::{CanonicalDeserialize, CanonicalSerialize};
//!
//! #[derive(CanonicalSerialize, CanonicalDeserialize)]
//! struct Example {
//!     #[prost(int32, tag = "1")]
//!     #[prost_canonical_serde(proto_name = "value", json_name = "value")]
//!     value: i32,
//! }
//!
//! let json = serde_json::to_string(&Example { value: 1 }).unwrap();
//! ```
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Ident, LitStr, Path,
    Token, Type, TypePath,
};

/// Derives `CanonicalSerialize` and `serde::Serialize` for prost messages.
#[proc_macro_derive(CanonicalSerialize, attributes(prost, prost_canonical_serde, serde))]
pub fn derive_canonical_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_serialize(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derives `CanonicalDeserialize` and `serde::Deserialize` for prost messages.
#[proc_macro_derive(CanonicalDeserialize, attributes(prost, prost_canonical_serde, serde))]
pub fn derive_canonical_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_deserialize(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_serialize(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        Data::Struct(data) => expand_serialize_struct(input, data),
        Data::Enum(data) => expand_serialize_enum(input, data),
        Data::Union(_) => Err(syn::Error::new(
            input.span(),
            "CanonicalSerialize does not support unions",
        )),
    }
}

fn expand_deserialize(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        Data::Struct(data) => expand_deserialize_struct(input, data),
        Data::Enum(data) => expand_deserialize_enum(input, data),
        Data::Union(_) => Err(syn::Error::new(
            input.span(),
            "CanonicalDeserialize does not support unions",
        )),
    }
}

fn wrap_with_serde_path(
    tokens: proc_macro2::TokenStream,
    serde_path: &Path,
) -> proc_macro2::TokenStream {
    quote! {
        const _: () = {
            use #serde_path as __pcs_serde;
            #tokens
        };
    }
}

fn expand_serialize_struct(
    input: &DeriveInput,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let container_attrs = parse_container_attrs(input)?;
    if let Some(tokens) = expand_serialize_conversion(name, &container_attrs.serialize_via) {
        return Ok(wrap_with_serde_path(tokens, &container_attrs.serde_path));
    }
    let fields = extract_fields(&data.fields, &container_attrs)?;
    let serialize_name = LitStr::new(&container_attrs.serialize_name, name.span());
    let has_flatten = fields.iter().any(|field| field.is_flatten);
    let deny_unknown_fields = container_attrs.deny_unknown_fields;
    let tag_key = container_attrs
        .tag
        .as_ref()
        .map(|tag| LitStr::new(tag, name.span()));

    if container_attrs.transparent {
        if container_attrs.tag.is_some() {
            return Err(syn::Error::new(
                input.span(),
                "tag is only supported on structs with named fields",
            ));
        }
        return expand_serialize_transparent_struct(name, &serialize_name, &fields)
            .map(|tokens| wrap_with_serde_path(tokens, &container_attrs.serde_path));
    }

    let mut field_serializers = Vec::new();
    let mut field_match_arms = Vec::new();
    let mut field_count_stmts = Vec::new();

    for field in &fields {
        field_serializers.push(serialize_field(field));
        field_match_arms.push(field_match_arm(field)?);
        field_count_stmts.push(serialized_field_count_stmt(field));
    }
    let tag_serializer = if let Some(tag_key) = &tag_key {
        quote! {
            map.serialize_entry(#tag_key, #serialize_name)?;
        }
    } else {
        quote! {}
    };
    let tag_match_arm = if let Some(tag_key) = &tag_key {
        quote! {
            #tag_key => true,
        }
    } else {
        quote! {}
    };
    let tag_count_stmt = if tag_key.is_some() {
        quote! {
            __pcs_len += 1;
        }
    } else {
        quote! {}
    };

    Ok(wrap_with_serde_path(quote! {
        impl ::prost_canonical_serde::ProstMessage for #name {
            const DENY_UNKNOWN_FIELDS: bool = #deny_unknown_fields;

            fn serialize_fields<S>(&self, map: &mut S) -> Result<(), S::Error>
            where
                S: ::prost_canonical_serde::SerializeObject,
            {
                #tag_serializer
                #(#field_serializers)*
                Ok(())
            }

            fn matches_field_name(key: &str) -> bool {
                match key {
                    #tag_match_arm
                    #(#field_match_arms)*
                    _ => false,
                }
            }
        }

        impl ::prost_canonical_serde::CanonicalSerialize for #name {
            fn serialize_canonical<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                if #has_flatten {
                    use __pcs_serde::ser::SerializeMap;
                    let mut map =
                        ::prost_canonical_serde::MapObjectSerializer::new(serializer.serialize_map(None)?);
                    <Self as ::prost_canonical_serde::ProstMessage>::serialize_fields(self, &mut map)?;
                    map.end()
                } else {
                    use __pcs_serde::ser::SerializeStruct;
                    let mut __pcs_len = 0usize;
                    #tag_count_stmt
                    #(#field_count_stmts)*
                    let mut map = ::prost_canonical_serde::StructObjectSerializer::new(
                        serializer.serialize_struct(#serialize_name, __pcs_len)?,
                    );
                    <Self as ::prost_canonical_serde::ProstMessage>::serialize_fields(self, &mut map)?;
                    map.end()
                }
            }
        }

        impl __pcs_serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                <Self as ::prost_canonical_serde::CanonicalSerialize>::serialize_canonical(
                    self,
                    serializer,
                )
            }
        }
    }, &container_attrs.serde_path))
}

fn expand_serialize_conversion(
    name: &Ident,
    serialize_via: &SerializeVia,
) -> Option<proc_macro2::TokenStream> {
    let into_ty = match serialize_via {
        SerializeVia::None => return None,
        SerializeVia::Into(into_ty) => into_ty,
    };

    Some(quote! {
        impl ::prost_canonical_serde::CanonicalSerialize for #name {
            fn serialize_canonical<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                let __pcs_value: #into_ty = <Self as ::core::clone::Clone>::clone(self).into();
                __pcs_serde::Serialize::serialize(&__pcs_value, serializer)
            }
        }

        impl __pcs_serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                <Self as ::prost_canonical_serde::CanonicalSerialize>::serialize_canonical(
                    self,
                    serializer,
                )
            }
        }
    })
}

fn expand_deserialize_struct(
    input: &DeriveInput,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let container_attrs = parse_container_attrs(input)?;
    if let Some(tokens) = expand_deserialize_conversion(name, &container_attrs.deserialize_via) {
        return Ok(wrap_with_serde_path(tokens, &container_attrs.serde_path));
    }
    let map_ident = Ident::new("__pcs_map", Span::call_site());
    let key_cow_ident = Ident::new("__pcs_key", Span::call_site());
    let key_str_ident = Ident::new("__pcs_key_str", Span::call_site());
    let oneof_value_ident = Ident::new("__pcs_oneof_value", Span::call_site());
    let fields = extract_fields(&data.fields, &container_attrs)?;
    let deserialize_name = LitStr::new(&container_attrs.deserialize_name, name.span());
    let has_flatten = fields.iter().any(|field| field.is_flatten);
    let deny_unknown_fields = container_attrs.deny_unknown_fields;
    let tag_key = container_attrs
        .tag
        .as_ref()
        .map(|tag| LitStr::new(tag, name.span()));
    let default_ident = Ident::new("__pcs_default", Span::call_site());

    if container_attrs.transparent {
        if container_attrs.tag.is_some() {
            return Err(syn::Error::new(
                input.span(),
                "tag is only supported on structs with named fields",
            ));
        }
        return expand_deserialize_transparent_struct(name, &deserialize_name, &fields)
            .map(|tokens| wrap_with_serde_path(tokens, &container_attrs.serde_path));
    }

    if deny_unknown_fields && has_flatten {
        return Err(syn::Error::new(
            input.span(),
            "deny_unknown_fields is not supported with flatten",
        ));
    }

    let mut field_inits = Vec::new();
    let mut field_names = Vec::new();
    let mut match_arms = Vec::new();
    let mut oneof_checks = Vec::new();
    let mut flatten_checks = Vec::new();
    let mut flatten_finishes = Vec::new();
    let mut flatten_deny_guards = Vec::new();
    let tag_seen_ident = Ident::new("__pcs_tag_seen", Span::call_site());
    let tag_value_ident = Ident::new("__pcs_tag_value", Span::call_site());

    for field in &fields {
        let ident = field.ident.clone();
        field_names.push(ident.clone());
        field_inits.push(init_field(
            field,
            Some(&container_attrs.default),
            &default_ident,
        ));

        if field.is_flatten {
            field_inits.push(init_flatten_buffer(field)?);
            flatten_checks.push(flatten_match_arm(field, &key_str_ident, &map_ident)?);
            flatten_finishes.push(finish_flatten_field(field)?);
            flatten_deny_guards.push(flatten_deny_unknown_guard(field)?);
            continue;
        }

        if field.is_oneof {
            let oneof_type = field
                .oneof_type
                .as_ref()
                .ok_or_else(|| syn::Error::new(ident.span(), "oneof field must be Option"))?;
            oneof_checks.push(quote! {
                match <#oneof_type as ::prost_canonical_serde::ProstOneof>::try_deserialize(
                    #key_str_ident,
                    &mut #map_ident,
                )? {
                    ::prost_canonical_serde::OneofMatch::Matched(Some(#oneof_value_ident)) => {
                        if #ident.is_some() {
                            return Err(__pcs_serde::de::Error::custom("multiple oneof fields set"));
                        }
                        #ident = Some(#oneof_value_ident);
                        continue;
                    }
                    ::prost_canonical_serde::OneofMatch::Matched(None) => {
                        continue;
                    }
                    ::prost_canonical_serde::OneofMatch::NoMatch => {}
                }
            });
        } else {
            match_arms.push(deserialize_match_arm(field, &map_ident)?);
        }
    }
    let tag_init = if tag_key.is_some() {
        quote! {
            let mut #tag_seen_ident = false;
        }
    } else {
        quote! {}
    };
    let tag_check = if let Some(tag_key) = &tag_key {
        quote! {
            if #key_str_ident == #tag_key {
                let #tag_value_ident = #map_ident.next_value::<::alloc::borrow::Cow<'de, str>>()?;
                if #tag_value_ident.as_ref() != #deserialize_name {
                    return Err(__pcs_serde::de::Error::custom("invalid struct tag"));
                }
                #tag_seen_ident = true;
                continue;
            }
        }
    } else {
        quote! {}
    };
    let tag_finish = if tag_key.is_some() {
        quote! {
            if !#tag_seen_ident {
                return Err(__pcs_serde::de::Error::missing_field(#tag_key));
            }
        }
    } else {
        quote! {}
    };
    let container_default_init = match &container_attrs.default {
        ContainerDefault::None => quote! {},
        ContainerDefault::Default => quote! {
            let #default_ident: #name = ::core::default::Default::default();
        },
        ContainerDefault::Path(path) => quote! {
            let #default_ident: #name = #path();
        },
    };

    Ok(wrap_with_serde_path(quote! {
        #(#flatten_deny_guards)*

        impl ::prost_canonical_serde::CanonicalDeserialize for #name {
            fn deserialize_canonical<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> __pcs_serde::de::Visitor<'de> for Visitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        formatter.write_str("map")
                    }

                    fn visit_map<A>(self, mut #map_ident: A) -> Result<Self::Value, A::Error>
                    where
                        A: __pcs_serde::de::MapAccess<'de>,
                    {
                        #tag_init
                        #container_default_init
                        #(#field_inits)*

                        while let Some(#key_cow_ident) = #map_ident.next_key::<::alloc::borrow::Cow<'de, str>>()? {
                            let #key_str_ident = #key_cow_ident.as_ref();
                            #tag_check
                            #(#oneof_checks)*
                            match #key_str_ident {
                                #(#match_arms)*
                                _ => {
                                    #(#flatten_checks)*
                                    if #deny_unknown_fields {
                                        return Err(__pcs_serde::de::Error::unknown_field(#key_str_ident, &[]));
                                    }
                                    let _ = #map_ident.next_value::<__pcs_serde::de::IgnoredAny>()?;
                                }
                            }
                        }

                        #tag_finish
                        #(#flatten_finishes)*

                        Ok(#name {
                            #(#field_names),*
                        })
                    }
                }

                if #has_flatten {
                    deserializer.deserialize_map(Visitor)
                } else {
                    deserializer.deserialize_struct(#deserialize_name, &[], Visitor)
                }
            }
        }

        impl<'de> __pcs_serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                <Self as ::prost_canonical_serde::CanonicalDeserialize>::deserialize_canonical(
                    deserializer,
                )
            }
        }
    }, &container_attrs.serde_path))
}

fn expand_serialize_transparent_struct(
    name: &Ident,
    serialize_name: &LitStr,
    fields: &[FieldInfo],
) -> syn::Result<proc_macro2::TokenStream> {
    let field = fields
        .first()
        .ok_or_else(|| syn::Error::new(name.span(), "transparent structs must have one field"))?;
    if fields.len() != 1 {
        return Err(syn::Error::new(
            name.span(),
            "transparent structs must have exactly one field",
        ));
    }
    if field.is_oneof || field.is_flatten {
        return Err(syn::Error::new(
            field.ident.span(),
            "transparent fields cannot also be oneof or flatten",
        ));
    }

    let serialize_expr = serialize_transparent_field(field, serialize_name);

    Ok(quote! {
        impl ::prost_canonical_serde::CanonicalSerialize for #name {
            fn serialize_canonical<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                #serialize_expr
            }
        }

        impl __pcs_serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                <Self as ::prost_canonical_serde::CanonicalSerialize>::serialize_canonical(
                    self,
                    serializer,
                )
            }
        }
    })
}

fn expand_deserialize_transparent_struct(
    name: &Ident,
    deserialize_name: &LitStr,
    fields: &[FieldInfo],
) -> syn::Result<proc_macro2::TokenStream> {
    let field = fields
        .first()
        .ok_or_else(|| syn::Error::new(name.span(), "transparent structs must have one field"))?;
    if fields.len() != 1 {
        return Err(syn::Error::new(
            name.span(),
            "transparent structs must have exactly one field",
        ));
    }
    if field.is_oneof || field.is_flatten {
        return Err(syn::Error::new(
            field.ident.span(),
            "transparent fields cannot also be oneof or flatten",
        ));
    }

    let ident = &field.ident;
    let deserialize_expr = deserialize_transparent_field(field)?;

    Ok(quote! {
        impl ::prost_canonical_serde::CanonicalDeserialize for #name {
            fn deserialize_canonical<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> __pcs_serde::de::Visitor<'de> for Visitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        formatter.write_str("newtype struct")
                    }

                    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: __pcs_serde::Deserializer<'de>,
                    {
                        let #ident = #deserialize_expr;
                        Ok(#name { #ident })
                    }
                }

                deserializer.deserialize_newtype_struct(#deserialize_name, Visitor)
            }
        }

        impl<'de> __pcs_serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                <Self as ::prost_canonical_serde::CanonicalDeserialize>::deserialize_canonical(
                    deserializer,
                )
            }
        }
    })
}

fn expand_deserialize_conversion(
    name: &Ident,
    deserialize_via: &DeserializeVia,
) -> Option<proc_macro2::TokenStream> {
    let body = match deserialize_via {
        DeserializeVia::None => return None,
        DeserializeVia::From(from_ty) => {
            quote! {
                let __pcs_value = <#from_ty as __pcs_serde::Deserialize<'de>>::deserialize(deserializer)?;
                Ok(<Self as ::core::convert::From<#from_ty>>::from(__pcs_value))
            }
        }
        DeserializeVia::TryFrom(from_ty) => {
            quote! {
                let __pcs_value = <#from_ty as __pcs_serde::Deserialize<'de>>::deserialize(deserializer)?;
                <Self as ::core::convert::TryFrom<#from_ty>>::try_from(__pcs_value)
                    .map_err(__pcs_serde::de::Error::custom)
            }
        }
    };

    Some(quote! {
        impl ::prost_canonical_serde::CanonicalDeserialize for #name {
            fn deserialize_canonical<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                #body
            }
        }

        impl<'de> __pcs_serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                <Self as ::prost_canonical_serde::CanonicalDeserialize>::deserialize_canonical(
                    deserializer,
                )
            }
        }
    })
}

fn expand_serialize_enum(
    input: &DeriveInput,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let container_attrs = parse_container_attrs(input)?;
    if is_oneof_enum(data) {
        let oneof_impl = expand_oneof_impl(input, data, &container_attrs)?;
        if let Some(tokens) = expand_serialize_conversion(name, &container_attrs.serialize_via) {
            return Ok(wrap_with_serde_path(quote! {
                #oneof_impl
                #tokens
            }, &container_attrs.serde_path));
        }
        let serialize_name = LitStr::new(&container_attrs.serialize_name, name.span());
        return Ok(wrap_with_serde_path(quote! {
            #oneof_impl
            impl ::prost_canonical_serde::CanonicalSerialize for #name {
                fn serialize_canonical<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: __pcs_serde::Serializer,
                {
                    use __pcs_serde::ser::SerializeStruct;
                    let mut map = ::prost_canonical_serde::StructObjectSerializer::new(
                        serializer.serialize_struct(#serialize_name, 1)?,
                    );
                    <Self as ::prost_canonical_serde::ProstOneof>::serialize_field(self, &mut map)?;
                    map.end()
                }
            }

            impl __pcs_serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: __pcs_serde::Serializer,
                {
                    <Self as ::prost_canonical_serde::CanonicalSerialize>::serialize_canonical(
                        self,
                        serializer,
                    )
                }
            }
        }, &container_attrs.serde_path));
    }
    if let Some(tokens) = expand_serialize_conversion(name, &container_attrs.serialize_via) {
        return Ok(wrap_with_serde_path(tokens, &container_attrs.serde_path));
    }
    if !matches!(&container_attrs.default, ContainerDefault::None) {
        return Err(syn::Error::new(
            input.span(),
            "default is only supported on structs in prost-canonical-serde",
        ));
    }
    if container_attrs.tag.is_some() {
        return Err(syn::Error::new(
            input.span(),
            "tag is not supported on enums in prost-canonical-serde",
        ));
    }
    Ok(wrap_with_serde_path(quote! {
        impl ::prost_canonical_serde::ProstEnum for #name {
            fn from_i32(value: i32) -> ::core::option::Option<Self> {
                Self::try_from(value).ok()
            }

            fn from_str_name(value: &str) -> ::core::option::Option<Self> {
                #name::from_str_name(value)
            }

            fn as_str_name(&self) -> &'static str {
                self.as_str_name()
            }

            fn as_i32(&self) -> i32 {
                *self as i32
            }
        }

        impl ::prost_canonical_serde::CanonicalSerialize for #name {
            fn serialize_canonical<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                serializer.serialize_str(self.as_str_name())
            }
        }

        impl __pcs_serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: __pcs_serde::Serializer,
            {
                <Self as ::prost_canonical_serde::CanonicalSerialize>::serialize_canonical(
                    self,
                    serializer,
                )
            }
        }
    }, &container_attrs.serde_path))
}

fn expand_deserialize_enum(
    input: &DeriveInput,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let container_attrs = parse_container_attrs(input)?;
    if let Some(tokens) = expand_deserialize_conversion(name, &container_attrs.deserialize_via) {
        return Ok(wrap_with_serde_path(tokens, &container_attrs.serde_path));
    }
    if !matches!(&container_attrs.default, ContainerDefault::None) {
        return Err(syn::Error::new(
            input.span(),
            "default is only supported on structs in prost-canonical-serde",
        ));
    }
    if container_attrs.tag.is_some() {
        return Err(syn::Error::new(
            input.span(),
            "tag is not supported on enums in prost-canonical-serde",
        ));
    }
    if is_oneof_enum(data) {
        let deserialize_name = LitStr::new(&container_attrs.deserialize_name, name.span());
        let deny_unknown_fields = container_attrs.deny_unknown_fields;
        let map_ident = Ident::new("__pcs_map", Span::call_site());
        let key_cow_ident = Ident::new("__pcs_key", Span::call_site());
        let key_str_ident = Ident::new("__pcs_key_str", Span::call_site());
        let value_ident = Ident::new("__pcs_value", Span::call_site());
        let found_ident = Ident::new("__pcs_found", Span::call_site());
        return Ok(wrap_with_serde_path(quote! {
            impl ::prost_canonical_serde::CanonicalDeserialize for #name {
                fn deserialize_canonical<'de, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: __pcs_serde::Deserializer<'de>,
                {
                    struct Visitor;

                    impl<'de> __pcs_serde::de::Visitor<'de> for Visitor {
                        type Value = #name;

                        fn expecting(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            formatter.write_str("map")
                        }

                        fn visit_map<A>(self, mut #map_ident: A) -> Result<Self::Value, A::Error>
                        where
                            A: __pcs_serde::de::MapAccess<'de>,
                        {
                            let mut #found_ident = None;
                            while let Some(#key_cow_ident) = #map_ident.next_key::<::alloc::borrow::Cow<'de, str>>()? {
                                let #key_str_ident = #key_cow_ident.as_ref();
                                match <#name as ::prost_canonical_serde::ProstOneof>::try_deserialize(
                                    #key_str_ident,
                                    &mut #map_ident,
                                )? {
                                    ::prost_canonical_serde::OneofMatch::Matched(Some(#value_ident)) => {
                                        if #found_ident.is_some() {
                                            return Err(__pcs_serde::de::Error::custom(
                                                "multiple oneof fields set",
                                            ));
                                        }
                                        #found_ident = Some(#value_ident);
                                        continue;
                                    }
                                    ::prost_canonical_serde::OneofMatch::Matched(None) => {
                                        continue;
                                    }
                                    ::prost_canonical_serde::OneofMatch::NoMatch => {
                                        if #deny_unknown_fields {
                                            return Err(__pcs_serde::de::Error::unknown_field(#key_str_ident, &[]));
                                        }
                                        let _ = #map_ident.next_value::<__pcs_serde::de::IgnoredAny>()?;
                                    }
                                }
                            }

                            #found_ident.ok_or_else(|| __pcs_serde::de::Error::custom("expected oneof field"))
                        }
                    }

                    deserializer.deserialize_struct(#deserialize_name, &[], Visitor)
            }
        }

        impl<'de> __pcs_serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                <Self as ::prost_canonical_serde::CanonicalDeserialize>::deserialize_canonical(
                    deserializer,
                )
            }
        }
        }, &container_attrs.serde_path));
    }

    Ok(wrap_with_serde_path(quote! {
        impl ::prost_canonical_serde::CanonicalDeserialize for #name {
            fn deserialize_canonical<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                let value = <::prost_canonical_serde::CanonicalEnumValue<#name> as __pcs_serde::Deserialize>::deserialize(
                    deserializer,
                )?
                .0;
                #name::from_i32(value)
                    .ok_or_else(|| __pcs_serde::de::Error::custom("unknown enum number"))
            }
        }

        impl<'de> __pcs_serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: __pcs_serde::Deserializer<'de>,
            {
                <Self as ::prost_canonical_serde::CanonicalDeserialize>::deserialize_canonical(
                    deserializer,
                )
            }
        }
    }, &container_attrs.serde_path))
}

fn expand_oneof_impl(
    input: &DeriveInput,
    data: &syn::DataEnum,
    container_attrs: &ContainerAttrs,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mut serialize_arms = Vec::new();
    let mut deserialize_arms = Vec::new();
    let mut matches_arms = Vec::new();

    for variant in &data.variants {
        let ident = &variant.ident;
        let attrs = parse_canonical_attrs(&variant.attrs)?;
        if attrs.transparent || attrs.flatten {
            return Err(syn::Error::new(
                ident.span(),
                "transparent and flatten are not supported on oneof variants",
            ));
        }
        let (value_ty, kind, enum_path) = parse_variant(variant)?;
        let fallback = RenameRule::CamelCase.apply_to_field(&ident.to_string());
        let proto_name = attrs.proto_name.clone().unwrap_or_else(|| fallback.clone());
        let serialize_name = name_for_variant(
            ident,
            &attrs,
            container_attrs.serialize_rename_all,
            &proto_name,
        );
        let deserialize_name = name_for_variant(
            ident,
            &attrs,
            container_attrs.deserialize_rename_all,
            &proto_name,
        );
        let serialize_name_literal = LitStr::new(&serialize_name, ident.span());
        let deserialize_name_literal = LitStr::new(&deserialize_name, ident.span());
        let proto_name_literal = LitStr::new(&proto_name, ident.span());
        let value_ident = Ident::new("value", ident.span());

        let serialize_expr = serialize_value_expr(&kind, &value_ident, enum_path.as_ref());
        let deserialize_expr = if let Kind::Enum(path) = &kind {
            let path = enum_path.as_ref().unwrap_or(path);
            quote! {
                map.next_value::<::prost_canonical_serde::CanonicalEnumOption<#path>>()?.0
            }
        } else {
            quote! {
                map.next_value::<::prost_canonical_serde::CanonicalOption<#value_ty>>()?.0
            }
        };

        serialize_arms.push(quote! {
            Self::#ident(#value_ident) => {
                let value = #serialize_expr;
                map.serialize_entry(#serialize_name_literal, &value)?;
            }
        });

        let match_pat = if deserialize_name == proto_name {
            quote! { #deserialize_name_literal }
        } else {
            quote! { #deserialize_name_literal | #proto_name_literal }
        };

        deserialize_arms.push(quote! {
            #match_pat => {
                let value = #deserialize_expr;
                Ok(::prost_canonical_serde::OneofMatch::Matched(value.map(Self::#ident)))
            }
        });
        matches_arms.push(quote! {
            #match_pat => true
        });
    }

    Ok(quote! {
        impl ::prost_canonical_serde::ProstOneof for #name {
            fn serialize_field<S>(&self, map: &mut S) -> Result<(), S::Error>
            where
                S: ::prost_canonical_serde::SerializeObject,
            {
                match self {
                    #(#serialize_arms),*
                }
                Ok(())
            }

            fn try_deserialize<'de, A>(key: &str, map: &mut A) -> Result<::prost_canonical_serde::OneofMatch<Self>, A::Error>
            where
                A: __pcs_serde::de::MapAccess<'de>,
            {
                match key {
                    #(#deserialize_arms),*,
                    _ => Ok(::prost_canonical_serde::OneofMatch::NoMatch),
                }
            }

            fn matches_field_name(key: &str) -> bool {
                match key {
                    #(#matches_arms),*,
                    _ => false,
                }
            }
        }
    })
}

fn serialize_field(field: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = &field.ident;
    let serialize_name = LitStr::new(&field.serialize_name, ident.span());

    if field.is_flatten {
        let target_ty = field.flatten_target_ty();
        return if field.is_option_message() {
            quote! {
                if let Some(value) = &self.#ident {
                    <#target_ty as ::prost_canonical_serde::ProstMessage>::serialize_fields(value, map)?;
                }
            }
        } else {
            quote! {
                <#target_ty as ::prost_canonical_serde::ProstMessage>::serialize_fields(&self.#ident, map)?;
            }
        };
    }

    if field.is_oneof {
        return quote! {
            if let Some(value) = &self.#ident {
                ::prost_canonical_serde::ProstOneof::serialize_field(value, map)?;
            }
        };
    }

    match &field.kind {
        Kind::Option(inner) => {
            let value_expr = serialize_value_expr(
                inner,
                &Ident::new("value", ident.span()),
                field.enum_path.as_ref(),
            );
            quote! {
                if let Some(value) = &self.#ident {
                    let value = #value_expr;
                    map.serialize_entry(#serialize_name, &value)?;
                }
            }
        }
        Kind::Vec(inner) => {
            let value_stmt = if let Kind::Enum(path) = inner.as_ref() {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalEnumSeq::<#path>::new(&self.#ident);
                    map.serialize_entry(#serialize_name, &value)?;
                }
            } else {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalSeq::new(&self.#ident);
                    map.serialize_entry(#serialize_name, &value)?;
                }
            };

            quote! {
                if !self.#ident.is_empty() {
                    #value_stmt
                }
            }
        }
        Kind::Map(_, _, value_kind) => {
            let value_stmt = if let Kind::Enum(path) = value_kind.as_ref() {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalEnumMapRef::<#path, _>::new(&self.#ident);
                    map.serialize_entry(#serialize_name, &value)?;
                }
            } else {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalMapRef::new(&self.#ident);
                    map.serialize_entry(#serialize_name, &value)?;
                }
            };

            quote! {
                if !self.#ident.is_empty() {
                    #value_stmt
                }
            }
        }
        _ => {
            let value_expr = serialize_value_expr(
                &field.kind,
                &Ident::new("value", ident.span()),
                field.enum_path.as_ref(),
            );
            let field_expr = quote! { self.#ident };
            let default_check = default_check_expr(&field.kind, &field_expr);
            quote! {
                if #default_check {
                    let value = &self.#ident;
                    let value = #value_expr;
                    map.serialize_entry(#serialize_name, &value)?;
                }
            }
        }
    }
}

fn serialize_transparent_field(
    field: &FieldInfo,
    serialize_name: &LitStr,
) -> proc_macro2::TokenStream {
    let ident = &field.ident;

    match &field.kind {
        Kind::Option(inner) => {
            let value_expr = serialize_value_expr(
                inner,
                &Ident::new("value", ident.span()),
                field.enum_path.as_ref(),
            );
            quote! {
                if let Some(value) = &self.#ident {
                    let value = #value_expr;
                    serializer.serialize_newtype_struct(#serialize_name, &value)
                } else {
                    serializer.serialize_newtype_struct(#serialize_name, &())
                }
            }
        }
        Kind::Vec(inner) => {
            if let Kind::Enum(path) = inner.as_ref() {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalEnumSeq::<#path>::new(&self.#ident);
                    serializer.serialize_newtype_struct(#serialize_name, &value)
                }
            } else {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalSeq::new(&self.#ident);
                    serializer.serialize_newtype_struct(#serialize_name, &value)
                }
            }
        }
        Kind::Map(_, _, value_kind) => {
            if let Kind::Enum(path) = value_kind.as_ref() {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalEnumMapRef::<#path, _>::new(&self.#ident);
                    serializer.serialize_newtype_struct(#serialize_name, &value)
                }
            } else {
                quote! {
                    let value = ::prost_canonical_serde::CanonicalMapRef::new(&self.#ident);
                    serializer.serialize_newtype_struct(#serialize_name, &value)
                }
            }
        }
        _ => {
            let value_expr = serialize_value_expr(
                &field.kind,
                &Ident::new("value", ident.span()),
                field.enum_path.as_ref(),
            );
            quote! {
                let value = &self.#ident;
                let value = #value_expr;
                serializer.serialize_newtype_struct(#serialize_name, &value)
            }
        }
    }
}

fn init_field(
    field: &FieldInfo,
    container_default: Option<&ContainerDefault>,
    default_ident: &Ident,
) -> proc_macro2::TokenStream {
    let ident = &field.ident;

    if !matches!(container_default, None | Some(ContainerDefault::None)) {
        return quote! {
            let mut #ident = #default_ident.#ident;
        };
    }

    if field.is_oneof {
        return quote! {
            let mut #ident = ::core::option::Option::None;
        };
    }

    match &field.kind {
        Kind::Option(_) => quote! {
            let mut #ident = ::core::option::Option::None;
        },
        Kind::Vec(_) => quote! {
            let mut #ident = ::alloc::vec::Vec::new();
        },
        Kind::Map(map_kind, _, _) => {
            let map_new = map_new_expr(map_kind);
            quote! {
                let mut #ident = #map_new;
            }
        }
        _ => {
            let default_expr = default_value_expr(&field.kind);
            quote! {
                let mut #ident = #default_expr;
            }
        }
    }
}

fn init_flatten_buffer(field: &FieldInfo) -> syn::Result<proc_macro2::TokenStream> {
    let flatten_ident = field.flatten_buffer_ident()?;
    Ok(quote! {
        let mut #flatten_ident = ::alloc::vec::Vec::<(
            ::alloc::string::String,
            ::prost_canonical_serde::BufferedValue,
        )>::new();
    })
}

fn flatten_match_arm(
    field: &FieldInfo,
    key_ident: &Ident,
    map_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let flatten_ident = field.flatten_buffer_ident()?;
    let target_ty = field.flatten_target_ty();
    Ok(quote! {
        if <#target_ty as ::prost_canonical_serde::ProstMessage>::matches_field_name(#key_ident) {
            let value = #map_ident.next_value::<::prost_canonical_serde::BufferedValue>()?;
            #flatten_ident.push((::alloc::borrow::ToOwned::to_owned(#key_ident), value));
            continue;
        }
    })
}

fn finish_flatten_field(field: &FieldInfo) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &field.ident;
    let flatten_ident = field.flatten_buffer_ident()?;
    let target_ty = field.flatten_target_ty();

    if field.is_option_message() {
        Ok(quote! {
            if !#flatten_ident.is_empty() {
                #ident = Some(
                    <::prost_canonical_serde::CanonicalValue<#target_ty> as __pcs_serde::Deserialize>::deserialize(
                        ::prost_canonical_serde::BufferedValue::Map(#flatten_ident),
                    )
                    .map_err(__pcs_serde::de::Error::custom)?
                    .0,
                );
            }
        })
    } else {
        Ok(quote! {
            if !#flatten_ident.is_empty() {
                #ident =
                    <::prost_canonical_serde::CanonicalValue<#target_ty> as __pcs_serde::Deserialize>::deserialize(
                        ::prost_canonical_serde::BufferedValue::Map(#flatten_ident),
                    )
                    .map_err(__pcs_serde::de::Error::custom)?
                    .0;
            }
        })
    }
}

fn flatten_deny_unknown_guard(field: &FieldInfo) -> syn::Result<proc_macro2::TokenStream> {
    let target_ty = field.flatten_target_ty();
    Ok(quote! {
        const _: () = {
            if <#target_ty as ::prost_canonical_serde::ProstMessage>::DENY_UNKNOWN_FIELDS {
                panic!("deny_unknown_fields is not supported with flatten");
            }
        };
    })
}

fn deserialize_match_arm(
    field: &FieldInfo,
    map_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &field.ident;
    let value_ident = Ident::new("__pcs_value", Span::call_site());
    let deserialize_name = LitStr::new(&field.deserialize_name, ident.span());
    let proto_name = LitStr::new(&field.proto_name, ident.span());
    let ty = &field.ty;
    let match_pat = if field.deserialize_name == field.proto_name {
        quote! { #deserialize_name }
    } else {
        quote! { #deserialize_name | #proto_name }
    };

    match &field.kind {
        Kind::Option(inner) => {
            let inner_ty = field
                .option_inner
                .as_ref()
                .ok_or_else(|| syn::Error::new(ident.span(), "missing Option inner type"))?;
            if is_prost_value_type(inner_ty) {
                return Ok(quote! {
                    #match_pat => {
                        #ident = Some(
                            #map_ident.next_value::<::prost_canonical_serde::CanonicalValue<#inner_ty>>()?
                                .0,
                        );
                    }
                });
            }
            let value_expr = if let Kind::Enum(path) = inner.as_ref() {
                let path = field.enum_path.as_ref().unwrap_or(path);
                quote! {
                    #map_ident.next_value::<::prost_canonical_serde::CanonicalEnumOption<#path>>()?.0
                }
            } else {
                quote! {
                    #map_ident.next_value::<::prost_canonical_serde::CanonicalOption<#inner_ty>>()?.0
                }
            };
            Ok(quote! {
                #match_pat => {
                    #ident = #value_expr;
                }
            })
        }
        Kind::Vec(inner) => {
            if let Kind::Enum(path) = inner.as_ref() {
                return Ok(quote! {
                    #match_pat => {
                        #ident = #map_ident
                            .next_value::<::prost_canonical_serde::CanonicalEnumVec<#path>>()?
                            .0;
                    }
                });
            }
            let inner_ty = field
                .vec_inner
                .as_ref()
                .ok_or_else(|| syn::Error::new(ident.span(), "missing Vec inner type"))?;
            Ok(quote! {
                #match_pat => {
                    #ident = #map_ident
                        .next_value::<::prost_canonical_serde::CanonicalVec<#inner_ty>>()?
                        .0;
                }
            })
        }
        Kind::Map(_, _, value_kind) => {
            let value_expr = if let Kind::Enum(path) = value_kind.as_ref() {
                quote! {
                    #map_ident.next_value::<::prost_canonical_serde::CanonicalEnumMap<#path, #ty>>()?.0
                }
            } else {
                quote! {
                    #map_ident.next_value::<::prost_canonical_serde::CanonicalMap<#ty>>()?.0
                }
            };
            Ok(quote! {
                #match_pat => {
                    #ident = #value_expr;
                }
            })
        }
        Kind::Enum(path) => {
            let path = field.enum_path.as_ref().unwrap_or(path);
            Ok(quote! {
                #match_pat => {
                    if let Some(#value_ident) = #map_ident
                        .next_value::<::prost_canonical_serde::CanonicalEnumOption<#path>>()?
                        .0
                    {
                        #ident = #value_ident;
                    }
                }
            })
        }
        _ => Ok(quote! {
            #match_pat => {
                if let Some(#value_ident) = #map_ident
                    .next_value::<::prost_canonical_serde::CanonicalOption<#ty>>()?
                    .0
                {
                    #ident = #value_ident;
                }
            }
        }),
    }
}

fn deserialize_transparent_field(field: &FieldInfo) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &field.ident;
    let ty = &field.ty;

    match &field.kind {
        Kind::Option(inner) => {
            let inner_ty = field
                .option_inner
                .as_ref()
                .ok_or_else(|| syn::Error::new(ident.span(), "missing Option inner type"))?;
            if let Kind::Enum(path) = inner.as_ref() {
                let path = field.enum_path.as_ref().unwrap_or(path);
                Ok(quote! {
                    <::prost_canonical_serde::CanonicalEnumOption<#path> as __pcs_serde::Deserialize>::deserialize(
                        deserializer,
                    )?
                    .0
                })
            } else {
                Ok(quote! {
                    <::prost_canonical_serde::CanonicalOption<#inner_ty> as __pcs_serde::Deserialize>::deserialize(
                        deserializer,
                    )?
                    .0
                })
            }
        }
        Kind::Vec(inner) => {
            if let Kind::Enum(path) = inner.as_ref() {
                Ok(quote! {
                    <::prost_canonical_serde::CanonicalEnumVec<#path> as __pcs_serde::Deserialize>::deserialize(
                        deserializer,
                    )?
                    .0
                })
            } else {
                let inner_ty = field
                    .vec_inner
                    .as_ref()
                    .ok_or_else(|| syn::Error::new(ident.span(), "missing Vec inner type"))?;
                Ok(quote! {
                    <::prost_canonical_serde::CanonicalVec<#inner_ty> as __pcs_serde::Deserialize>::deserialize(
                        deserializer,
                    )?
                    .0
                })
            }
        }
        Kind::Map(_, _, value_kind) => {
            if let Kind::Enum(path) = value_kind.as_ref() {
                Ok(quote! {
                    <::prost_canonical_serde::CanonicalEnumMap<#path, #ty> as __pcs_serde::Deserialize>::deserialize(
                        deserializer,
                    )?
                    .0
                })
            } else {
                Ok(quote! {
                    <::prost_canonical_serde::CanonicalMap<#ty> as __pcs_serde::Deserialize>::deserialize(
                        deserializer,
                    )?
                    .0
                })
            }
        }
        Kind::Enum(path) => {
            let path = field.enum_path.as_ref().unwrap_or(path);
            Ok(quote! {
                <::prost_canonical_serde::CanonicalEnumValue<#path> as __pcs_serde::Deserialize>::deserialize(
                    deserializer,
                )?
                .0
            })
        }
        _ => Ok(quote! {
            <::prost_canonical_serde::CanonicalValue<#ty> as __pcs_serde::Deserialize>::deserialize(
                deserializer,
            )?
            .0
        }),
    }
}

fn serialized_field_count_stmt(field: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = &field.ident;

    if field.is_oneof || matches!(field.kind, Kind::Option(_)) {
        return quote! {
            if self.#ident.is_some() {
                __pcs_len += 1;
            }
        };
    }

    match &field.kind {
        Kind::Vec(_) | Kind::Map(_, _, _) => quote! {
            if !self.#ident.is_empty() {
                __pcs_len += 1;
            }
        },
        _ => {
            let field_expr = quote! { self.#ident };
            let default_check = default_check_expr(&field.kind, &field_expr);
            quote! {
                if #default_check {
                    __pcs_len += 1;
                }
            }
        }
    }
}

fn field_match_arm(field: &FieldInfo) -> syn::Result<proc_macro2::TokenStream> {
    if field.is_flatten {
        let target_ty = field.flatten_target_ty();
        return Ok(quote! {
            key if <#target_ty as ::prost_canonical_serde::ProstMessage>::matches_field_name(key) => true,
        });
    }

    if field.is_oneof {
        let oneof_ty = field
            .oneof_type
            .as_ref()
            .ok_or_else(|| syn::Error::new(field.ident.span(), "oneof field must be Option"))?;
        return Ok(quote! {
            key if <#oneof_ty as ::prost_canonical_serde::ProstOneof>::matches_field_name(key) => true,
        });
    }

    let ident = &field.ident;
    let deserialize_name = LitStr::new(&field.deserialize_name, ident.span());
    let proto_name = LitStr::new(&field.proto_name, ident.span());
    if field.deserialize_name == field.proto_name {
        Ok(quote! {
            #deserialize_name => true,
        })
    } else {
        Ok(quote! {
            #deserialize_name | #proto_name => true,
        })
    }
}

fn serialize_value_expr(
    kind: &Kind,
    ident: &Ident,
    enum_path: Option<&Path>,
) -> proc_macro2::TokenStream {
    if let Kind::Enum(path) = kind {
        let path = enum_path.unwrap_or(path);
        quote! {
            ::prost_canonical_serde::CanonicalEnum::<#path>::new(*#ident)
        }
    } else {
        quote! { ::prost_canonical_serde::Canonical::new(#ident) }
    }
}

fn map_new_expr(kind: &MapKind) -> proc_macro2::TokenStream {
    match kind {
        MapKind::Hash => quote! { ::std::collections::HashMap::new() },
        MapKind::BTree => quote! { ::alloc::collections::BTreeMap::new() },
    }
}

fn default_value_expr(kind: &Kind) -> proc_macro2::TokenStream {
    match kind {
        Kind::Scalar(ScalarKind::Bool) => quote! { false },
        Kind::Scalar(ScalarKind::I32 | ScalarKind::U32 | ScalarKind::I64 | ScalarKind::U64)
        | Kind::Enum(_) => quote! { 0 },
        Kind::Scalar(ScalarKind::F32 | ScalarKind::F64) => quote! { 0.0 },
        Kind::Scalar(ScalarKind::String) => quote! { ::alloc::string::String::new() },
        Kind::Bytes | Kind::Vec(_) => quote! { ::alloc::vec::Vec::new() },
        Kind::Map(map_kind, _, _) => map_new_expr(map_kind),
        Kind::Timestamp => quote! { ::prost_types::Timestamp::default() },
        Kind::Duration => quote! { ::prost_types::Duration::default() },
        Kind::Message => quote! { ::core::default::Default::default() },
        Kind::Option(_) => quote! { None },
    }
}

fn default_check_expr(kind: &Kind, field: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match kind {
        Kind::Scalar(ScalarKind::Bool) => quote! { #field },
        Kind::Scalar(ScalarKind::I32 | ScalarKind::U32 | ScalarKind::I64 | ScalarKind::U64)
        | Kind::Enum(_) => quote! { #field != 0 },
        Kind::Scalar(ScalarKind::F32 | ScalarKind::F64) => quote! { #field != 0.0 },
        Kind::Scalar(ScalarKind::String) | Kind::Bytes | Kind::Vec(_) | Kind::Map(_, _, _) => {
            quote! { !#field.is_empty() }
        }
        Kind::Timestamp | Kind::Duration | Kind::Message => quote! { true },
        Kind::Option(_) => quote! { #field.is_some() },
    }
}

fn is_prost_value_type(ty: &Type) -> bool {
    let Type::Path(path) = ty else { return false };
    let last = path.path.segments.last().map(|seg| seg.ident.to_string());
    if last.as_deref() != Some("Value") {
        return false;
    }
    path.path
        .segments
        .iter()
        .any(|seg| seg.ident == "prost_types")
}

fn extract_fields(
    fields: &Fields,
    container_attrs: &ContainerAttrs,
) -> syn::Result<Vec<FieldInfo>> {
    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| FieldInfo::from_field(field, container_attrs))
            .collect(),
        Fields::Unnamed(_) | Fields::Unit => Err(syn::Error::new(
            fields.span(),
            "CanonicalSerialize requires named fields",
        )),
    }
}

fn parse_variant(variant: &syn::Variant) -> syn::Result<(Type, Kind, Option<Path>)> {
    let fields = match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
        _ => {
            return Err(syn::Error::new(
                variant.span(),
                "oneof variants must be tuple variants with one field",
            ))
        }
    };

    let (is_oneof, enum_path) = parse_prost_attrs(&variant.attrs)?;
    if is_oneof {
        return Err(syn::Error::new(
            variant.span(),
            "unexpected oneof attribute on variant",
        ));
    }

    let mut kind = classify_type(&fields.ty)?;
    if let Some(enum_path) = enum_path.clone() {
        kind = apply_enum(kind, enum_path);
    }

    Ok((fields.ty.clone(), kind, enum_path))
}

fn parse_prost_attrs(attrs: &[Attribute]) -> syn::Result<(bool, Option<Path>)> {
    let mut is_oneof = false;
    let mut enum_path = None;

    for attr in attrs {
        if !attr.path().is_ident("prost") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("oneof") {
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let _ = value.parse::<syn::Lit>()?;
                }
                is_oneof = true;
                return Ok(());
            }
            if meta.path.is_ident("enumeration") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                let path = syn::parse_str::<Path>(&lit.value())?;
                enum_path = Some(path);
                return Ok(());
            }
            if meta.path.is_ident("btree_map")
                || meta.path.is_ident("map")
                || meta.path.is_ident("hash_map")
            {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                if let Some(path) = parse_enum_path_from_map(&lit.value())? {
                    enum_path = Some(path);
                }
                return Ok(());
            }
            if meta.input.peek(syn::Token![=]) {
                let value = meta.value()?;
                let _ = value.parse::<syn::Lit>()?;
            }
            Ok(())
        })?;
    }

    Ok((is_oneof, enum_path))
}

fn parse_enum_path_from_map(value: &str) -> syn::Result<Option<Path>> {
    let needle = "enumeration(";
    let start = match value.find(needle) {
        Some(index) => index + needle.len(),
        None => return Ok(None),
    };
    let end = value[start..]
        .find(')')
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "invalid map enum"))?;
    let path_str = value[start..start + end].trim();
    if path_str.is_empty() {
        return Ok(None);
    }
    let path = syn::parse_str::<Path>(path_str)?;
    Ok(Some(path))
}

fn is_oneof_enum(data: &syn::DataEnum) -> bool {
    data.variants.iter().any(|variant| {
        variant
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("prost"))
    })
}

fn classify_type(ty: &Type) -> syn::Result<Kind> {
    if let Some(inner) = extract_generic(ty, "Option", 0) {
        return Ok(Kind::Option(Box::new(classify_type(inner)?)));
    }

    if let Some(inner) = extract_generic(ty, "Vec", 0) {
        if is_u8(inner) {
            return Ok(Kind::Bytes);
        }
        return Ok(Kind::Vec(Box::new(classify_type(inner)?)));
    }

    if let Some((map_kind, key, value)) = extract_map_types(ty) {
        let key_kind = classify_key(key)?;
        let value_kind = classify_type(value)?;
        return Ok(Kind::Map(map_kind, key_kind, Box::new(value_kind)));
    }

    if is_bool(ty) {
        return Ok(Kind::Scalar(ScalarKind::Bool));
    }
    if is_i32(ty) {
        return Ok(Kind::Scalar(ScalarKind::I32));
    }
    if is_u32(ty) {
        return Ok(Kind::Scalar(ScalarKind::U32));
    }
    if is_i64(ty) {
        return Ok(Kind::Scalar(ScalarKind::I64));
    }
    if is_u64(ty) {
        return Ok(Kind::Scalar(ScalarKind::U64));
    }
    if is_f32(ty) {
        return Ok(Kind::Scalar(ScalarKind::F32));
    }
    if is_f64(ty) {
        return Ok(Kind::Scalar(ScalarKind::F64));
    }
    if is_string(ty) {
        return Ok(Kind::Scalar(ScalarKind::String));
    }
    if is_timestamp(ty) {
        return Ok(Kind::Timestamp);
    }
    if is_duration(ty) {
        return Ok(Kind::Duration);
    }

    Ok(Kind::Message)
}

fn classify_key(ty: &Type) -> syn::Result<KeyKind> {
    if is_string(ty) {
        return Ok(KeyKind::String);
    }
    if is_bool(ty) {
        return Ok(KeyKind::Bool);
    }
    if is_i32(ty) {
        return Ok(KeyKind::I32);
    }
    if is_i64(ty) {
        return Ok(KeyKind::I64);
    }
    if is_u32(ty) {
        return Ok(KeyKind::U32);
    }
    if is_u64(ty) {
        return Ok(KeyKind::U64);
    }

    Err(syn::Error::new(ty.span(), "unsupported map key type"))
}

fn apply_enum(kind: Kind, enum_path: Path) -> Kind {
    match kind {
        Kind::Scalar(ScalarKind::I32) => Kind::Enum(enum_path),
        Kind::Vec(inner) => match *inner {
            Kind::Scalar(ScalarKind::I32) => Kind::Vec(Box::new(Kind::Enum(enum_path))),
            other => Kind::Vec(Box::new(other)),
        },
        Kind::Option(inner) => match *inner {
            Kind::Scalar(ScalarKind::I32) => Kind::Option(Box::new(Kind::Enum(enum_path))),
            other => Kind::Option(Box::new(other)),
        },
        Kind::Map(map_kind, key_kind, value_kind) => match *value_kind {
            Kind::Scalar(ScalarKind::I32) => {
                Kind::Map(map_kind, key_kind, Box::new(Kind::Enum(enum_path)))
            }
            other => Kind::Map(map_kind, key_kind, Box::new(other)),
        },
        other => other,
    }
}

fn extract_generic<'a>(ty: &'a Type, name: &str, index: usize) -> Option<&'a Type> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let segment = path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let arg = args.args.iter().nth(index)?;
    if let syn::GenericArgument::Type(ty) = arg {
        Some(ty)
    } else {
        None
    }
}

fn extract_map_types(ty: &Type) -> Option<(MapKind, &Type, &Type)> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let segment = path.segments.last()?;
    let map_kind = if segment.ident == "HashMap" {
        MapKind::Hash
    } else if segment.ident == "BTreeMap" {
        MapKind::BTree
    } else {
        return None;
    };
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let mut iter = args.args.iter();
    let key = iter.next()?;
    let value = iter.next()?;
    match (key, value) {
        (syn::GenericArgument::Type(key), syn::GenericArgument::Type(value)) => {
            Some((map_kind, key, value))
        }
        _ => None,
    }
}

fn is_bool(ty: &Type) -> bool {
    path_ends_with_ident(ty, "bool")
}

fn is_i32(ty: &Type) -> bool {
    path_ends_with_ident(ty, "i32")
}

fn is_u32(ty: &Type) -> bool {
    path_ends_with_ident(ty, "u32")
}

fn is_i64(ty: &Type) -> bool {
    path_ends_with_ident(ty, "i64")
}

fn is_u64(ty: &Type) -> bool {
    path_ends_with_ident(ty, "u64")
}

fn is_f32(ty: &Type) -> bool {
    path_ends_with_ident(ty, "f32")
}

fn is_f64(ty: &Type) -> bool {
    path_ends_with_ident(ty, "f64")
}

fn is_u8(ty: &Type) -> bool {
    path_ends_with_ident(ty, "u8")
}

fn is_string(ty: &Type) -> bool {
    path_ends_with_ident(ty, "String")
}

fn is_timestamp(ty: &Type) -> bool {
    path_ends_with(ty, &["prost_types", "Timestamp"])
}

fn is_duration(ty: &Type) -> bool {
    path_ends_with(ty, &["prost_types", "Duration"])
}

fn path_ends_with_ident(ty: &Type, ident: &str) -> bool {
    let Type::Path(TypePath { path, .. }) = ty else {
        return false;
    };
    path.segments.last().is_some_and(|seg| seg.ident == ident)
}

fn path_ends_with(ty: &Type, idents: &[&str]) -> bool {
    let Type::Path(TypePath { path, .. }) = ty else {
        return false;
    };
    if path.segments.len() < idents.len() {
        return false;
    }
    let start = path.segments.len() - idents.len();
    path.segments
        .iter()
        .skip(start)
        .zip(idents)
        .all(|(seg, ident)| seg.ident == ident)
}

fn name_for_field(
    ident: &Ident,
    attrs: &CanonicalAttrs,
    rename_rule: Option<RenameRule>,
    proto_name: &str,
) -> String {
    if let Some(json_name) = &attrs.json_name {
        return json_name.clone();
    }
    if attrs.proto_name.is_some() {
        return RenameRule::CamelCase.apply_to_field(proto_name);
    }
    rename_rule
        .map(|rule| rule.apply_to_field(&ident.to_string()))
        .unwrap_or_else(|| RenameRule::CamelCase.apply_to_field(proto_name))
}

fn name_for_variant(
    ident: &Ident,
    attrs: &CanonicalAttrs,
    rename_rule: Option<RenameRule>,
    proto_name: &str,
) -> String {
    if let Some(json_name) = &attrs.json_name {
        return json_name.clone();
    }
    if attrs.proto_name.is_some() {
        return RenameRule::CamelCase.apply_to_variant(proto_name);
    }
    rename_rule
        .map(|rule| rule.apply_to_variant(&ident.to_string()))
        .unwrap_or_else(|| RenameRule::CamelCase.apply_to_variant(proto_name))
}

fn split_words(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut words = Vec::new();
    let mut current = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        if matches!(ch, '_' | '-') {
            if !current.is_empty() {
                words.push(current);
                current = String::new();
            }
            continue;
        }

        let boundary = if current.is_empty() {
            false
        } else {
            let prev = chars[index - 1];
            let next = chars.get(index + 1).copied();
            (prev.is_ascii_lowercase() && ch.is_ascii_uppercase())
                || (prev.is_ascii_alphabetic() && ch.is_ascii_digit())
                || (prev.is_ascii_digit() && ch.is_ascii_alphabetic())
                || (prev.is_ascii_uppercase()
                    && ch.is_ascii_uppercase()
                    && next.is_some_and(|next| next.is_ascii_lowercase()))
        };

        if boundary {
            words.push(current);
            current = String::new();
        }

        current.push(ch.to_ascii_lowercase());
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut result = String::new();
    result.push(first.to_ascii_uppercase());
    result.push_str(chars.as_str());
    result
}

#[derive(Clone, Copy)]
enum RenameRule {
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    fn parse(value: &LitStr) -> syn::Result<Self> {
        match value.value().as_str() {
            "lowercase" => Ok(Self::LowerCase),
            "UPPERCASE" => Ok(Self::UpperCase),
            "PascalCase" => Ok(Self::PascalCase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebabCase),
            _ => Err(syn::Error::new(
                value.span(),
                "unsupported rename_all case convention",
            )),
        }
    }

    fn apply_to_field(self, name: &str) -> String {
        self.apply(name)
    }

    fn apply_to_variant(self, name: &str) -> String {
        self.apply(name)
    }

    fn apply(self, name: &str) -> String {
        let words = split_words(name);
        match self {
            Self::LowerCase => words.concat(),
            Self::UpperCase => words.concat().to_ascii_uppercase(),
            Self::PascalCase => words
                .iter()
                .map(|word| capitalize(word))
                .collect::<Vec<_>>()
                .join(""),
            Self::CamelCase => {
                let mut iter = words.iter();
                let Some(first) = iter.next() else {
                    return String::new();
                };
                let mut result = first.clone();
                for word in iter {
                    result.push_str(&capitalize(word));
                }
                result
            }
            Self::SnakeCase => words.join("_"),
            Self::ScreamingSnakeCase => words
                .iter()
                .map(|word| word.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
            Self::KebabCase => words.join("-"),
            Self::ScreamingKebabCase => words
                .iter()
                .map(|word| word.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join("-"),
        }
    }
}

#[derive(Clone)]
struct FieldInfo {
    ident: Ident,
    ty: Type,
    kind: Kind,
    enum_path: Option<Path>,
    is_oneof: bool,
    is_flatten: bool,
    serialize_name: String,
    deserialize_name: String,
    proto_name: String,
    oneof_type: Option<Type>,
    option_inner: Option<Type>,
    vec_inner: Option<Type>,
}

impl FieldInfo {
    fn from_field(field: &syn::Field, container_attrs: &ContainerAttrs) -> syn::Result<Self> {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(field.span(), "expected named field"))?;
        let (is_oneof, enum_path) = parse_prost_attrs(&field.attrs)?;
        let attrs = parse_canonical_attrs(&field.attrs)?;
        if attrs.transparent {
            return Err(syn::Error::new(
                field.span(),
                "transparent is only supported on containers",
            ));
        }
        let mut kind = classify_type(&field.ty)?;
        let mut oneof_type = None;
        let option_inner = extract_generic(&field.ty, "Option", 0).cloned();
        let vec_inner = extract_generic(&field.ty, "Vec", 0).cloned();

        if let Some(enum_path) = enum_path.clone() {
            kind = apply_enum(kind, enum_path);
        }

        if is_oneof {
            if let Some(inner) = extract_generic(&field.ty, "Option", 0) {
                oneof_type = Some(inner.clone());
                kind = Kind::Option(Box::new(Kind::Message));
            }
        }

        if attrs.flatten {
            if is_oneof {
                return Err(syn::Error::new(
                    field.span(),
                    "flatten is not supported on oneof fields",
                ));
            }
            if attrs.proto_name.is_some() || attrs.json_name.is_some() {
                return Err(syn::Error::new(
                    field.span(),
                    "flatten fields cannot set proto_name or json_name",
                ));
            }
            if !is_message_like(&kind) {
                return Err(syn::Error::new(
                    field.span(),
                    "flatten is only supported on message fields",
                ));
            }
        }

        let proto_name = attrs
            .proto_name
            .clone()
            .unwrap_or_else(|| ident.to_string());
        let serialize_name = name_for_field(
            &ident,
            &attrs,
            container_attrs.serialize_rename_all,
            &proto_name,
        );
        let deserialize_name = name_for_field(
            &ident,
            &attrs,
            container_attrs.deserialize_rename_all,
            &proto_name,
        );

        Ok(Self {
            ident,
            ty: field.ty.clone(),
            kind,
            enum_path,
            is_oneof,
            is_flatten: attrs.flatten,
            serialize_name,
            deserialize_name,
            proto_name,
            oneof_type,
            option_inner,
            vec_inner,
        })
    }

    fn flatten_target_ty(&self) -> &Type {
        if self.is_option_message() {
            self.option_inner
                .as_ref()
                .expect("flatten option fields must have an inner type")
        } else {
            &self.ty
        }
    }

    fn is_option_message(&self) -> bool {
        matches!(&self.kind, Kind::Option(inner) if matches!(inner.as_ref(), Kind::Message))
    }

    fn flatten_buffer_ident(&self) -> syn::Result<Ident> {
        let ident = format!("__pcs_flatten_{}", self.ident);
        Ok(Ident::new(&ident, self.ident.span()))
    }
}

fn parse_canonical_attrs(attrs: &[Attribute]) -> syn::Result<CanonicalAttrs> {
    let mut parsed = CanonicalAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("prost_canonical_serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("proto_name") {
                let value: LitStr = meta.value()?.parse()?;
                parsed.proto_name = Some(value.value());
            } else if meta.path.is_ident("json_name") {
                let value: LitStr = meta.value()?.parse()?;
                parsed.json_name = Some(value.value());
            } else if meta.path.is_ident("transparent") {
                parsed.transparent = true;
            } else if meta.path.is_ident("flatten") {
                parsed.flatten = true;
            }
            Ok(())
        })?;
    }

    Ok(parsed)
}

fn parse_container_attrs(input: &DeriveInput) -> syn::Result<ContainerAttrs> {
    let attrs = parse_canonical_attrs(&input.attrs)?;
    let default_name = input.ident.to_string();
    let mut container = ContainerAttrs {
        transparent: attrs.transparent,
        serialize_name: default_name.clone(),
        deserialize_name: default_name,
        serialize_rename_all: None,
        deserialize_rename_all: None,
        serde_path: syn::parse_str("::serde")?,
        serialize_via: SerializeVia::None,
        deserialize_via: DeserializeVia::None,
        deny_unknown_fields: false,
        tag: None,
        default: ContainerDefault::None,
    };

    for attr in &input.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                if meta.input.peek(Token![=]) {
                    let value: LitStr = meta.value()?.parse()?;
                    let name = value.value();
                    container.serialize_name = name.clone();
                    container.deserialize_name = name;
                } else {
                    meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("serialize") {
                            let value: LitStr = nested.value()?.parse()?;
                            container.serialize_name = value.value();
                        } else if nested.path.is_ident("deserialize") {
                            let value: LitStr = nested.value()?.parse()?;
                            container.deserialize_name = value.value();
                        }
                        Ok(())
                    })?;
                }
            } else if meta.path.is_ident("rename_all") {
                if meta.input.peek(Token![=]) {
                    let value: LitStr = meta.value()?.parse()?;
                    let rule = RenameRule::parse(&value)?;
                    container.serialize_rename_all = Some(rule);
                    container.deserialize_rename_all = Some(rule);
                } else {
                    meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("serialize") {
                            let value: LitStr = nested.value()?.parse()?;
                            container.serialize_rename_all = Some(RenameRule::parse(&value)?);
                        } else if nested.path.is_ident("deserialize") {
                            let value: LitStr = nested.value()?.parse()?;
                            container.deserialize_rename_all = Some(RenameRule::parse(&value)?);
                        }
                        Ok(())
                    })?;
                }
            } else if meta.path.is_ident("deny_unknown_fields") {
                container.deny_unknown_fields = true;
            } else if meta.path.is_ident("tag") {
                let value: LitStr = meta.value()?.parse()?;
                container.tag = Some(value.value());
            } else if meta.path.is_ident("crate") {
                let value: LitStr = meta.value()?.parse()?;
                container.serde_path = syn::parse_str::<Path>(&value.value())?;
            } else if meta.path.is_ident("into") {
                let value: LitStr = meta.value()?.parse()?;
                let into_ty = syn::parse_str::<Type>(&value.value())?;
                set_serialize_via(
                    &mut container.serialize_via,
                    SerializeVia::Into(into_ty),
                    meta.path.span(),
                )?;
            } else if meta.path.is_ident("default") {
                if meta.input.peek(Token![=]) {
                    let value: LitStr = meta.value()?.parse()?;
                    let path = syn::parse_str::<Path>(&value.value())?;
                    container.default = ContainerDefault::Path(path);
                } else {
                    container.default = ContainerDefault::Default;
                }
            } else if meta.path.is_ident("from") {
                let value: LitStr = meta.value()?.parse()?;
                let from_ty = syn::parse_str::<Type>(&value.value())?;
                set_deserialize_via(
                    &mut container.deserialize_via,
                    DeserializeVia::From(from_ty),
                    meta.path.span(),
                )?;
            } else if meta.path.is_ident("try_from") {
                let value: LitStr = meta.value()?.parse()?;
                let from_ty = syn::parse_str::<Type>(&value.value())?;
                set_deserialize_via(
                    &mut container.deserialize_via,
                    DeserializeVia::TryFrom(from_ty),
                    meta.path.span(),
                )?;
            } else {
                return Err(unsupported_serde_container_attr(&meta.path));
            }
            Ok(())
        })?;
    }

    Ok(container)
}

fn unsupported_serde_container_attr(path: &syn::Path) -> syn::Error {
    let attr = path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_else(|| "attribute".to_string());
    syn::Error::new(
        path.span(),
        format!("unsupported serde container attribute `{attr}`"),
    )
}

fn set_serialize_via(
    slot: &mut SerializeVia,
    value: SerializeVia,
    span: Span,
) -> syn::Result<()> {
    if !matches!(slot, SerializeVia::None) {
        return Err(syn::Error::new(
            span,
            "only one serde into attribute may be specified",
        ));
    }
    *slot = value;
    Ok(())
}

fn set_deserialize_via(
    slot: &mut DeserializeVia,
    value: DeserializeVia,
    span: Span,
) -> syn::Result<()> {
    if !matches!(slot, DeserializeVia::None) {
        return Err(syn::Error::new(
            span,
            "only one of serde from and try_from may be specified",
        ));
    }
    *slot = value;
    Ok(())
}

#[derive(Default)]
struct CanonicalAttrs {
    proto_name: Option<String>,
    json_name: Option<String>,
    transparent: bool,
    flatten: bool,
}

struct ContainerAttrs {
    transparent: bool,
    serialize_name: String,
    deserialize_name: String,
    serialize_rename_all: Option<RenameRule>,
    deserialize_rename_all: Option<RenameRule>,
    serde_path: Path,
    serialize_via: SerializeVia,
    deserialize_via: DeserializeVia,
    deny_unknown_fields: bool,
    tag: Option<String>,
    default: ContainerDefault,
}

enum ContainerDefault {
    None,
    Default,
    Path(Path),
}

enum SerializeVia {
    None,
    Into(Type),
}

enum DeserializeVia {
    None,
    From(Type),
    TryFrom(Type),
}

fn is_message_like(kind: &Kind) -> bool {
    match kind {
        Kind::Message => true,
        Kind::Option(inner) => matches!(inner.as_ref(), Kind::Message),
        _ => false,
    }
}

#[derive(Clone)]
enum Kind {
    Scalar(ScalarKind),
    Bytes,
    Vec(Box<Kind>),
    Map(MapKind, KeyKind, Box<Kind>),
    Option(Box<Kind>),
    Enum(Path),
    Timestamp,
    Duration,
    Message,
}

#[derive(Clone)]
enum ScalarKind {
    Bool,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    String,
}

#[derive(Clone)]
enum KeyKind {
    String,
    Bool,
    I32,
    I64,
    U32,
    U64,
}

#[derive(Clone)]
enum MapKind {
    Hash,
    BTree,
}

