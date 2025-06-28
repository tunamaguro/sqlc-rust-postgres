use crate::db_support::DbCrate;
use crate::plugin;
use crate::rust_gen::const_gen::PostgresConstQuery;
use crate::rust_gen::func_gen::PostgresFunc;
use crate::rust_gen::param_gen::PgParams;
use crate::rust_gen::struct_api_gen::PostgresStructApi;
use crate::rust_gen::struct_gen::PgStruct;
use crate::sqlc::QueryAnnotation;
use crate::user_type::TypeMap;
use quote::quote;

#[derive(Debug, Clone)]
pub(crate) struct PostgresQuery {
    query_type: QueryAnnotation,
    query_const: PostgresConstQuery,
    returning_row: PgStruct,
    query_params: PgParams,
    query_func: PostgresFunc,
    struct_api: PostgresStructApi,
}

impl PostgresQuery {
    pub(crate) fn new(
        query: &plugin::Query,
        pg_map: &impl TypeMap,
        db_crate: DbCrate,
    ) -> crate::Result<Self> {
        let query_type = query.cmd.parse::<QueryAnnotation>().unwrap();

        let query_const = PostgresConstQuery::new(query, &query_type);
        let returning_row = PgStruct::new(query, pg_map, db_crate)?;
        let query_params = PgParams::new(query, pg_map)?;
        let query_func = PostgresFunc::new(query, query_type.clone(), db_crate);
        let struct_api = PostgresStructApi::new(query, query_type.clone(), db_crate);
        Ok(Self {
            query_type,
            query_const,
            returning_row,
            query_params,
            query_func,
            struct_api,
        })
    }

    pub(crate) fn with_derive(
        &self,
        row_derive: &proc_macro2::TokenStream,
        type_map: &impl crate::user_type::TypeMap,
    ) -> crate::Result<proc_macro2::TokenStream> {
        let Self {
            query_const,
            returning_row,
            query_params,
            query_type,
            query_func,
            struct_api,
        } = self;
        // Generate struct-based API only if there are parameters
        let struct_api_tokens = if !query_params.params.is_empty() {
            let query_struct =
                struct_api.generate_query_struct(query_const, query_params, type_map);
            let execution_methods = struct_api.generate_execution_methods(
                query_const,
                returning_row,
                query_params,
                type_map,
            );
            // Temporarily disable builder generation to test basic struct API
            // let builder_pattern = builder_gen.generate_builder(query_params, type_map);

            quote! {
                #query_struct
                #execution_methods
                // #builder_pattern
            }
        } else {
            quote! {}
        };

        let query_func = query_func.generate(query_const, returning_row, query_params, type_map)?;

        // Always generate standalone query constant for backward compatibility
        let query_tt = query_const.to_tokens()?;

        let tokens = match query_type {
            QueryAnnotation::Exec => {
                quote! {
                    #query_tt
                    #query_func
                    #struct_api_tokens
                }
            }
            _ => {
                quote! {
                    #query_tt
                    #row_derive
                    #returning_row
                    #query_func
                    #struct_api_tokens
                }
            }
        };

        Ok(tokens)
    }
}
