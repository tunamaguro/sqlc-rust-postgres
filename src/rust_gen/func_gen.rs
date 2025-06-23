use crate::db_support::DbCrate;
use crate::rust_gen::const_gen::PostgresConstQuery;
use crate::rust_gen::naming::RustSelfIdent;
use crate::sqlc::QueryAnnotation;
use crate::utils;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use super::param_gen::PgParams;
use super::struct_gen::PgStruct;

/// PostgreSQL function generator
#[derive(Debug, Clone)]
pub(crate) struct PostgresFunc {
    query_name: String,
    annotation: QueryAnnotation,
    db_crate: DbCrate,
}

impl PostgresFunc {
    pub(crate) fn new(
        query: &crate::plugin::Query,
        annotation: QueryAnnotation,
        db_crate: DbCrate,
    ) -> Self {
        let query_name = utils::rust_fn_ident(&query.name);
        Self {
            query_name,
            annotation,
            db_crate,
        }
    }

    fn func_def(&self, query_params: &PgParams) -> proc_macro2::TokenStream {
        let func_ident = self.ident();
        let client_ident = self.db_crate.client_ident();
        let args = query_params.to_func_args();

        let async_ident = self.db_crate.async_ident();

        quote! {
            pub #async_ident fn #func_ident(client:#client_ident,#args)
        }
    }

    fn generate_exec(
        &self,
        query_const: &PostgresConstQuery,
        query_params: &PgParams,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params);
        let await_def = self.db_crate.await_ident();
        let error_ident = self.db_crate.error_ident();

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
        let await_def = self.db_crate.await_ident();

        let error_ident = self.db_crate.error_ident();

        let query_ident = query_const.ident();
        let returning_ident = returning_row.ident();
        let params = query_params.to_stmt_params();

        let row_ident = Ident::new("row", Span::call_site());
        let val_ident = Ident::new("v", Span::call_site());
        let from_expr = returning_row.to_from_row_expr(&val_ident);

        quote! {
            #func_def -> Result<Option<#returning_ident>,#error_ident> {
                let #row_ident = client.query_opt(#query_ident,#params)#await_def?;
                let #val_ident = match #row_ident {
                    Some(#val_ident) => #from_expr,
                    None => return Ok(None),
                };

                Ok(Some(#val_ident))
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
        let await_def = self.db_crate.await_ident();

        let error_ident = self.db_crate.error_ident();

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

    pub(crate) fn generate(
        &self,
        query_const: &PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
    ) -> crate::Result<proc_macro2::TokenStream> {
        match self.annotation {
            QueryAnnotation::Exec => Ok(self.generate_exec(query_const, query_params)),
            QueryAnnotation::One => Ok(self.generate_one(query_const, returning_row, query_params)),
            QueryAnnotation::Many => {
                Ok(self.generate_many(query_const, returning_row, query_params))
            }
            _ => Err(crate::Error::unsupported_annotation(
                self.annotation.to_string(),
            )),
        }
    }
}

impl RustSelfIdent for PostgresFunc {
    fn ident_str(&self) -> String {
        self.query_name.clone()
    }
}
