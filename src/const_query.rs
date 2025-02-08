use crate::plugin;
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

trait FromQuery: Send + Sync + Sized {
    type Error;
    fn from_query(query: &plugin::Query) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct PostgresQuery {
    name: String,
    query: String,
}

impl PostgresQuery {
    pub(crate) fn ident_str(&self) -> String {
        self.name.to_case(Case::UpperSnake)
    }

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
        let raw_str = format!("r#\"{}\"#", self.query);
        let raw_literal: proc_macro2::TokenStream =
            raw_str.parse().expect("Failed to parse raw literal");
        tokens.extend(quote! {
            const #ident: &str = #raw_literal;
        });
    }
}
