#[allow(warnings)]
pub(crate) mod queries;
#[cfg(test)]
mod tests {
    use crate::queries;
    use test_context::test_context;
    use test_utils::PgTokioTestContext;

    async fn migrate_db(client: &tokio_postgres::Client) {
        client
            .batch_execute(include_str!("./schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn queries_works(ctx: &mut PgTokioTestContext) {
        migrate_db(&ctx.client).await;
        let author = queries::create_author(&ctx.client, "Bob")
            .await
            .unwrap()
            .unwrap();
        let get_author = queries::get_author(&ctx.client, author.author_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(get_author.name, "Bob");

        let hello = queries::say_hello(&ctx.client, "world")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(hello.say_hello.unwrap(), "hello world")
    }
}
