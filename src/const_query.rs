use crate::plugin;
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

pub(crate) trait GenericQuery {
    // query ident name
    fn ident_str(&self) -> String;
    // sql query
    fn sql_str(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
        let ident = Ident::new(&self.ident_str(), Span::call_site());
        let raw_str = format!("r#\"{}\"#", self.sql_str());
        let raw_literal: proc_macro2::TokenStream =
            raw_str.parse().expect("Failed to parse raw literal");
        tokens.extend(quote! {
            const #ident: &str = #raw_literal;
        });
    }
}
