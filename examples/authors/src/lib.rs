#[allow(warnings)]
pub(crate) mod queries;

#[cfg(test)]
mod tests {
    use crate::queries;
    use test_context::test_context;
    use test_utils::PgTokioTestContext;

    async fn migrate_db(clinet: &tokio_postgres::Client) {
        clinet
            .batch_execute(include_str!("./schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn queries_works(ctx: &mut PgTokioTestContext) {
        migrate_db(&ctx.client).await;
        let res = queries::create_author(&ctx.client, "FOO", Some("BAR"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(res.name, "FOO");
        assert_eq!(res.bio.as_ref().unwrap(), "BAR");

        let author = queries::get_author(&ctx.client, &res.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(res.name, author.name);
        assert_eq!(res.bio, author.bio);

        let authors_list = queries::list_authors(&ctx.client)
            .await
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(authors_list.len(), 1);

        queries::delete_author(&ctx.client, &author.id)
            .await
            .unwrap();

        let authors_list = queries::list_authors(&ctx.client)
            .await
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(authors_list.len(), 0);
    }
}
