use crate::plugin;
use crate::user_type::{TypeMap, col_type};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::num::NonZeroUsize;
use syn::Ident;

/// Represents a PostgreSQL column with Rust type mapping
#[derive(Debug, Clone)]
pub(crate) struct PgColumn {
    pub(crate) name: String,
    pub(crate) rs_type: TokenStream,
    /// None => not array
    pub(crate) array_dim: Option<NonZeroUsize>,
    pub(crate) is_nullable: bool,
}

impl PgColumn {
    pub(crate) fn from_column(
        col_name: String,
        column: &plugin::Column,
        pg_map: &impl TypeMap,
    ) -> crate::Result<Self> {
        let pg_type = column
            .r#type
            .as_ref()
            .ok_or_else(|| crate::Error::missing_col_info(&col_name))?;

        let col_type = col_type(pg_type);
        let rs_type = pg_map.get(&col_type)?.to_token_stream();

        let array_dim = NonZeroUsize::new(column.array_dims.try_into().unwrap_or(0));
        let is_nullable = !column.not_null;

        Ok(Self {
            name: col_name,
            rs_type,
            array_dim,
            is_nullable,
        })
    }
}

impl ToTokens for PgColumn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ident = Ident::new(&self.name, Span::call_site());
        let rs_type = &self.rs_type;
        let mut ty_tokens = quote! { #rs_type };

        if let Some(dim) = self.array_dim {
            for _ in 0..dim.get() {
                ty_tokens = quote! { Vec<#ty_tokens> };
            }
        }

        if self.is_nullable {
            ty_tokens = quote! { Option<#ty_tokens> };
        }

        tokens.extend(quote! {
            pub #field_ident: #ty_tokens
        });
    }
}

/// Generate ref type for params
#[derive(Debug, Clone)]
pub(crate) struct PgColumnRef {
    pub(crate) inner: PgColumn,
}

impl PgColumnRef {
    pub(crate) fn new(inner: PgColumn) -> Self {
        PgColumnRef { inner }
    }

    /// convert type utility. do below  
    /// - `String` to `str`
    /// - `Vec<T>` to `&[T]`
    pub(crate) fn wrap_type(&self) -> TokenStream {
        let rs_type = self.inner.rs_type.clone();

        let dim = match self.inner.array_dim {
            Some(dim) => dim.get(),
            _ => {
                let rs_type_str = rs_type.to_string();
                if rs_type_str == "String" {
                    return quote! { str };
                } else {
                    return rs_type;
                }
            }
        };

        let mut inner = rs_type;
        if dim > 1 {
            for _ in 1..dim {
                inner = quote! { Vec<#inner>};
            }
        }

        quote! {[#inner]}
    }

    /// Check if the type is copy-cheap (should be passed by value rather than reference)
    pub(crate) fn is_copy_cheap_type(&self, type_map: &impl crate::user_type::TypeMap) -> bool {
        // Only consider non-array types for copy-cheap optimization
        if self.inner.array_dim.is_some() {
            return false;
        }

        let rs_type_str = self.inner.rs_type.to_string();
        type_map.is_copy_cheap_type(&rs_type_str)
    }

    /// Generate tokens with type map for copy-cheap type detection
    pub(crate) fn to_tokens_with_type_map(
        &self,
        type_map: &impl crate::user_type::TypeMap,
    ) -> proc_macro2::TokenStream {
        let field_ident = Ident::new(&self.inner.name, Span::call_site());
        let rs_type = self.wrap_type();

        let param_type = if self.is_copy_cheap_type(type_map) {
            // For copy-cheap types, pass by value
            if self.inner.is_nullable {
                quote! {Option<#rs_type>}
            } else {
                quote! {#rs_type}
            }
        } else {
            // For expensive types, pass by reference
            if self.inner.is_nullable {
                quote! {Option<& #rs_type>}
            } else {
                quote! {& #rs_type}
            }
        };

        quote! {
            #field_ident: #param_type
        }
    }
}

impl ToTokens for PgColumnRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ident = Ident::new(&self.inner.name, Span::call_site());
        let rs_type = self.wrap_type();

        // Fallback to reference for backward compatibility when type map not available
        let param_type = if self.inner.is_nullable {
            quote! {Option<& #rs_type>}
        } else {
            quote! {& #rs_type}
        };

        tokens.extend(quote! {
            #field_ident: #param_type
        });
    }
}
