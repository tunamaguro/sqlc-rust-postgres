use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Deserializer};

/// Supported PostgreSQL database crates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DbCrate {
    #[default]
    TokioPostgres,
    Postgres,
    DeadPoolPostgres,
}

impl<'de> Deserialize<'de> for DbCrate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "tokio_postgres" => Ok(DbCrate::TokioPostgres),
            "postgres" => Ok(DbCrate::Postgres),
            "deadpool_postgres" => Ok(DbCrate::DeadPoolPostgres),
            _ => Err(serde::de::Error::custom(format!("unknown db crate: {}", s))),
        }
    }
}

impl DbCrate {
    /// Returns the client type tokens for the specific database crate
    pub(crate) fn client_ident(&self) -> TokenStream {
        match self {
            DbCrate::TokioPostgres => {
                quote! {&impl tokio_postgres::GenericClient}
            }
            DbCrate::Postgres => {
                quote! {&mut impl postgres::GenericClient}
            }
            DbCrate::DeadPoolPostgres => {
                quote! {&impl deadpool_postgres::GenericClient}
            }
        }
    }

    /// Returns the error type tokens for the specific database crate
    pub(crate) fn error_ident(&self) -> TokenStream {
        match self {
            DbCrate::TokioPostgres => {
                quote! {tokio_postgres::Error}
            }
            DbCrate::Postgres => {
                quote! {postgres::Error}
            }
            DbCrate::DeadPoolPostgres => {
                // deadpool_postgres use tokio_postgres::Error
                // https://docs.rs/deadpool-postgres/latest/deadpool_postgres/trait.GenericClient.html
                quote! {deadpool_postgres::tokio_postgres::Error}
            }
        }
    }

    /// Returns async keyword or empty based on database crate async support
    pub(crate) fn async_ident(&self) -> TokenStream {
        match self {
            Self::TokioPostgres | Self::DeadPoolPostgres => {
                quote! {async}
            }
            Self::Postgres => {
                quote! {}
            }
        }
    }

    /// Returns .await or empty based on database crate async support
    pub(crate) fn await_ident(&self) -> TokenStream {
        match self {
            Self::TokioPostgres | Self::DeadPoolPostgres => {
                quote! {.await}
            }
            Self::Postgres => {
                quote! {}
            }
        }
    }
}