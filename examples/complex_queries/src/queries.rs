//! Code generated by sqlc. SHOULD NOT EDIT.
//! sqlc version: v1.28.0
//! sqlc-rust-postgres version: v0.1.4
pub const GET_BOOK_WITH_AUTHOR_AND_CATEGORIES: &str = r#"-- name: GetBookWithAuthorAndCategories :many
SELECT 
    b.id,
    b.title,
    b.published_year,
    a.id,
    a.name,
    a.birth_year,
    c.id,
    c.name,
    c.description
FROM books b
JOIN authors a ON b.author_id = a.id
JOIN book_categories bc ON b.id = bc.book_id
JOIN categories c ON bc.category_id = c.id
WHERE b.published_year > $1"#;
#[derive(Debug, Clone)]
pub struct GetBookWithAuthorAndCategoriesRow {
    pub books_id: i32,
    pub title: String,
    pub published_year: Option<i32>,
    pub authors_id: i32,
    pub authors_name: String,
    pub birth_year: Option<i32>,
    pub categories_id: i32,
    pub categories_name: String,
    pub description: Option<String>,
}
impl GetBookWithAuthorAndCategoriesRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(GetBookWithAuthorAndCategoriesRow {
            books_id: row.try_get(0)?,
            title: row.try_get(1)?,
            published_year: row.try_get(2)?,
            authors_id: row.try_get(3)?,
            authors_name: row.try_get(4)?,
            birth_year: row.try_get(5)?,
            categories_id: row.try_get(6)?,
            categories_name: row.try_get(7)?,
            description: row.try_get(8)?,
        })
    }
}
pub async fn get_book_with_author_and_categories(
    client: &impl tokio_postgres::GenericClient,
    published_year: Option<i32>,
) -> Result<
    impl Iterator<Item = Result<GetBookWithAuthorAndCategoriesRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client
        .query(GET_BOOK_WITH_AUTHOR_AND_CATEGORIES, &[&published_year])
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| GetBookWithAuthorAndCategoriesRow::from_row(&r)))
}
#[derive(Debug)]
pub struct GetBookWithAuthorAndCategories {
    pub published_year: Option<i32>,
}
impl GetBookWithAuthorAndCategories {
    pub const QUERY: &'static str = r#"-- name: GetBookWithAuthorAndCategories :many
SELECT 
    b.id,
    b.title,
    b.published_year,
    a.id,
    a.name,
    a.birth_year,
    c.id,
    c.name,
    c.description
FROM books b
JOIN authors a ON b.author_id = a.id
JOIN book_categories bc ON b.id = bc.book_id
JOIN categories c ON bc.category_id = c.id
WHERE b.published_year > $1"#;
}
impl GetBookWithAuthorAndCategories {
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<GetBookWithAuthorAndCategoriesRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.published_year]).await?;
        rows.into_iter()
            .map(|r| GetBookWithAuthorAndCategoriesRow::from_row(&r))
            .collect()
    }
    pub async fn query_raw(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<
        impl Iterator<Item = Result<GetBookWithAuthorAndCategoriesRow, tokio_postgres::Error>>,
        tokio_postgres::Error,
    > {
        let rows = client.query(Self::QUERY, &[&self.published_year]).await?;
        Ok(rows
            .into_iter()
            .map(|r| GetBookWithAuthorAndCategoriesRow::from_row(&r)))
    }
}
#[derive(Debug)]
pub struct GetBookWithAuthorAndCategoriesBuilder<Fields = ()> {
    fields: Fields,
    phantom: std::marker::PhantomData<()>,
}
impl GetBookWithAuthorAndCategories {
    pub fn builder() -> GetBookWithAuthorAndCategoriesBuilder<()> {
        GetBookWithAuthorAndCategoriesBuilder {
            fields: (),
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetBookWithAuthorAndCategoriesBuilder<()> {
    pub fn published_year(
        self,
        published_year: Option<i32>,
    ) -> GetBookWithAuthorAndCategoriesBuilder<Option<i32>> {
        let () = self.fields;
        GetBookWithAuthorAndCategoriesBuilder {
            fields: published_year,
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetBookWithAuthorAndCategoriesBuilder<Option<i32>> {
    pub fn build(self) -> GetBookWithAuthorAndCategories {
        let published_year = self.fields;
        GetBookWithAuthorAndCategories { published_year }
    }
}
pub const GET_EMPLOYEES_WITH_MANAGERS: &str = r#"-- name: GetEmployeesWithManagers :many
SELECT 
    e.id,
    e.name,
    e.department,
    e.salary,
    m.id,
    m.name,
    m.department
FROM employees e
LEFT JOIN employees m ON e.manager_id = m.id"#;
#[derive(Debug, Clone)]
pub struct GetEmployeesWithManagersRow {
    pub employees_id_1: i32,
    pub employees_name_1: String,
    pub employees_department_1: Option<String>,
    pub salary: Option<i32>,
    pub employees_id_2: Option<i32>,
    pub employees_name_2: Option<String>,
    pub employees_department_2: Option<String>,
}
impl GetEmployeesWithManagersRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(GetEmployeesWithManagersRow {
            employees_id_1: row.try_get(0)?,
            employees_name_1: row.try_get(1)?,
            employees_department_1: row.try_get(2)?,
            salary: row.try_get(3)?,
            employees_id_2: row.try_get(4)?,
            employees_name_2: row.try_get(5)?,
            employees_department_2: row.try_get(6)?,
        })
    }
}
pub async fn get_employees_with_managers(
    client: &impl tokio_postgres::GenericClient,
) -> Result<
    impl Iterator<Item = Result<GetEmployeesWithManagersRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client.query(GET_EMPLOYEES_WITH_MANAGERS, &[]).await?;
    Ok(rows
        .into_iter()
        .map(|r| GetEmployeesWithManagersRow::from_row(&r)))
}
pub const GET_TOP_RATED_BOOKS: &str = r#"-- name: GetTopRatedBooks :many
SELECT id, title, published_year
FROM books
WHERE id IN (
    SELECT book_id 
    FROM reviews 
    WHERE rating >= $1
)"#;
#[derive(Debug, Clone)]
pub struct GetTopRatedBooksRow {
    pub id: i32,
    pub title: String,
    pub published_year: Option<i32>,
}
impl GetTopRatedBooksRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(GetTopRatedBooksRow {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            published_year: row.try_get(2)?,
        })
    }
}
pub async fn get_top_rated_books(
    client: &impl tokio_postgres::GenericClient,
    rating: Option<i32>,
) -> Result<
    impl Iterator<Item = Result<GetTopRatedBooksRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client.query(GET_TOP_RATED_BOOKS, &[&rating]).await?;
    Ok(rows.into_iter().map(|r| GetTopRatedBooksRow::from_row(&r)))
}
#[derive(Debug)]
pub struct GetTopRatedBooks {
    pub rating: Option<i32>,
}
impl GetTopRatedBooks {
    pub const QUERY: &'static str = r#"-- name: GetTopRatedBooks :many
SELECT id, title, published_year
FROM books
WHERE id IN (
    SELECT book_id 
    FROM reviews 
    WHERE rating >= $1
)"#;
}
impl GetTopRatedBooks {
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<GetTopRatedBooksRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.rating]).await?;
        rows.into_iter()
            .map(|r| GetTopRatedBooksRow::from_row(&r))
            .collect()
    }
    pub async fn query_raw(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<
        impl Iterator<Item = Result<GetTopRatedBooksRow, tokio_postgres::Error>>,
        tokio_postgres::Error,
    > {
        let rows = client.query(Self::QUERY, &[&self.rating]).await?;
        Ok(rows.into_iter().map(|r| GetTopRatedBooksRow::from_row(&r)))
    }
}
#[derive(Debug)]
pub struct GetTopRatedBooksBuilder<Fields = ()> {
    fields: Fields,
    phantom: std::marker::PhantomData<()>,
}
impl GetTopRatedBooks {
    pub fn builder() -> GetTopRatedBooksBuilder<()> {
        GetTopRatedBooksBuilder {
            fields: (),
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetTopRatedBooksBuilder<()> {
    pub fn rating(self, rating: Option<i32>) -> GetTopRatedBooksBuilder<Option<i32>> {
        let () = self.fields;
        GetTopRatedBooksBuilder {
            fields: rating,
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetTopRatedBooksBuilder<Option<i32>> {
    pub fn build(self) -> GetTopRatedBooks {
        let rating = self.fields;
        GetTopRatedBooks { rating }
    }
}
pub const GET_AUTHOR_BOOK_STATS: &str = r#"-- name: GetAuthorBookStats :many
SELECT 
    a.id,
    a.name,
    COUNT(DISTINCT b.id) as book_count,
    AVG(r.rating) as avg_rating,
    b.id,
    b.title
FROM authors a
LEFT JOIN books b ON a.id = b.author_id
LEFT JOIN reviews r ON b.id = r.book_id
WHERE b.id IS NOT NULL
GROUP BY a.id, a.name, b.id, b.title
HAVING COUNT(DISTINCT b.id) > $1"#;
#[derive(Debug, Clone)]
pub struct GetAuthorBookStatsRow {
    pub authors_id: i32,
    pub name: String,
    pub book_count: i64,
    pub avg_rating: f64,
    pub books_id: Option<i32>,
    pub title: Option<String>,
}
impl GetAuthorBookStatsRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(GetAuthorBookStatsRow {
            authors_id: row.try_get(0)?,
            name: row.try_get(1)?,
            book_count: row.try_get(2)?,
            avg_rating: row.try_get(3)?,
            books_id: row.try_get(4)?,
            title: row.try_get(5)?,
        })
    }
}
pub async fn get_author_book_stats(
    client: &impl tokio_postgres::GenericClient,
    id: i32,
) -> Result<
    impl Iterator<Item = Result<GetAuthorBookStatsRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client.query(GET_AUTHOR_BOOK_STATS, &[&id]).await?;
    Ok(rows
        .into_iter()
        .map(|r| GetAuthorBookStatsRow::from_row(&r)))
}
#[derive(Debug)]
pub struct GetAuthorBookStats {
    pub id: i32,
}
impl GetAuthorBookStats {
    pub const QUERY: &'static str = r#"-- name: GetAuthorBookStats :many
SELECT 
    a.id,
    a.name,
    COUNT(DISTINCT b.id) as book_count,
    AVG(r.rating) as avg_rating,
    b.id,
    b.title
FROM authors a
LEFT JOIN books b ON a.id = b.author_id
LEFT JOIN reviews r ON b.id = r.book_id
WHERE b.id IS NOT NULL
GROUP BY a.id, a.name, b.id, b.title
HAVING COUNT(DISTINCT b.id) > $1"#;
}
impl GetAuthorBookStats {
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<GetAuthorBookStatsRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.id]).await?;
        rows.into_iter()
            .map(|r| GetAuthorBookStatsRow::from_row(&r))
            .collect()
    }
    pub async fn query_raw(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<
        impl Iterator<Item = Result<GetAuthorBookStatsRow, tokio_postgres::Error>>,
        tokio_postgres::Error,
    > {
        let rows = client.query(Self::QUERY, &[&self.id]).await?;
        Ok(rows
            .into_iter()
            .map(|r| GetAuthorBookStatsRow::from_row(&r)))
    }
}
#[derive(Debug)]
pub struct GetAuthorBookStatsBuilder<Fields = ()> {
    fields: Fields,
    phantom: std::marker::PhantomData<()>,
}
impl GetAuthorBookStats {
    pub fn builder() -> GetAuthorBookStatsBuilder<()> {
        GetAuthorBookStatsBuilder {
            fields: (),
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetAuthorBookStatsBuilder<()> {
    pub fn id(self, id: i32) -> GetAuthorBookStatsBuilder<i32> {
        let () = self.fields;
        GetAuthorBookStatsBuilder {
            fields: id,
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetAuthorBookStatsBuilder<i32> {
    pub fn build(self) -> GetAuthorBookStats {
        let id = self.fields;
        GetAuthorBookStats { id }
    }
}
pub const COMPARE_BOOK_YEARS: &str = r#"-- name: CompareBookYears :many
SELECT 
    old_books.id,
    old_books.title,
    old_books.published_year,
    new_books.id,
    new_books.title,
    new_books.published_year
FROM books old_books
CROSS JOIN books new_books
WHERE old_books.published_year < $1 
  AND new_books.published_year > $2
  AND old_books.id != new_books.id
LIMIT 10"#;
#[derive(Debug, Clone)]
pub struct CompareBookYearsRow {
    pub books_id_1: i32,
    pub books_title_1: String,
    pub books_published_year_1: Option<i32>,
    pub books_id_2: i32,
    pub books_title_2: String,
    pub books_published_year_2: Option<i32>,
}
impl CompareBookYearsRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(CompareBookYearsRow {
            books_id_1: row.try_get(0)?,
            books_title_1: row.try_get(1)?,
            books_published_year_1: row.try_get(2)?,
            books_id_2: row.try_get(3)?,
            books_title_2: row.try_get(4)?,
            books_published_year_2: row.try_get(5)?,
        })
    }
}
pub async fn compare_book_years(
    client: &impl tokio_postgres::GenericClient,
    published_year_1: Option<i32>,
    published_year_2: Option<i32>,
) -> Result<
    impl Iterator<Item = Result<CompareBookYearsRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client
        .query(COMPARE_BOOK_YEARS, &[&published_year_1, &published_year_2])
        .await?;
    Ok(rows.into_iter().map(|r| CompareBookYearsRow::from_row(&r)))
}
#[derive(Debug)]
pub struct CompareBookYears {
    pub published_year_1: Option<i32>,
    pub published_year_2: Option<i32>,
}
impl CompareBookYears {
    pub const QUERY: &'static str = r#"-- name: CompareBookYears :many
SELECT 
    old_books.id,
    old_books.title,
    old_books.published_year,
    new_books.id,
    new_books.title,
    new_books.published_year
FROM books old_books
CROSS JOIN books new_books
WHERE old_books.published_year < $1 
  AND new_books.published_year > $2
  AND old_books.id != new_books.id
LIMIT 10"#;
}
impl CompareBookYears {
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<CompareBookYearsRow>, tokio_postgres::Error> {
        let rows = client
            .query(
                Self::QUERY,
                &[&self.published_year_1, &self.published_year_2],
            )
            .await?;
        rows.into_iter()
            .map(|r| CompareBookYearsRow::from_row(&r))
            .collect()
    }
    pub async fn query_raw(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<
        impl Iterator<Item = Result<CompareBookYearsRow, tokio_postgres::Error>>,
        tokio_postgres::Error,
    > {
        let rows = client
            .query(
                Self::QUERY,
                &[&self.published_year_1, &self.published_year_2],
            )
            .await?;
        Ok(rows.into_iter().map(|r| CompareBookYearsRow::from_row(&r)))
    }
}
#[derive(Debug)]
pub struct CompareBookYearsBuilder<Fields = ((), ())> {
    fields: Fields,
    phantom: std::marker::PhantomData<()>,
}
impl CompareBookYears {
    pub fn builder() -> CompareBookYearsBuilder<((), ())> {
        CompareBookYearsBuilder {
            fields: ((), ()),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<V1> CompareBookYearsBuilder<((), V1)> {
    pub fn published_year_1(
        self,
        published_year_1: Option<i32>,
    ) -> CompareBookYearsBuilder<(Option<i32>, V1)> {
        let ((), v1) = self.fields;
        CompareBookYearsBuilder {
            fields: (published_year_1, v1),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<V0> CompareBookYearsBuilder<(V0, ())> {
    pub fn published_year_2(
        self,
        published_year_2: Option<i32>,
    ) -> CompareBookYearsBuilder<(V0, Option<i32>)> {
        let (v0, ()) = self.fields;
        CompareBookYearsBuilder {
            fields: (v0, published_year_2),
            phantom: std::marker::PhantomData,
        }
    }
}
impl CompareBookYearsBuilder<(Option<i32>, Option<i32>)> {
    pub fn build(self) -> CompareBookYears {
        let (published_year_1, published_year_2) = self.fields;
        CompareBookYears {
            published_year_1,
            published_year_2,
        }
    }
}
pub const GET_BOOKS_WITH_ALIASES: &str = r#"-- name: GetBooksWithAliases :many
SELECT 
    id as book_id,
    title as book_title,
    published_year as year
FROM books
WHERE published_year BETWEEN $1 AND $2"#;
#[derive(Debug, Clone)]
pub struct GetBooksWithAliasesRow {
    pub book_id: i32,
    pub book_title: String,
    pub year: Option<i32>,
}
impl GetBooksWithAliasesRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(GetBooksWithAliasesRow {
            book_id: row.try_get(0)?,
            book_title: row.try_get(1)?,
            year: row.try_get(2)?,
        })
    }
}
pub async fn get_books_with_aliases(
    client: &impl tokio_postgres::GenericClient,
    published_year_1: Option<i32>,
    published_year_2: Option<i32>,
) -> Result<
    impl Iterator<Item = Result<GetBooksWithAliasesRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client
        .query(
            GET_BOOKS_WITH_ALIASES,
            &[&published_year_1, &published_year_2],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| GetBooksWithAliasesRow::from_row(&r)))
}
#[derive(Debug)]
pub struct GetBooksWithAliases {
    pub published_year_1: Option<i32>,
    pub published_year_2: Option<i32>,
}
impl GetBooksWithAliases {
    pub const QUERY: &'static str = r#"-- name: GetBooksWithAliases :many
SELECT 
    id as book_id,
    title as book_title,
    published_year as year
FROM books
WHERE published_year BETWEEN $1 AND $2"#;
}
impl GetBooksWithAliases {
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<GetBooksWithAliasesRow>, tokio_postgres::Error> {
        let rows = client
            .query(
                Self::QUERY,
                &[&self.published_year_1, &self.published_year_2],
            )
            .await?;
        rows.into_iter()
            .map(|r| GetBooksWithAliasesRow::from_row(&r))
            .collect()
    }
    pub async fn query_raw(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<
        impl Iterator<Item = Result<GetBooksWithAliasesRow, tokio_postgres::Error>>,
        tokio_postgres::Error,
    > {
        let rows = client
            .query(
                Self::QUERY,
                &[&self.published_year_1, &self.published_year_2],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| GetBooksWithAliasesRow::from_row(&r)))
    }
}
#[derive(Debug)]
pub struct GetBooksWithAliasesBuilder<Fields = ((), ())> {
    fields: Fields,
    phantom: std::marker::PhantomData<()>,
}
impl GetBooksWithAliases {
    pub fn builder() -> GetBooksWithAliasesBuilder<((), ())> {
        GetBooksWithAliasesBuilder {
            fields: ((), ()),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<V1> GetBooksWithAliasesBuilder<((), V1)> {
    pub fn published_year_1(
        self,
        published_year_1: Option<i32>,
    ) -> GetBooksWithAliasesBuilder<(Option<i32>, V1)> {
        let ((), v1) = self.fields;
        GetBooksWithAliasesBuilder {
            fields: (published_year_1, v1),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<V0> GetBooksWithAliasesBuilder<(V0, ())> {
    pub fn published_year_2(
        self,
        published_year_2: Option<i32>,
    ) -> GetBooksWithAliasesBuilder<(V0, Option<i32>)> {
        let (v0, ()) = self.fields;
        GetBooksWithAliasesBuilder {
            fields: (v0, published_year_2),
            phantom: std::marker::PhantomData,
        }
    }
}
impl GetBooksWithAliasesBuilder<(Option<i32>, Option<i32>)> {
    pub fn build(self) -> GetBooksWithAliases {
        let (published_year_1, published_year_2) = self.fields;
        GetBooksWithAliases {
            published_year_1,
            published_year_2,
        }
    }
}
pub const GET_CATEGORY_STATS: &str = r#"-- name: GetCategoryStats :many
SELECT 
    c.id,
    c.name,
    COUNT(DISTINCT b.id) as book_count,
    COUNT(DISTINCT a.id) as author_count,
    AVG(r.rating) as avg_rating
FROM categories c
LEFT JOIN book_categories bc ON c.id = bc.category_id
LEFT JOIN books b ON bc.book_id = b.id
LEFT JOIN authors a ON b.author_id = a.id
LEFT JOIN reviews r ON b.id = r.book_id
GROUP BY c.id, c.name
HAVING COUNT(DISTINCT b.id) > 0"#;
#[derive(Debug, Clone)]
pub struct GetCategoryStatsRow {
    pub id: i32,
    pub name: String,
    pub book_count: i64,
    pub author_count: i64,
    pub avg_rating: f64,
}
impl GetCategoryStatsRow {
    pub(crate) fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(GetCategoryStatsRow {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            book_count: row.try_get(2)?,
            author_count: row.try_get(3)?,
            avg_rating: row.try_get(4)?,
        })
    }
}
pub async fn get_category_stats(
    client: &impl tokio_postgres::GenericClient,
) -> Result<
    impl Iterator<Item = Result<GetCategoryStatsRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client.query(GET_CATEGORY_STATS, &[]).await?;
    Ok(rows.into_iter().map(|r| GetCategoryStatsRow::from_row(&r)))
}
