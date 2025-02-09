pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

mod codegen;
pub(crate) mod query;
pub(crate) mod sqlc_annotation;
pub(crate) mod user_type;

pub use codegen::*;

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;

    use crate::deserialize_codegen_request;
    #[test]
    #[ignore]
    fn test_parse() {
        let s = r#"
           pub struct GetAuthorByIdParams<'a> {
    author_id: Option<&'a i32>,
}
    pub struct SomeQueryParams<'a> {
    tags: &'a [String],
}
        "#;
        let tt: TokenStream = s.parse().unwrap();
        dbg!(tt);
    }

    #[test]
    #[ignore]
    fn test_input() {
        let f = include_bytes!("../gen/input.bin");
        let req = deserialize_codegen_request(f.as_slice()).unwrap();
        let catalog = req.catalog.as_ref().unwrap();
        dbg!(req
            .queries
            .iter()
            .flat_map(|q| q.params.as_slice())
            .map(|p| p.column.as_ref())
            .collect::<Vec<_>>());
        dbg!(&catalog
            .schemas
            .iter()
            .flat_map(|s| s.tables.as_slice())
            .flat_map(|table| table.columns.as_slice())
            .map(|col| col.r#type.as_ref())
            .take(40)
            .collect::<Vec<_>>());
    }
}
