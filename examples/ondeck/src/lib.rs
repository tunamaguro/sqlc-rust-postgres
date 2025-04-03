#[allow(warnings)]
pub(crate) mod queries;

#[cfg(test)]
mod tests {
    use crate::queries;
    use test_context::test_context;
    use test_utils::DeadPoolContext;

    async fn migrate_db(client: &mut deadpool_postgres::Client) {
        let tx = client.transaction().await.unwrap();
        tx.batch_execute(include_str!("../schema/0001_city.sql"))
            .await
            .unwrap();
        tx.batch_execute(include_str!("../schema/0002_venue.sql"))
            .await
            .unwrap();
        tx.batch_execute(include_str!("../schema/0003_add_columns.sql"))
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    #[test_context(DeadPoolContext)]
    #[tokio::test]
    async fn queries_works(ctx: &mut DeadPoolContext) {
        let mut client = ctx.pool.get().await.unwrap();
        migrate_db(&mut client).await;

        let city = queries::create_city(&client, "San Francisco", "san-francisco")
            .await
            .unwrap()
            .unwrap();

        let venue = queries::create_venue(
            &client,
            "the-fillmore",
            "The Fillmore",
            &city.city_slug,
            "spotify:uro",
            &queries::Status::Open,
            Some(&[queries::Status::Open, queries::Status::Closed]),
            Some(&["rock".to_string(), "punk".to_string()]),
        )
        .await
        .unwrap()
        .unwrap();

        let get_venue = queries::get_venue(&client, "the-fillmore", &city.city_slug)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(get_venue.venue_id, venue.venue_id);
    }
}
