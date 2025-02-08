use crate::plugin;
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

pub(crate) trait GenericEnum {
    fn ident_str(&self) -> String;
    fn ident(&self) -> Ident {
        Ident::new(&self.ident_str(), Span::call_site())
    }
}

pub(crate) struct PostgresEnum {
    name: String,
    values: Vec<String>,
}

impl PostgresEnum {
    pub(crate) fn new(catalog_enum: &plugin::Enum) -> Self {
        let name = catalog_enum.name.clone();
        let values = catalog_enum.vals.clone();
        Self { name, values }
    }
}

impl GenericEnum for PostgresEnum {
    fn ident_str(&self) -> String {
        self.name.to_case(Case::Pascal)
    }
}

impl ToTokens for PostgresEnum {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident();
        let variants = self
            .values
            .iter()
            .map(|v| Ident::new(v, Span::call_site()))
            .collect::<Vec<_>>();

        let tt = quote! {
            pub enum #ident {
                #(#variants),*
            }
        };

        tokens.extend(tt);
    }
}
