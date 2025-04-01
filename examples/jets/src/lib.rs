#[allow(warnings)]
pub(crate) mod queries;

#[cfg(test)]
mod tests {

    use test_context::test_context;
    use test_utils::PgSyncTestContext;

    use crate::queries;

    fn migrate_db(clinet: &mut postgres::Client) {
        clinet.batch_execute(include_str!("./schema.sql")).unwrap();
    }

    #[test_context(PgSyncTestContext)]
    #[test]
    fn queries_works(ctx: &mut PgSyncTestContext) {
        migrate_db(&mut ctx.client);

        let count = queries::count_pilots(&mut ctx.client).unwrap().unwrap();
        assert_eq!(count.count, 0)
    }
}
