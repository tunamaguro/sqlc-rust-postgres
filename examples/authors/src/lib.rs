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
            .unwrap();
        assert_eq!(res.authors_name, "FOO");
        assert_eq!(res.authors_bio.as_ref().unwrap(), "BAR");

        let author = queries::get_author(&ctx.client, &res.authors_id)
            .await
            .unwrap();

        assert_eq!(res.authors_name, author.authors_name);
        assert_eq!(res.authors_bio, author.authors_bio);

        let authors_list = queries::list_authors(&ctx.client)
            .await
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(authors_list.len(), 1);

        queries::delete_author(&ctx.client, &author.authors_id)
            .await
            .unwrap();

        let authors_list = queries::list_authors(&ctx.client)
            .await
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(authors_list.len(), 0);
    }
}
