use crate::db_support::{PgColumn, PgColumnRef};
use crate::rust_gen::naming::{RustSelfIdent, column_name_from_list, generate_unique_param_names};
use crate::user_type::TypeMap;
use crate::{plugin, utils};
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

/// PostgreSQL parameters generator
#[derive(Debug, Clone)]
pub(crate) struct PgParams {
    pub(crate) name: String,
    pub(crate) params: Vec<PgColumnRef>,
}

impl PgParams {
    pub(crate) fn new(query: &plugin::Query, pg_map: &impl TypeMap) -> crate::Result<Self> {
        // reordering by number
        let mut params = query.params.clone();
        params.sort_by(|a, b| a.number.cmp(&b.number));

        // Check all parameter have column
        let params = params
            .iter()
            .map(|p| p.column.as_ref().map(|col| (p.number, col)))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| crate::Error::missing_col_info(&query.name))?;

        // Generate unique parameter names using dedicated function
        let param_field_names = generate_unique_param_names(&params);

        let params = params
            .iter()
            .enumerate()
            .map(|(idx, (col_idx, column))| {
                let _col_idx = *col_idx;
                PgColumn::from_column(
                    column_name_from_list(&param_field_names, idx),
                    column,
                    pg_map,
                )
            })
            .map(|v| v.map(PgColumnRef::new))
            .collect::<crate::Result<Vec<_>>>()?;
        let name = utils::rust_value_ident(&query.name);
        let name = format!("{}Params", name);
        Ok(Self { name, params })
    }

    pub(crate) fn to_func_args(
        &self,
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        if self.params.is_empty() {
            return Default::default();
        }

        let mut tokens = quote! {};

        for p in self.params.iter() {
            let param_tokens = p.to_tokens_with_type_map(type_map);
            tokens = quote! {#tokens #param_tokens,}
        }

        tokens
    }

    pub(crate) fn to_stmt_params(&self) -> proc_macro2::TokenStream {
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
