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

    fn func_def(
        &self,
        query_params: &PgParams,
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        let func_ident = self.ident();
        let client_ident = self.db_crate.client_ident();
        let args = query_params.to_func_args(type_map);

        let async_ident = self.db_crate.async_ident();

        quote! {
            pub #async_ident fn #func_ident(client:#client_ident,#args)
        }
    }

    fn generate_exec(
        &self,
        query_const: &PostgresConstQuery,
        query_params: &PgParams,
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params, type_map);
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
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params, type_map);
        let error_ident = self.db_crate.error_ident();
        let returning_ident = returning_row.ident();

        // If there are parameters, use the struct API internally
        if !query_params.params.is_empty() {
            let struct_ident = Ident::new(
                &crate::utils::rust_value_ident(&self.query_name),
                Span::call_site(),
            );
            let field_assignments = self.generate_struct_field_assignments(query_params, type_map);

            quote! {
                #func_def -> Result<Option<#returning_ident>,#error_ident> {
                    let query_struct = #struct_ident {
                        #field_assignments
                    };
                    query_struct.query_opt(client).await
                }
            }
        } else {
            // For queries without parameters, keep the original implementation
            let await_def = self.db_crate.await_ident();
            let query_ident = query_const.ident();
            let params = query_params.to_stmt_params();
            let row_ident = Ident::new("row", Span::call_site());

            quote! {
                #func_def -> Result<Option<#returning_ident>,#error_ident> {
                    let #row_ident = client.query_opt(#query_ident,#params)#await_def?;
                    match #row_ident {
                        Some(ref #row_ident) => Ok(Some(#returning_ident::from_row(#row_ident)?)),
                        None => Ok(None),
                    }
                }
            }
        }
    }

    fn generate_many(
        &self,
        query_const: &PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        let func_def = self.func_def(query_params, type_map);
        let await_def = self.db_crate.await_ident();

        let error_ident = self.db_crate.error_ident();

        let query_ident = query_const.ident();
        let returning_ident = returning_row.ident();
        let params = query_params.to_stmt_params();

        let rows_ident = Ident::new("rows", Span::call_site());
        let row_ident = Ident::new("r", Span::call_site());

        quote! {
            #func_def -> Result<impl Iterator<Item = Result<#returning_ident,#error_ident>>,#error_ident> {
                let #rows_ident = client.query(#query_ident,#params)#await_def?;
                Ok(#rows_ident.into_iter().map(|#row_ident| #returning_ident::from_row(&#row_ident)))
            }
        }
    }

    fn generate_struct_field_assignments(
        &self,
        query_params: &PgParams,
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        let mut assignments = quote! {};

        for param in query_params.params.iter() {
            let field_name = &param.inner.name;
            let field_ident = Ident::new(field_name, Span::call_site());
            let param_ident = Ident::new(field_name, Span::call_site());

            if param.is_copy_cheap_type(type_map) {
                // Copy-cheap types: direct assignment
                assignments.extend(quote! {
                    #field_ident: #param_ident,
                });
            } else {
                // Non-copy types: handle Option<T> vs T appropriately
                if !param.inner.is_nullable {
                    // Non-optional: T -> Cow::Borrowed(T)
                    assignments.extend(quote! {
                        #field_ident: std::borrow::Cow::Borrowed(#param_ident),
                    });
                } else {
                    // Optional: Option<T> -> Option<Cow<'a, T>>
                    assignments.extend(quote! {
                        #field_ident: #param_ident.map(std::borrow::Cow::Borrowed),
                    });
                }
            }
        }

        assignments
    }

    pub(crate) fn generate(
        &self,
        query_const: &PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
        type_map: &impl crate::user_type::TypeMap,
    ) -> crate::Result<proc_macro2::TokenStream> {
        match self.annotation {
            QueryAnnotation::Exec => Ok(self.generate_exec(query_const, query_params, type_map)),
            QueryAnnotation::One => {
                Ok(self.generate_one(query_const, returning_row, query_params, type_map))
            }
            QueryAnnotation::Many => {
                Ok(self.generate_many(query_const, returning_row, query_params, type_map))
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
