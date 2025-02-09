use crate::plugin;
use crate::sqlc_annotation::QueryAnnotation;
use crate::user_type::{col_type, TypeMap};
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::num::NonZeroUsize;
use syn::Ident;

pub(crate) trait GenericConstQuery {
    // query ident name
    fn ident_str(&self) -> String;
    fn ident(&self) -> Ident {
        Ident::new(&self.ident_str(), Span::call_site())
    }
    // sql query
    fn sql_str(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PostgresConstQuery {
    name: String,
    comment: String,
    query: String,
}

impl GenericConstQuery for PostgresConstQuery {
    fn ident_str(&self) -> String {
        self.name.to_case(Case::UpperSnake)
    }

    fn sql_str(&self) -> String {
        format!("{}{}", self.comment, self.query)
    }
}

impl PostgresConstQuery {
    pub(crate) fn new(query: &plugin::Query, query_type: &QueryAnnotation) -> Self {
        let name = query.name.clone();
        let comment = format!("-- name: {} {}\n", name, query_type);

        Self {
            name,
            comment,
            query: query.text.clone(),
        }
    }
}

impl ToTokens for PostgresConstQuery {
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

#[derive(Debug, Clone)]
pub(crate) struct PostgresQuery {
    _query_type: QueryAnnotation,
    query_const: PostgresConstQuery,
    returning_row: PgStruct,
    query_params: PgStruct,
}

impl PostgresQuery {
    pub(crate) fn new(query: &plugin::Query, pg_map: &impl TypeMap) -> Self {
        let query_type = query.cmd.parse::<QueryAnnotation>().unwrap();
        let query_const = PostgresConstQuery::new(query, &query_type);
        let returning_row = PgStruct::generate_row(query, pg_map);
        let query_params = PgStruct::generate_param(query, pg_map);

        Self {
            _query_type: query_type,
            query_const,
            returning_row,
            query_params,
        }
    }
}

impl ToTokens for PostgresQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            query_const,
            returning_row,
            query_params,
            ..
        } = self;

        tokens.extend(quote! {
            #query_const
            #returning_row
            #query_params
        });
    }
}

fn column_name(column: &plugin::Column, idx: usize) -> String {
    let name = if let Some(table) = &column.table {
        format!("{}_{}", table.name, column.name)
    } else if !column.name.is_empty() {
        column.name.clone()
    } else {
        // column name may empty
        format!("column_{}", idx)
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
    pub(crate) fn from_column(
        col_name: String,
        column: &plugin::Column,
        pg_map: &impl TypeMap,
    ) -> Self {
        let pg_type = column.r#type.as_ref();

        let rs_type = pg_map
            .get(&col_type(pg_type))
            .expect("Column type not found")
            .clone();

        let array_dim = NonZeroUsize::new(column.array_dims.try_into().unwrap_or(0));
        let is_nullable = !column.not_null;

        Self {
            name: col_name,
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
struct PgStruct {
    name: String,
    columns: Vec<PgColumn>,
}

impl PgStruct {
    fn generate_row(query: &plugin::Query, pg_map: &impl TypeMap) -> Self {
        let columns = query
            .columns
            .iter()
            .enumerate()
            .map(|(idx, c)| PgColumn::from_column(column_name(c, idx), c, pg_map))
            .collect::<Vec<_>>();

        let name = query.name.to_case(Case::Pascal);
        let name = format!("{}Row", name);
        Self { name, columns }
    }

    fn generate_param(query: &plugin::Query, pg_map: &impl TypeMap) -> Self {
        // reordering by number
        let mut params = query.params.clone();
        params.sort_by(|a, b| a.number.cmp(&b.number));

        // Check all parameter have column
        if params.iter().any(|p| p.column.is_none()) {
            std::process::exit(1)
        };

        let columns = params
            .iter()
            .map(|p| {
                PgColumn::from_column(
                    column_name(p.column.as_ref().unwrap(), p.number.try_into().unwrap_or(0)),
                    p.column.as_ref().unwrap(),
                    pg_map,
                )
            })
            .collect::<Vec<_>>();
        let name = query.name.to_case(Case::Pascal);
        let name = format!("{}Params", name);
        Self { name, columns }
    }
}

impl ToTokens for PgStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.columns.is_empty() {
            return;
        }

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

            assert_eq!(column_name(&col, 0), "author_name")
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

            assert_eq!(column_name(&col, 0), "as_column_name")
        }
    }
}
