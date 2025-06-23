pub mod queries;

#[cfg(test)]
mod tests {
    use crate::queries::*;
    use test_context::test_context;
    use test_utils::PgTokioTestContext;
    use tokio_postgres::NoTls;

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn test_complex_queries_compile(_ctx: &PgTokioTestContext) {
        // This test just ensures the generated code compiles
        // In a real scenario, you would test against a populated database

        // Test 1: Multiple table JOIN
        let client = tokio_postgres::connect("", NoTls).await;
        if let Ok((client, _)) = client {
            let _result = get_book_with_author_and_categories(&client, Some(&2020)).await;
        }

        // Test 2: Self-join
        let client = tokio_postgres::connect("", NoTls).await;
        if let Ok((client, _)) = client {
            let _result = get_employees_with_managers(&client).await;
        }

        // Test 3: Subquery (single table)
        let client = tokio_postgres::connect("", NoTls).await;
        if let Ok((client, _)) = client {
            let _result = get_top_rated_books(&client, Some(&4)).await;
        }

        // This test passes if compilation succeeds
        assert!(true);
    }
}
