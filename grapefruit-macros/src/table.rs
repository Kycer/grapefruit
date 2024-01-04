use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::abort;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

use crate::{
    common::{ColumnAttribute, ColumnInformation, TableAttribute},
    util::{get_attributes, get_fields_type, to_snake_name, to_upper_camel_case},
};

pub fn impl_table(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let res = parse_table(&derive_input);
    res.into()
}

fn parse_table(ast: &syn::DeriveInput) -> TokenStream {
    // let generics = &ast.generics;
    let struct_name = &ast.ident;

    let (table_name, _curd) = get_table_info(ast);
    let fields = get_field_info(ast);

    let column_info = get_column_info(&fields);

    let build_generate_enum = build_generate_enum(struct_name, &column_info, &fields);

    let build_impl_try_get = build_impl_try_get(struct_name, &fields);

    let build_impl_entity =
        build_impl_entity(struct_name, table_name.clone(), &column_info, &fields);

    // let build_table_crud = build_table_crud(struct_name, &fields, curd);

    quote!(

        #build_generate_enum

        #build_impl_try_get

        #build_impl_entity

        // #build_table_crud

    )
    .into()
}

// fn build_table_crud(
//     struct_name: &Ident,
//     fields: &[ColumnInformation],
//     curd: bool,
// ) -> proc_macro2::TokenStream {
//     if !curd {
//         return quote!();
//     }

//     quote!(
//         // impl #struct_name {
//         //     pub async fn insert(&self) -> grapefruit::GrapefruitResult<#struct_name> {
//         //         todo!()
//         //     }
//         // }
//     )
// }

fn get_table_info(ast: &syn::DeriveInput) -> (String, bool) {
    let tables = get_attributes(&ast.attrs, &TableAttribute::Table);
    if tables.len() > 1 {
        abort!(ast.span(), "Only one table is allowed");
    }

    let mut table_name = None;
    let mut curd = false;

    if let Some(table_attr) = tables.iter().next() {
        if let Some(ref table) = table_attr.get_table() {
            table_name = table.name.clone();
            curd = table.curd.clone();
        }
    }
    if table_name.is_none() {
        table_name = Some(to_snake_name(&ast.ident.to_string()))
    }
    (table_name.unwrap(), curd)
}

/// generate enum
fn build_generate_enum(
    struct_name: &Ident,
    column_info: &HashMap<String, proc_macro2::TokenStream>,
    fields: &[ColumnInformation],
) -> proc_macro2::TokenStream {
    let enum_name = format!("{}Def", struct_name.to_string());
    let enum_ident = Ident::new(&enum_name, struct_name.span());

    let mut enum_columns = Vec::new();
    let mut enum_infos = Vec::new();
    fields.iter().for_each(|f| {
        let enum_column = Ident::new(&to_upper_camel_case(&f.name.clone()), f.name.span());
        let value = column_info.get(&f.name.clone()).unwrap();
        enum_columns.push(enum_column.clone());
        enum_infos.push(quote!(#enum_ident::#enum_column => Some(#value)));
    });

    quote!(
        #[derive(Debug)]
        pub enum #enum_ident {
            #(#enum_columns),*
        }

        impl grapefruit::Column for #enum_ident {
            fn column_info(&self) -> Option<grapefruit::ColumnInfo> {
                match self {
                    #(
                        #enum_infos,
                    )*
                }
            }
        }
    )
}

/// build try get
fn build_impl_try_get(
    struct_name: &Ident,
    fields: &[ColumnInformation],
) -> proc_macro2::TokenStream {
    let mut entity_try_get = Vec::new();

    fields.iter().for_each(|f| {
        let name = f.name.clone();
        let alias = f.alias.clone();
        let column_ident = Ident::new(&name.clone(), f.name.span());
        let is_optional = f.field_type.starts_with("Option<");
        if is_optional {
            entity_try_get.push(quote!(
                if let Ok(#column_ident) =  row.try_get(#alias) {
                    if let Some(value) = #column_ident {
                        entity.#column_ident = value;
                    }
                }
            ));
        } else {
            entity_try_get.push(quote!(
                if let Ok(#column_ident) =  row.try_get(#alias) {
                    entity.#column_ident = #column_ident;
                }

            ));
        }
    });

    quote!(
        use sqlx::Row;
        impl grapefruit::TryGetable for #struct_name {
            fn try_get(res: &grapefruit::QueryResult) -> grapefruit::GrapefruitResult<Option<Self>> {
                let entity = match res.row.as_ref() {
                    Some(row) => {
                        let mut entity = #struct_name::default();
                        match row {
                            #[cfg(feature = "mysql")]
                            grapefruit::QueryRow::MySql(row) => {
                                #(#entity_try_get)*
                            }
                            #[cfg(feature = "postgres")]
                            grapefruit::QueryRow::Postgres(row) => {
                                #(#entity_try_get)*
                            }
                            #[cfg(feature = "sqlite")]
                            grapefruit::QueryRow::Sqlite(row) => {
                                #(#entity_try_get)*
                            }
                        }
                        Some(entity)
                    },
                    None => None,
                };

                Ok(entity)
            }
        }
    )
}

/// build impl entity
fn build_impl_entity(
    struct_name: &Ident,
    table_name: String,
    column_info: &HashMap<String, proc_macro2::TokenStream>,
    fields: &[ColumnInformation],
) -> proc_macro2::TokenStream {
    let mut table_id_info = quote!(None);
    let mut logic_delete_info = quote!(None);
    let mut version_info = quote!(None);
    let mut column_map_info = Vec::new();
    let mut to_value = Vec::new();
    let mut insert_column_info = Vec::new();
    let mut update_column_info = Vec::new();
    let mut select_column_info = Vec::new();

    fields.iter().for_each(|f| {
        let name = f.name.clone();
        let column_ident = Ident::new(&name.clone(), f.name.span());
        let value = column_info.get(&name).unwrap();
        if f.is_table_id() {
            table_id_info = quote!(Some(#value));
            let table_id = &f.get_table_id();
            if !table_id.id_type.eq("auto") {
                insert_column_info.push(name.clone());
            }
            select_column_info.push(name.clone());
        } else {
            let table_column = &f.get_table_column();

            if !table_column.ignore {
                if !table_column.insert_strateg.eq("never") {
                    insert_column_info.push(name.clone());
                }
                if !table_column.update_strateg.eq("never") && !f.is_logic_delete() {
                    update_column_info.push(name.clone());
                }
                if table_column.select {
                    select_column_info.push(name.clone());
                }
            }

            if f.is_logic_delete() {
                logic_delete_info = quote!(Some(#value));
            }

            if f.is_version() {
                version_info = quote!(Some(#value));
            }
        }

        column_map_info.push(quote!(map.insert(#name.to_string(), #value);));

        to_value.push(quote!(map.insert(#name.to_string(), self.#column_ident.clone().into());));
    });

    quote!(
        use grapefruit::Column;
        impl grapefruit::Entity for #struct_name {

            fn table_info() -> grapefruit::TableInfo {
                grapefruit::TableInfo {
                    table_name: #table_name.into(),
                }
            }

            fn columns() -> std::collections::HashMap<String, grapefruit::ColumnInfo> {
                let mut map = std::collections::HashMap::new();
                #(#column_map_info)*
                map
            }

            fn to_value(&self) -> std::collections::HashMap<String, grapefruit::Value> {
                let mut map = std::collections::HashMap::new();
                #(#to_value)*
                map
            }

            fn primary_key() -> Option<grapefruit::ColumnInfo> {
                #table_id_info
            }

            fn logic_delete() -> Option<grapefruit::ColumnInfo> {
                #logic_delete_info
            }

            fn version() -> Option<grapefruit::ColumnInfo> {
                #version_info
            }

            fn insert_columns() -> Vec<String> {
                vec![
                    #(#insert_column_info.to_string()),*
                ]
            }

            fn update_columns() -> Vec<String>{
                vec![
                    #(#update_column_info.to_string()),*
                ]
            }

            fn select_columns() -> Vec<String>{
                vec![
                    #(#select_column_info.to_string()),*
                ]
            }
        }
    )
}

fn get_column_info(fields: &[ColumnInformation]) -> HashMap<String, proc_macro2::TokenStream> {
    let mut map = HashMap::new();
    fields.iter().for_each(|f| {
        let name = f.name.clone();
        let alias = f.alias.clone().unwrap_or(name.clone());
        let attribute = f.attribute.clone();

        let field_type = f.field_type.clone();
        let mut ignore = false;
        let mut insert_strateg = quote!(grapefruit::ColumnStrategy::Default);
        let mut update_strateg = quote!(grapefruit::ColumnStrategy::Default);
        let mut column_type = quote!(grapefruit::ColumnType::TableColumn(#field_type.to_string()));
        let mut fill = quote!(grapefruit::Fill::Default);
        let mut logic_delete = false;
        let mut version = false;

        match attribute {
            ColumnAttribute::TableId(table_id) => {
                let id_type = table_id.id_type;
                insert_strateg = quote!(grapefruit::ColumnStrategy::Default);
                update_strateg = quote!(grapefruit::ColumnStrategy::Default);
                column_type =
                    quote!(grapefruit::ColumnType::TableId(grapefruit::IdType::from_str(#id_type)));
                fill = quote!(grapefruit::Fill::Insert);
            }
            ColumnAttribute::TableColumn(table_field) => {
                ignore = table_field.ignore;
                let insert_strateg_str = table_field.insert_strateg;
                let update_strateg_str = table_field.update_strateg;
                let fill_str = table_field.fill;
                insert_strateg = quote!(grapefruit::ColumnStrategy::from_str(#insert_strateg_str));
                update_strateg = quote!(grapefruit::ColumnStrategy::from_str(#update_strateg_str));
                fill = quote!(grapefruit::Fill::from_str(#fill_str));
                logic_delete = table_field.is_logic_delete;
                version = table_field.version;
            }
            _ => {}
        }

        let value = quote!(
            grapefruit::ColumnInfo {
                name: #name.into(),
                alias:#alias.into(),
                ignore: #ignore,
                insert_strateg: #insert_strateg,
                update_strateg: #update_strateg,
                column_type: #column_type,
                fill: #fill,
                is_logic_delete: #logic_delete,
                version: #version,
            },
        );
        map.insert(name, value);
    });
    map
}

fn get_field_info(ast: &syn::DeriveInput) -> Vec<ColumnInformation> {
    let mut fields = collect_fields(ast);
    let field_types = get_fields_type(&fields);

    let column_attrs = fields.drain(..).fold(vec![], |mut acc, field| {
        let key = field.ident.clone().unwrap().to_string();

        let field_name = field.ident.clone().unwrap().to_string();

        let table_ids = get_attributes(&field.attrs, &TableAttribute::TableId);
        let table_columns = get_attributes(&field.attrs, &TableAttribute::TableColumn);

        if !table_ids.is_empty() && !table_columns.is_empty() {
            abort!(
                field.span(),
                "Cannot use both `#[id]` and `#[column]` on the same field"
            );
        }

        if table_ids.is_empty() && table_columns.is_empty() {
            abort!(
                field.span(),
                "Must use either `#[id]` or `#[column]` on the same field"
            );
        }

        if table_ids.len() > 1 || table_columns.len() > 1 {
            abort!(
                field.span(),
                "Cannot use multiple `#[id]` or `#[column]` on the same field"
            );
        }

        let field_type = field_types.get(&key).unwrap().clone();
        if !table_ids.is_empty() {
            if !field_type.starts_with("Option<") && !field_type.ends_with(">") {
                abort!(field.span(), "Must use `Option<>` for primary_key");
            }
        }

        let attribute = if !table_ids.is_empty() {
            table_ids.get(0).unwrap().clone()
        } else {
            table_columns.get(0).unwrap().clone()
        };

        acc.push(ColumnInformation::new(
            field, field_type, field_name, &attribute,
        ));
        acc
    });

    if !column_attrs.iter().any(|c| c.is_table_id()) {
        abort!(
            ast.span(),
            "Cannot use `#[id]` on a struct with other `#[column]` fields"
        );
    }

    let columns = column_attrs
        .iter()
        .filter(|c| c.is_table_column())
        .map(|c| c.table_column.clone())
        .collect::<Vec<_>>();

    if columns
        .iter()
        .filter(|c| c.is_some())
        .map(|c| {
            let tc = c.as_ref().unwrap();
            tc.is_logic_delete.clone()
        })
        .filter(|logic| *logic)
        .count()
        > 1
    {
        abort!(
            ast.span(),
            "Cannot use multiple `#[column]` fields with `#[is_logic_delete]`"
        );
    }

    if columns
        .iter()
        .filter(|c| c.is_some())
        .map(|c| {
            let tc = c.as_ref().unwrap();
            tc.version.clone()
        })
        .filter(|logic| *logic)
        .count()
        > 1
    {
        abort!(
            ast.span(),
            "Cannot use multiple `#[column]` fields with `#[version]`"
        );
    }

    column_attrs
}

/// collect the ast fields
fn collect_fields(ast: &syn::DeriveInput) -> Vec<syn::Field> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "struct has unnamed fields";
                    help = "#[derive(GrapefruitTable)] can only be used on structs with named fields";
                );
            }
            fields.iter().cloned().collect::<Vec<_>>()
        }
        _ => abort!(
            ast.span(),
            "#[derive(GrapefruitTable)] can only be used with structs"
        ),
    }
}
