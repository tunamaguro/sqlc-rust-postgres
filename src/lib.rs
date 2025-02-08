pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

mod codegen;
pub(crate) mod query;
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
            /// Long comment
            #[derive(Debug)]
            enum Foo{
                A,
                B,
                C
            };

            let a: Foo = Foo::A;
        "#;
        let tt: TokenStream = s.parse().unwrap();
        dbg!(tt);
    }

    #[test]
    // #[ignore]
    fn test_input() {
        let f = std::fs::read("./gen/input.bin").unwrap();
        let req = deserialize_codegen_request(&f).unwrap();
        let catalog = req.catalog.as_ref().unwrap();
        dbg!(&catalog
            .schemas
            .iter()
            .flat_map(|s| s.tables.clone())
            .take(3)
            .collect::<Vec<_>>());
    }
}
