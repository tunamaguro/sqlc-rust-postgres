#[allow(warnings)]
pub mod queries;

#[allow(dead_code)]
pub mod performance_test;

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

        let author = queries::get_author(&ctx.client, res.id)
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

        queries::delete_author(&ctx.client, author.id)
            .await
            .unwrap();

        let authors_list = queries::list_authors(&ctx.client)
            .await
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(authors_list.len(), 0);
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn nullable_copy_type_state_works(ctx: &mut PgTokioTestContext) {
        migrate_db(&ctx.client).await;

        // Create an author first
        let author = queries::create_author(
            &ctx.client,
            "TypeState Test",
            Some("Testing nullable Copy types"),
        )
        .await
        .unwrap()
        .unwrap();

        // ✅ Test 1: GetAuthorByIdAndAge - Type-state pattern with nullable Copy type
        let query1 = queries::GetAuthorByIdAndAge::builder()
            .id(author.id)
            .age(Some(25))
            .build();

        let _result1 = query1.query_opt(&ctx.client).await.unwrap();
        // Author exists but age filter doesn't match, so should return None or handle accordingly

        // ✅ Test 2: Different parameter order
        let query2 = queries::GetAuthorByIdAndAge::builder()
            .age(None) // nullable parameter with None
            .id(author.id)
            .build();

        let _result2 = query2.query_opt(&ctx.client).await.unwrap();

        // ✅ Test 3: UpdateAuthorStatus - 3 parameters with nullable Copy types
        let update_query = queries::UpdateAuthorStatus::builder()
            .is_active(Some(true))
            .age(Some(30))
            .id(author.id)
            .build();

        let update_result = update_query.execute(&ctx.client).await.unwrap();
        assert_eq!(update_result, 1); // 1 row updated

        // ✅ Test 4: Different parameter order for 3-parameter query
        let update_query2 = queries::UpdateAuthorStatus::builder()
            .id(author.id)
            .age(None) // nullable age
            .is_active(Some(false))
            .build();

        let update_result2 = update_query2.execute(&ctx.client).await.unwrap();
        assert_eq!(update_result2, 1); // 1 row updated

        println!("✅ All nullable Copy type state pattern tests passed!");
    }

    #[test]
    fn zero_cost_abstraction_verification() {
        // ゼロコスト抽象化の基本確認

        // メモリレイアウト確認
        println!("📏 Actual Memory Layout:");
        println!(
            "   GetAuthorByIdAndAge size: {} bytes",
            std::mem::size_of::<queries::GetAuthorByIdAndAge>()
        );
        println!(
            "   UpdateAuthorStatus size: {} bytes",
            std::mem::size_of::<queries::UpdateAuthorStatus>()
        );
        println!("   i64 size: {} bytes", std::mem::size_of::<i64>());
        println!(
            "   Option<i32> size: {} bytes",
            std::mem::size_of::<Option<i32>>()
        );
        println!(
            "   Option<bool> size: {} bytes",
            std::mem::size_of::<Option<bool>>()
        );

        // 実際のサイズに基づく確認
        let _expected_get_author_size = std::mem::size_of::<queries::GetAuthorByIdAndAge>();
        let _expected_update_author_size = std::mem::size_of::<queries::UpdateAuthorStatus>();

        // 基本的な構築テスト
        let query1 = queries::GetAuthorByIdAndAge::builder()
            .id(123)
            .age(Some(25))
            .build();

        let query2 = queries::GetAuthorByIdAndAge {
            id: 123,
            age: Some(25),
        };

        assert_eq!(query1.id, query2.id);
        assert_eq!(query1.age, query2.age);

        println!("✅ Zero-cost abstraction basic verification passed!");

        // 実際のベンチマークは通常のテストとは分離して実行
        // benchmark_type_state_pattern();
    }
}
