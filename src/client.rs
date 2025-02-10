//! Copy from cornucopia  
//!
//! Original Copyright (c) [2022] [cornucopia-rs/cornucopia]
//! Licensed under the [MIT/Apache-2.0] License.
//!
//! MIT: https://github.com/cornucopia-rs/cornucopia/blob/d1229ae6948e691c40e851377b9e5a410305ec4f/LICENSE-MIT
//! Apache-2.0: https://github.com/cornucopia-rs/cornucopia/blob/d1229ae6948e691c40e851377b9e5a410305ec4f/LICENSE-APACHE

pub trait GenericClient: Send + Sync {
    async fn prepare(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error>;
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
    async fn prepare(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error> {
        tokio_postgres::Client::prepare(self, query).await
    }

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
    async fn prepare(
        &self,
        query: &str,
    ) -> Result<tokio_postgres::Statement, tokio_postgres::Error> {
        tokio_postgres::Transaction::prepare(self, query).await
    }

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
