use crate::sqlc::QueryAnnotation;
use crate::{plugin, utils};
use quote::quote;

use super::naming::RustSelfIdent;

/// Trait for generating SQL query constants
pub(crate) trait GenericConstQuery {
    // sql query
    fn sql_str(&self) -> String;
}

/// PostgreSQL constant query generator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PostgresConstQuery {
    name: String,
    comment: String,
    query: String,
}

impl RustSelfIdent for PostgresConstQuery {
    fn ident_str(&self) -> String {
        utils::rust_const_ident(&self.name)
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

    pub(crate) fn to_tokens(&self) -> crate::Result<proc_macro2::TokenStream> {
        let ident = self.ident();
        let raw_str = format!("r#\"{}\"#", self.sql_str());
        let raw_literal = raw_str.parse::<proc_macro2::TokenStream>().map_err(|_| {
            crate::Error::any_error(format!("Failed to parse raw literal({})", raw_str))
        })?;
        Ok(quote! {
            pub const #ident: &str = #raw_literal;
        })
    }
}