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
    #[test]
    #[ignore]
    fn test_parse() {
        let s = r#"
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
}
