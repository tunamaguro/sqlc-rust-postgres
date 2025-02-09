use std::num::NonZeroUsize;

use crate::plugin;
use crate::user_type::{col_type, TypeMap};
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

pub(crate) trait GenericQuery {
    // query ident name
    fn ident_str(&self) -> String;
    fn ident(&self) -> Ident {
        Ident::new(&self.ident_str(), Span::call_site())
    }
    // sql query
    fn sql_str(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PostgresQuery {
    name: String,
    query: String,
}

impl GenericQuery for PostgresQuery {
    fn ident_str(&self) -> String {
        self.name.to_case(Case::UpperSnake)
    }

    fn sql_str(&self) -> String {
        self.query.clone()
    }
}

impl PostgresQuery {
    pub(crate) fn new(query: &plugin::Query) -> Self {
        Self {
            name: query.name.clone(),
            query: query.text.clone(),
        }
    }
}

impl ToTokens for PostgresQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident();
        let raw_str = format!("r#\"{}\"#", self.sql_str());
        let raw_literal: proc_macro2::TokenStream =
            raw_str.parse().expect("Failed to parse raw literal");
        tokens.extend(quote! {
            pub const #ident: &str = #raw_literal;
        });
    }
}

fn column_name(column: &plugin::Column) -> String {
    let name = if let Some(table) = &column.table {
        format!("{}_{}", table.name, column.name)
    } else {
        column.name.clone()
    };
    name.to_case(Case::Snake)
}

#[derive(Debug, Clone)]
struct PgColumn {
    name: String,
    rs_type: proc_macro2::TokenStream,
    /// None => not array
    array_dim: Option<NonZeroUsize>,
    is_nullable: bool,
}

impl PgColumn {
    pub(crate) fn new(column: &plugin::Column, pg_map: &impl TypeMap) -> Self {
        let pg_type = column.r#type.as_ref();

        let rs_type = pg_map
            .get(&col_type(pg_type))
            .expect("Column type not found")
            .clone();

        let array_dim = NonZeroUsize::new(column.array_dims.try_into().unwrap_or(0));
        let is_nullable = !column.not_null;
        let name = column_name(column);

        Self {
            name,
            rs_type,
            array_dim,
            is_nullable,
        }
    }
}

impl ToTokens for PgColumn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_ident = Ident::new(&self.name, Span::call_site());
        let rs_type = &self.rs_type;
        let mut ty_tokens = quote! { #rs_type };

        if let Some(dim) = self.array_dim {
            for _ in 0..dim.get() {
                ty_tokens = quote! { Vec<#ty_tokens> };
            }
        }

        if self.is_nullable {
            ty_tokens = quote! { Option<#ty_tokens> };
        }

        tokens.extend(quote! {
            #field_ident: #ty_tokens
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PgStruct {
    name: String,
    columns: Vec<PgColumn>,
}

impl PgStruct {
    pub(crate) fn generate_row(query: &plugin::Query, pg_map: &impl TypeMap) -> Self {
        let columns = query
            .columns
            .iter()
            .map(|c| PgColumn::new(c, pg_map))
            .collect::<Vec<_>>();

        let name = query.name.to_case(Case::Pascal);
        let name = format!("{}Row", name);
        Self { name, columns }
    }

    pub(crate) fn generate_param(query: &plugin::Query, pg_map: &impl TypeMap) -> Self {
        todo!()
    }
}

impl ToTokens for PgStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = Ident::new(&self.name, Span::call_site());
        let columns = &self.columns;
        tokens.extend(quote! {
            pub struct #ident {
                #(#columns),*
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_col_name() {
        {
            let col = plugin::Column {
                name: "name".to_owned(),
                not_null: true,
                is_array: false,
                comment: "".to_owned(),
                length: -1,
                is_named_param: false,
                is_func_call: false,
                scope: "".to_owned(),
                table: Some(plugin::Identifier {
                    catalog: "".to_owned(),
                    schema: "".to_owned(),
                    name: "author".to_owned(),
                }),
                table_alias: "".to_owned(),
                r#type: Some(plugin::Identifier {
                    catalog: "".to_owned(),
                    schema: "pg_catalog".to_owned(),
                    name: "varchar".to_owned(),
                }),
                is_sqlc_slice: false,
                embed_table: None,
                original_name: "name".to_owned(),
                unsigned: false,
                array_dims: 0,
            };

            assert_eq!(column_name(&col), "author_name")
        }

        {
            let col = plugin::Column {
                name: "AsColumnName".to_owned(),
                not_null: true,
                is_array: false,
                comment: "".to_owned(),
                length: -1,
                is_named_param: false,
                is_func_call: false,
                scope: "".to_owned(),
                table: None,
                table_alias: "".to_owned(),
                r#type: Some(plugin::Identifier {
                    catalog: "".to_owned(),
                    schema: "pg_catalog".to_owned(),
                    name: "int4".to_owned(),
                }),
                is_sqlc_slice: false,
                embed_table: None,
                original_name: "".to_owned(),
                unsigned: false,
                array_dims: 0,
            };

            assert_eq!(column_name(&col), "as_column_name")
        }
    }
}
