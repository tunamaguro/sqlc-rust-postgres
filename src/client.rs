pub trait GenericClient: Send + Sync {
    async fn execute<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send;
    async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send;
    async fn query_opt<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send;
    async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send;

    async fn query_raw<T, P, I>(
        &self,
        statement: &T,
        params: I,
    ) -> Result<tokio_postgres::RowStream, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
        P: tokio_postgres::types::BorrowToSql,
        I: IntoIterator<Item = P> + Sync + Send,
        I::IntoIter: ExactSizeIterator;
}

impl GenericClient for tokio_postgres::Client {
    async fn execute<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Client::execute(self, statement, params).await
    }

    async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Client::query_one(self, statement, params).await
    }

    async fn query_opt<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Client::query_opt(self, statement, params).await
    }

    async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Client::query(self, statement, params).await
    }

    async fn query_raw<T, P, I>(
        &self,
        statement: &T,
        params: I,
    ) -> Result<tokio_postgres::RowStream, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
        P: tokio_postgres::types::BorrowToSql,
        I: IntoIterator<Item = P> + Sync + Send,
        I::IntoIter: ExactSizeIterator,
    {
        tokio_postgres::Client::query_raw(self, statement, params).await
    }
}

impl GenericClient for tokio_postgres::Transaction<'_> {
    async fn execute<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Transaction::execute(self, statement, params).await
    }

    async fn query_one<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Transaction::query_one(self, statement, params).await
    }

    async fn query_opt<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Transaction::query_opt(self, statement, params).await
    }

    async fn query<T>(
        &self,
        statement: &T,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
    {
        tokio_postgres::Transaction::query(self, statement, params).await
    }

    async fn query_raw<T, P, I>(
        &self,
        statement: &T,
        params: I,
    ) -> Result<tokio_postgres::RowStream, tokio_postgres::Error>
    where
        T: ?Sized + tokio_postgres::ToStatement + Sync + Send,
        P: tokio_postgres::types::BorrowToSql,
        I: IntoIterator<Item = P> + Sync + Send,
        I::IntoIter: ExactSizeIterator,
    {
        tokio_postgres::Transaction::query_raw(self, statement, params).await
    }
}
