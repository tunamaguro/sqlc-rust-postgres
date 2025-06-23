use crate::db_support::PgColumn;
use crate::rust_gen::naming::{RustSelfIdent, has_single_table_identifier, generate_unique_field_names, column_name_from_list};
use crate::user_type::TypeMap;
use crate::{plugin, utils};
use proc_macro2::{Literal, Span};
use quote::{ToTokens, quote};
use syn::Ident;

/// PostgreSQL struct generator for query results
#[derive(Debug, Clone)]
pub(crate) struct PgStruct {
    pub(crate) name: String,
    pub(crate) columns: Vec<PgColumn>,
}

impl PgStruct {
    pub(crate) fn new(query: &plugin::Query, pg_map: &impl TypeMap) -> crate::Result<Self> {
        let is_single_table_identifier = has_single_table_identifier(query);

        // Generate unique field names to avoid conflicts
        let field_names = if is_single_table_identifier {
            // For single table, use simple names (already conflict-free)
            query
                .columns
                .iter()
                .enumerate()
                .map(|(idx, c)| {
                    if !c.name.is_empty() {
                        crate::utils::rust_struct_field(&c.name)
                    } else {
                        format!("column_{}", idx)
                    }
                })
                .collect()
        } else {
            // For multi-table, use unique name generation
            generate_unique_field_names(query)
        };

        let columns = query
            .columns
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                PgColumn::from_column(column_name_from_list(&field_names, idx), c, pg_map)
            })
            .collect::<crate::Result<Vec<_>>>()?;

        let name = utils::rust_value_ident(&query.name);
        let name = format!("{}Row", name);
        Ok(Self { name, columns })
    }

    pub(crate) fn to_from_row_expr(&self, var_ident: &Ident) -> proc_macro2::TokenStream {
        let mut st_inner = quote! {};
        for (idx, c) in self.columns.iter().enumerate() {
            let field_ident = Ident::new(&c.name, Span::call_site());
            let literal = Literal::usize_unsuffixed(idx);
            st_inner = quote! {
                #st_inner
                #field_ident: #var_ident.try_get(#literal)?,
            }
        }

        let ident = self.ident();
        quote! {
            #ident {
                #st_inner
            }
        }
    }
}

impl RustSelfIdent for PgStruct {
    fn ident_str(&self) -> String {
        self.name.clone()
    }
}

impl ToTokens for PgStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.columns.is_empty() {
            return;
        }

        let ident = self.ident();
        let columns = &self.columns;
        tokens.extend(quote! {
            pub struct #ident {
                #(#columns),*
            }
        });
    }
}