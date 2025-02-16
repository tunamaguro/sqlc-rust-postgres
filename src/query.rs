use crate::plugin;
use crate::sqlc_annotation::QueryAnnotation;
use crate::user_type::{col_type, TypeMap};
use convert_case::{Case, Casing};
use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use std::num::NonZeroUsize;
use syn::Ident;

pub(crate) trait RustSelfIdent {
    fn ident_str(&self) -> String;
    fn ident(&self) -> Ident {
        Ident::new(&self.ident_str(), Span::call_site())
    }
}

pub(crate) trait GenericConstQuery {
    // sql query
    fn sql_str(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PostgresConstQuery {
    name: String,
    comment: String,
    query: String,
}
impl RustSelfIdent for PostgresConstQuery {
    fn ident_str(&self) -> String {
        self.name.to_case(Case::UpperSnake)
    }
}

impl GenericConstQuery for PostgresConstQuery {
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
struct PostgresFunc {
    query_name: String,
    annotation: QueryAnnotation,
    use_async: bool,
}

impl PostgresFunc {
    fn new(query: &plugin::Query, annotation: QueryAnnotation, use_async: bool) -> Self {
        let query_name = query.name.to_case(Case::Snake);
        Self {
            query_name,
            annotation,
            use_async,
        }
    }

    fn client_ident(&self) -> proc_macro2::TokenStream {
        if self.use_async {
            "& impl tokio_postgres::GenericClient".parse().unwrap()
        } else {
            "&mut impl postgres::GenericClient".parse().unwrap()
        }
    }

    fn error_ident(&self) -> proc_macro2::TokenStream {
        if self.use_async {
            "tokio_postgres::Error".parse().unwrap()
        } else {
            "postgres::Error".parse().unwrap()
        }
    }

    fn func_def(&self, query_params: &PgParams) -> proc_macro2::TokenStream {
        let func_ident = self.ident();
        let client_ident = self.client_ident();
        let args = query_params.to_func_args();

        let async_ident = if self.use_async {
            quote! {async}
        } else {
            quote! {}
        };

        quote! {
            pub #async_ident fn #func_ident(client:#client_ident,#args)
        }
    }

    fn await_def(&self) -> proc_macro2::TokenStream {
        if self.use_async {
            quote! {.await}
        } else {
            quote! {}
        }
    }

    fn generate_exec(
        &self,
        query_const: &PostgresConstQuery,
        query_params: &PgParams,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params);
        let await_def = self.await_def();
        let error_ident = self.error_ident();

        let query_ident = query_const.ident();
        let params = query_params.to_stmt_params();
        quote! {
            #func_def -> Result<u64,#error_ident> {
                client.execute(#query_ident,#params)#await_def
            }
        }
    }

    fn generate_one(
        &self,
        query_const: &PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params);
        let await_def = self.await_def();

        let error_ident = self.error_ident();

        let query_ident = query_const.ident();
        let returning_ident = returning_row.ident();
        let params = query_params.to_stmt_params();

        let row_ident = Ident::new("row", Span::call_site());
        let from_expr = returning_row.to_from_row_expr(&row_ident);

        quote! {
            #func_def -> Result<#returning_ident,#error_ident> {
                let #row_ident = client.query_one(#query_ident,#params)#await_def?;
                Ok(#from_expr)
            }
        }
    }

    fn generate_many(
        &self,
        query_const: &PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params);
        let await_def = self.await_def();

        let error_ident = self.error_ident();

        let query_ident = query_const.ident();
        let returning_ident = returning_row.ident();
        let params = query_params.to_stmt_params();

        let rows_ident = Ident::new("rows", Span::call_site());
        let row_ident = Ident::new("r", Span::call_site());
        let from_expr = returning_row.to_from_row_expr(&row_ident);

        quote! {
            #func_def -> Result<impl Iterator<Item = Result<#returning_ident,#error_ident>>,#error_ident> {
                let #rows_ident = client.query(#query_ident,#params)#await_def?;
                Ok(#rows_ident.into_iter().map(|#row_ident|Ok(#from_expr)))
            }
        }
    }

    fn generate(
        &self,
        query_const: &PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
    ) -> proc_macro2::TokenStream {
        match self.annotation {
            QueryAnnotation::Exec => self.generate_exec(query_const, query_params),
            QueryAnnotation::One => self.generate_one(query_const, returning_row, query_params),
            QueryAnnotation::Many => self.generate_many(query_const, returning_row, query_params),
            _ => {
                panic!("query annotation `{}` is not supported", self.annotation)
            }
        }
    }
}

impl RustSelfIdent for PostgresFunc {
    fn ident_str(&self) -> String {
        self.query_name.clone()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PostgresQuery {
    query_type: QueryAnnotation,
    query_const: PostgresConstQuery,
    returning_row: PgStruct,
    query_params: PgParams,
    query_func: PostgresFunc,
}

impl PostgresQuery {
    pub(crate) fn new(
        query: &plugin::Query,
        pg_map: &impl TypeMap,
        use_async: bool,
    ) -> crate::Result<Self> {
        let query_type = query.cmd.parse::<QueryAnnotation>().unwrap();

        let query_const = PostgresConstQuery::new(query, &query_type);
        let returning_row = PgStruct::new(query, pg_map)?;
        let query_params = PgParams::new(query, pg_map)?;
        let query_func = PostgresFunc::new(query, query_type.clone(), use_async);
        Ok(Self {
            query_type,
            query_const,
            returning_row,
            query_params,
            query_func,
        })
    }

    pub(crate) fn with_derive(
        &self,
        row_derive: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let Self {
            query_const,
            returning_row,
            query_params,
            query_type,
            query_func,
            ..
        } = self;

        let query_func = query_func.generate(query_const, returning_row, query_params);
        match query_type {
            QueryAnnotation::Exec => {
                quote! {
                    #query_const
                    #query_func
                }
            }
            _ => {
                quote! {
                    #query_const
                    #row_derive
                    #returning_row
                    #query_func
                }
            }
        }
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
    ) -> crate::Result<Self> {
        let pg_type = column
            .r#type
            .as_ref()
            .ok_or_else(|| crate::Error::col_type_not_found(&col_name))?;

        let col_type = col_type(pg_type);
        let rs_type = pg_map.get(&col_type)?.to_token_stream();

        let array_dim = NonZeroUsize::new(column.array_dims.try_into().unwrap_or(0));
        let is_nullable = !column.not_null;

        Ok(Self {
            name: col_name,
            rs_type,
            array_dim,
            is_nullable,
        })
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
            pub #field_ident: #ty_tokens
        });
    }
}

/// generate ref type for params
#[derive(Debug, Clone)]
struct PgColumnRef {
    inner: PgColumn,
}

impl PgColumnRef {
    fn new(inner: PgColumn) -> Self {
        PgColumnRef { inner }
    }

    /// convert type utility. do below  
    /// - `String` to `str`
    /// - `Vec<T>` to `&[T]`
    fn wrap_type(&self) -> proc_macro2::TokenStream {
        let rs_type = self.inner.rs_type.clone();

        let dim = if let Some(dim) = self.inner.array_dim {
            dim.get()
        } else {
            let rs_type_str = rs_type.to_string();
            if rs_type_str == "String" {
                return quote! { str };
            } else {
                return rs_type;
            }
        };

        let mut inner = rs_type;
        if dim > 1 {
            for _ in 1..dim {
                inner = quote! { Vec<#inner>};
            }
        }

        quote! {[#inner]}
    }
}

impl ToTokens for PgColumnRef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_ident = Ident::new(&self.inner.name, Span::call_site());
        let rs_type = self.wrap_type();

        let ref_type = if self.inner.is_nullable {
            quote! {Option<& #rs_type>}
        } else {
            quote! {& #rs_type}
        };

        tokens.extend(quote! {
            #field_ident: #ref_type
        });
    }
}

#[derive(Debug, Clone)]
struct PgStruct {
    name: String,
    columns: Vec<PgColumn>,
}

impl PgStruct {
    fn new(query: &plugin::Query, pg_map: &impl TypeMap) -> crate::Result<Self> {
        let columns = query
            .columns
            .iter()
            .enumerate()
            .map(|(idx, c)| PgColumn::from_column(column_name(c, idx), c, pg_map))
            .collect::<crate::Result<Vec<_>>>()?;

        let name = query.name.to_case(Case::Pascal);
        let name = format!("{}Row", name);
        Ok(Self { name, columns })
    }

    fn to_from_row_expr(&self, var_ident: &Ident) -> proc_macro2::TokenStream {
        let mut st_inner = quote! {};
        for (idx, c) in self.columns.iter().enumerate() {
            let field_ident = Ident::new(&c.name, Span::call_site());
            let literal = Literal::usize_unsuffixed(idx);
            st_inner = quote! {
                #st_inner
                #field_ident: #var_ident.try_get(#literal)?,
            }
        }

        let ident = self.ident();
        quote! {
            #ident {
                #st_inner
            }
        }
    }
}

impl RustSelfIdent for PgStruct {
    fn ident_str(&self) -> String {
        self.name.clone()
    }
}

impl ToTokens for PgStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.columns.is_empty() {
            return;
        }

        let ident = self.ident();
        let columns = &self.columns;
        tokens.extend(quote! {
            pub struct #ident {
                #(#columns),*
            }
        });
    }
}

#[derive(Debug, Clone)]
struct PgParams {
    name: String,
    params: Vec<PgColumnRef>,
}

impl PgParams {
    fn new(query: &plugin::Query, pg_map: &impl TypeMap) -> crate::Result<Self> {
        // reordering by number
        let mut params = query.params.clone();
        params.sort_by(|a, b| a.number.cmp(&b.number));

        // Check all parameter have column
        if params.iter().any(|p| p.column.is_none()) {
            std::process::exit(1)
        };

        let params = params
            .iter()
            .map(|p| {
                PgColumn::from_column(
                    column_name(p.column.as_ref().unwrap(), p.number.try_into().unwrap_or(0)),
                    p.column.as_ref().unwrap(),
                    pg_map,
                )
            })
            .map(|v| v.map(PgColumnRef::new))
            .collect::<crate::Result<Vec<_>>>()?;
        let name = query.name.to_case(Case::Pascal);
        let name = format!("{}Params", name);
        Ok(Self { name, params })
    }

    fn to_func_args(&self) -> proc_macro2::TokenStream {
        if self.params.is_empty() {
            return Default::default();
        }

        let mut tokens = quote! {};

        for p in self.params.iter() {
            tokens = quote! {#tokens #p,}
        }

        tokens
    }

    fn to_stmt_params(&self) -> proc_macro2::TokenStream {
        let mut tokens = quote! {};

        for p in self.params.iter() {
            let ident = Ident::new(&p.inner.name, Span::call_site());
            tokens = quote! {#tokens &#ident,}
        }

        quote! {&[#tokens]}
    }
}

impl RustSelfIdent for PgParams {
    fn ident_str(&self) -> String {
        self.name.clone()
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
