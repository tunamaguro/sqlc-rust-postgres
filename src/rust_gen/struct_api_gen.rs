use crate::db_support::DbCrate;
use crate::rust_gen::naming::RustSelfIdent;
use crate::rust_gen::param_gen::PgParams;
use crate::rust_gen::struct_gen::PgStruct;
use crate::sqlc::QueryAnnotation;
use crate::user_type::TypeMap;
use proc_macro2::TokenStream;
use quote::quote;

/// Struct-based API generator for type-safe query building
#[derive(Debug, Clone)]
pub(crate) struct PostgresStructApi {
    query_name: String,
    annotation: QueryAnnotation,
    db_crate: DbCrate,
}

impl PostgresStructApi {
    pub(crate) fn new(
        query: &crate::plugin::Query,
        annotation: QueryAnnotation,
        db_crate: DbCrate,
    ) -> Self {
        let query_name = crate::utils::rust_value_ident(&query.name);
        Self {
            query_name,
            annotation,
            db_crate,
        }
    }

    /// Generate the main query struct that holds parameters
    pub(crate) fn generate_query_struct(
        &self,
        query_const: &crate::rust_gen::const_gen::PostgresConstQuery,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let mut field_tokens = quote! {};
        let has_lifetime = self.needs_lifetime(query_params, type_map);
        let query_const_tokens = query_const.as_struct_const().unwrap_or_default();

        // Generate struct fields with optimized types
        for param in &query_params.params {
            let field_name = &param.inner.name;
            let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());

            if param.is_copy_cheap_type(type_map) {
                // Copy-cheap types: pass by value
                let rs_type = &param.inner.rs_type;
                if param.inner.is_nullable {
                    field_tokens.extend(quote! {
                        pub #field_ident: Option<#rs_type>,
                    });
                } else {
                    field_tokens.extend(quote! {
                        pub #field_ident: #rs_type,
                    });
                }
            } else {
                // Non-copy types: use Cow for optimization
                let base_type = param.wrap_type();
                if param.inner.is_nullable {
                    field_tokens.extend(quote! {
                        pub #field_ident: Option<std::borrow::Cow<'a, #base_type>>,
                    });
                } else {
                    field_tokens.extend(quote! {
                        pub #field_ident: std::borrow::Cow<'a, #base_type>,
                    });
                }
            }
        }

        let lifetime_param = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        quote! {
            #[derive(Debug)]
            pub struct #struct_ident #lifetime_param {
                #field_tokens
            }

            impl #lifetime_param #struct_ident #lifetime_param {
                #query_const_tokens
            }
        }
    }

    /// Generate execution methods based on QueryAnnotation
    pub(crate) fn generate_execution_methods(
        &self,
        query_const: &crate::rust_gen::const_gen::PostgresConstQuery,
        returning_row: &PgStruct,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let has_lifetime = self.needs_lifetime(query_params, type_map);
        let lifetime_param = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        let client_ident = self.db_crate.client_ident();
        let error_ident = self.db_crate.error_ident();
        let await_def = self.db_crate.await_ident();
        let async_def = self.db_crate.async_ident();
        let _query_ident = query_const.ident();

        // Generate parameter passing for SQL execution
        let params = self.generate_stmt_params(query_params, type_map);

        match self.annotation {
            QueryAnnotation::One => {
                let returning_ident = returning_row.ident();
                let row_ident = syn::Ident::new("row", proc_macro2::Span::call_site());

                quote! {
                    impl #lifetime_param #struct_ident #lifetime_param {
                        pub #async_def fn query_one(&self, client: #client_ident) -> Result<#returning_ident, #error_ident> {
                            let #row_ident = client.query_one(Self::QUERY, #params)#await_def?;
                            #returning_ident::from_row(&#row_ident)
                        }

                        pub #async_def fn query_opt(&self, client: #client_ident) -> Result<Option<#returning_ident>, #error_ident> {
                            let #row_ident = client.query_opt(Self::QUERY, #params)#await_def?;
                            match #row_ident {
                                Some(ref #row_ident) => Ok(Some(#returning_ident::from_row(#row_ident)?)),
                                None => Ok(None),
                            }
                        }
                    }
                }
            }
            QueryAnnotation::Many => {
                let returning_ident = returning_row.ident();
                let rows_ident = syn::Ident::new("rows", proc_macro2::Span::call_site());
                let row_ident = syn::Ident::new("r", proc_macro2::Span::call_site());

                quote! {
                    impl #lifetime_param #struct_ident #lifetime_param {
                        pub #async_def fn query(&self, client: #client_ident) -> Result<impl Iterator<Item = Result<#returning_ident, #error_ident>>, #error_ident> {
                            let #rows_ident = client.query(Self::QUERY, #params)#await_def?;
                            Ok(#rows_ident.into_iter().map(|#row_ident| #returning_ident::from_row(&#row_ident)))
                        }

                        pub #async_def fn query_many(&self, client: #client_ident) -> Result<Vec<#returning_ident>, #error_ident> {
                            let #rows_ident = client.query(Self::QUERY, #params)#await_def?;
                            #rows_ident.into_iter().map(|#row_ident| #returning_ident::from_row(&#row_ident)).collect()
                        }

                        pub #async_def fn query_raw(&self, client: #client_ident) -> Result<impl Iterator<Item = Result<#returning_ident, #error_ident>>, #error_ident> {
                            let #rows_ident = client.query(Self::QUERY, #params)#await_def?;
                            Ok(#rows_ident.into_iter().map(|#row_ident| #returning_ident::from_row(&#row_ident)))
                        }
                    }
                }
            }
            QueryAnnotation::Exec => {
                quote! {
                    impl #lifetime_param #struct_ident #lifetime_param {
                        pub #async_def fn execute(&self, client: #client_ident) -> Result<u64, #error_ident> {
                            client.execute(Self::QUERY, #params)#await_def
                        }
                    }
                }
            }
            _ => {
                // For unsupported annotations, return empty implementation
                quote! {}
            }
        }
    }

    fn query_struct_ident(&self) -> syn::Ident {
        syn::Ident::new(&self.query_name, proc_macro2::Span::call_site())
    }

    fn needs_lifetime(&self, query_params: &PgParams, type_map: &impl TypeMap) -> bool {
        query_params
            .params
            .iter()
            .any(|param| !param.is_copy_cheap_type(type_map))
    }

    fn generate_stmt_params(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let mut param_tokens = quote! {};

        for param in &query_params.params {
            let field_name = &param.inner.name;
            let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());

            if param.is_copy_cheap_type(type_map) {
                // Copy-cheap types: pass by reference for SQL execution
                param_tokens.extend(quote! { &self.#field_ident, });
            } else {
                // Non-copy types: handle Cow and Option<Cow> appropriately
                if !param.inner.is_nullable {
                    // Non-optional: Cow<'a, T> -> .as_ref() returns &T, need & for ToSql
                    param_tokens.extend(quote! { &self.#field_ident.as_ref(), });
                } else {
                    // Optional: Option<Cow<'a, T>> -> .as_deref() returns Option<&T>, need & for ToSql
                    param_tokens.extend(quote! { &self.#field_ident.as_deref(), });
                }
            }
        }

        quote! { &[#param_tokens] }
    }
}

impl RustSelfIdent for PostgresStructApi {
    fn ident_str(&self) -> String {
        self.query_name.clone()
    }
}
