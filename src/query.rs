use crate::db_support::DbCrate;
use crate::plugin;
use crate::rust_gen::const_gen::PostgresConstQuery;
use crate::rust_gen::func_gen::PostgresFunc;
use crate::rust_gen::param_gen::PgParams;
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
}

impl PostgresQuery {
    pub(crate) fn new(
        query: &plugin::Query,
        pg_map: &impl TypeMap,
        db_crate: DbCrate,
    ) -> crate::Result<Self> {
        let query_type = query.cmd.parse::<QueryAnnotation>().unwrap();

        let query_const = PostgresConstQuery::new(query, &query_type);
        let returning_row = PgStruct::new(query, pg_map)?;
        let query_params = PgParams::new(query, pg_map)?;
        let query_func = PostgresFunc::new(query, query_type.clone(), db_crate);
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
    ) -> crate::Result<proc_macro2::TokenStream> {
        let Self {
            query_const,
            returning_row,
            query_params,
            query_type,
            query_func,
            ..
        } = self;
        let query_tt = query_const.to_tokens()?;
        let query_func = query_func.generate(query_const, returning_row, query_params)?;
        let tokens = match query_type {
            QueryAnnotation::Exec => {
                quote! {
                    #query_tt
                    #query_func
                }
            }
            _ => {
                quote! {
                    #query_tt
                    #row_derive
                    #returning_row
                    #query_func
                }
            }
        };

        Ok(tokens)
    }
}
