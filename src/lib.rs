pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

mod codegen;
pub mod error;

pub(crate) mod utils;
pub use error::Error;
pub(crate) mod query;
pub(crate) mod sqlc;
pub(crate) mod user_type;
pub(crate) type Result<T> = std::result::Result<T, error::Error>;

pub use codegen::*;

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;

    use crate::deserialize_codegen_request;
    #[test]
    // #[ignore]
    fn test_parse() {
        let s = r#"
//! Code generated by sqlc. DO NOT EDIT.
        "#;
        let a = syn::parse_str::<proc_macro2::Literal>(r##"r#"SELECT * FROM TABLE;"#"##).unwrap();
        dbg!(a);
        let tt: TokenStream = s.parse().unwrap();
        dbg!(tt);
    }

    #[test]
    #[ignore]
    fn test_input() {
        let f = std::fs::read("./gen/input.bin").unwrap();
        let req = deserialize_codegen_request(&f).unwrap();
        let catalog = req.catalog.as_ref().unwrap();
        dbg!(
            req.queries
                .iter()
                .flat_map(|q| q.params.as_slice())
                .map(|p| p.column.as_ref())
                .collect::<Vec<_>>()
        );
        dbg!(
            &catalog
                .schemas
                .iter()
                .flat_map(|s| s.tables.as_slice())
                .flat_map(|table| table.columns.as_slice())
                .map(|col| col.r#type.as_ref())
                .take(40)
                .collect::<Vec<_>>()
        );
    }
}
