#![allow(unused)]

use std::{collections::HashMap, f64::consts::E};

use proc_macro_error::abort;
use quote::ToTokens;
use syn::{punctuated::Punctuated, spanned::Spanned, Error, LitBool, LitStr, Meta, Token};

use crate::common::{ColumnAttribute, TableAttribute, TableColumn, TableId};

/// Find the types (as string) for each field of the struct
pub(crate) fn get_fields_type(fields: &[syn::Field]) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field.ident.clone().unwrap().to_string();
        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref lifetime,
                ref elem,
                ..
            }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            _ => {
                let mut field_type = proc_macro2::TokenStream::new();
                field.ty.to_tokens(&mut field_type);
                abort!(
                    field.ty.span(),
                    "Type `{}` of field `{}` not supported",
                    field_type,
                    field_ident
                )
            }
        };
        types.insert(field_ident, field_type);
    }

    types
}

///  `#[name]` attribute name ignore or not
pub(crate) fn has_attribute(attr: &syn::Attribute, name: &str) -> bool {
    attr.path().is_ident(name)
}

pub fn to_snake_name(name: &str) -> String {
    let chs = name.chars();
    let mut new_name = String::new();
    let mut index = 0;
    let chs_len = name.len();
    for x in chs {
        if x.is_uppercase() {
            if index != 0 && (index + 1) != chs_len {
                new_name.push_str("_");
            }
            new_name.push_str(x.to_lowercase().to_string().as_str());
        } else {
            new_name.push(x);
        }
        index += 1;
    }
    return new_name;
}

pub fn string_to_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn to_upper_camel_case(input: &str) -> String {
    let mut camel_case = String::new();
    let mut should_capitalize = true;

    for c in input.chars() {
        if c == '_' {
            should_capitalize = true;
        } else {
            if should_capitalize {
                camel_case.push(c.to_ascii_uppercase());
                should_capitalize = false;
            } else {
                camel_case.push(c);
            }
        }
    }

    camel_case
}

pub(crate) fn get_attributes(
    attrs: &[syn::Attribute],
    attribute: &TableAttribute,
) -> Vec<ColumnAttribute> {
    attrs.iter()
    .filter(|attr| attr.path().is_ident(&attribute.name()))
    .map(|attr| {
        let mut map = HashMap::new();
        match &attr.meta {
                // #[name(key="val")]
                Meta::List(list) => {
                    let nested = attr
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .unwrap();

                    for meta in nested {
                        match &meta {
                            // #[name = "value"]
                            Meta::NameValue(syn::MetaNameValue {
                                path,
                                eq_token,
                                value,
                            }) => {
                                let key = path.get_ident().unwrap().to_string();
                                if !attribute.has_attr(&key) {
                                    abort!(
                                        &meta.span(),
                                        format!("unexpected name value annotion: {:?}", key)
                                    )
                                }

                                let v = expr_to_string(&value).unwrap_or_default();

                                let (check, msg) = attribute.check_value(&key, &v);
                                if !check {
                                    abort!(&meta.span(), format!("invalid argument type for {:?} annotion: expected {:?}", key, msg))
                                }

                                map.insert(key, v);
                            }
                            _ => abort!(list.span(), "Unsupported attribute"),
                        }
                    }
                }
                _ => abort!(attr.span(), "Unsupported attribute"),
            }
        ColumnAttribute::from_attribute(attribute, map)
    }).collect::<Vec<_>>()
}

pub fn expr_to_string(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Some(s.value()),
            _ => None,
        },
        _ => None,
    }
}

macro_rules! get_attributes_value {
    ( $attr:expr, $($element:expr),* ) => {
        {
            let mut map = HashMap::new();
            if let Err(err) = $attr.parse_nested_meta(|meta| {
                let ident = meta.path.get_ident().unwrap();
                let value = meta.value()?;
                $(
                    if meta.path.is_ident($element) {
                        let v: LitStr = value.parse()?;
                        map.insert($element.to_string(), v.value());
                        return Ok(());
                    }
                )*

                Err(meta.error(format!(
                    "unexpected name value annotion: {:?}",
                    ident.to_string()
                )))
            }) {
                abort!(err.span(), err);
            }

            map
        }

    };
}

macro_rules! get_attributes1 {
    ( $attrs:expr, $attr_type:expr ) => {{
        let name = $attr_type.name();
        let attr_names = $attr_type.attr_names();
        let attr_values = $attr_type.attrs();

        let attributes = $attrs
            .iter()
            .filter(|attr| attr.path().is_ident(name))
            .map(|attr| {
                let mut map = HashMap::new();
                if let Err(err) = attr.parse_nested_meta(|meta| {
                    let ident = meta.path.get_ident().unwrap();
                    let value = meta.value()?;
                    // $(
                    //     if meta.path.is_ident($element) {
                    //         let v: LitStr = value.parse()?;
                    //         map.insert($element.to_string(), v.value());
                    //         return Ok(());
                    //     }
                    // )*

                    // get_attributes_value_str!( $($attr_type.attrs()),* );

                    Err(meta.error(format!(
                        "unexpected name value annotion: {:?}",
                        ident.to_string()
                    )))
                }) {
                    abort!(err.span(), err);
                }

                map
            })
            .collect::<Vec<HashMap<String, String>>>();
        match attributes.last() {
            Some(res) => res.clone(),
            None => HashMap::new(),
        }
    }};
}
