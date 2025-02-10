pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

#[cfg(test)]
pub(crate) mod client;
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
    // #[ignore]
    fn test_parse() {
        let s = r#"
#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "mood")]
enum Mood {
    #[postgres(name = "sad")]
    Sad,
    #[postgres(name = "ok")]
    Ok,
    #[postgres(name = "happy")]
    Happy,
}
        "#;
        let tt: TokenStream = s.parse().unwrap();
        dbg!(tt);
    }

    #[test]
    #[ignore]
    fn test_input() {
        let f = std::fs::read("./gen/input.bin").unwrap();
        let req = deserialize_codegen_request(&f).unwrap();
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

    fn a(
        rows: Vec<tokio_postgres::Row>,
    ) -> Result<impl Iterator<Item = Result<(), tokio_postgres::Error>>, tokio_postgres::Error>
    {
        Ok(rows.into_iter().map(|r| {
            let _: i32 = r.try_get(0)?;
            Ok(())
        }))
    }
}
