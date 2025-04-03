//! Code generated by sqlc. SHOULD NOT EDIT.
//! sqlc version: v1.28.0
//! sqlc-rust-postgres version: v0.1.3
#[derive(Debug, Clone, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "book_type")]
pub enum BookType {
    #[postgres(name = "FICTION")]
    Fiction,
    #[postgres(name = "NONFICTION")]
    Nonfiction,
}
pub const GET_AUTHOR: &str = r#"-- name: GetAuthor :one
SELECT author_id, name FROM authors
WHERE author_id = $1"#;
#[derive(Debug, Clone)]
pub struct GetAuthorRow {
    pub authors_author_id: i32,
    pub authors_name: String,
}
pub async fn get_author(
    client: &impl tokio_postgres::GenericClient,
    authors_author_id: &i32,
) -> Result<Option<GetAuthorRow>, tokio_postgres::Error> {
    let row = client.query_opt(GET_AUTHOR, &[&authors_author_id]).await?;
    let v = match row {
        Some(v) => GetAuthorRow {
            authors_author_id: v.try_get(0)?,
            authors_name: v.try_get(1)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const GET_BOOK: &str = r#"-- name: GetBook :one
SELECT book_id, author_id, isbn, book_type, title, year, available, tags FROM books
WHERE book_id = $1"#;
#[derive(Debug, Clone)]
pub struct GetBookRow {
    pub books_book_id: i32,
    pub books_author_id: i32,
    pub books_isbn: String,
    pub books_book_type: BookType,
    pub books_title: String,
    pub books_year: i32,
    pub books_available: ::std::time::SystemTime,
    pub books_tags: Vec<String>,
}
pub async fn get_book(
    client: &impl tokio_postgres::GenericClient,
    books_book_id: &i32,
) -> Result<Option<GetBookRow>, tokio_postgres::Error> {
    let row = client.query_opt(GET_BOOK, &[&books_book_id]).await?;
    let v = match row {
        Some(v) => GetBookRow {
            books_book_id: v.try_get(0)?,
            books_author_id: v.try_get(1)?,
            books_isbn: v.try_get(2)?,
            books_book_type: v.try_get(3)?,
            books_title: v.try_get(4)?,
            books_year: v.try_get(5)?,
            books_available: v.try_get(6)?,
            books_tags: v.try_get(7)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const DELETE_BOOK: &str = r#"-- name: DeleteBook :exec
DELETE FROM books
WHERE book_id = $1"#;
pub async fn delete_book(
    client: &impl tokio_postgres::GenericClient,
    books_book_id: &i32,
) -> Result<u64, tokio_postgres::Error> {
    client.execute(DELETE_BOOK, &[&books_book_id]).await
}
pub const BOOKS_BY_TITLE_YEAR: &str = r#"-- name: BooksByTitleYear :many
SELECT book_id, author_id, isbn, book_type, title, year, available, tags FROM books
WHERE title = $1 AND year = $2"#;
#[derive(Debug, Clone)]
pub struct BooksByTitleYearRow {
    pub books_book_id: i32,
    pub books_author_id: i32,
    pub books_isbn: String,
    pub books_book_type: BookType,
    pub books_title: String,
    pub books_year: i32,
    pub books_available: ::std::time::SystemTime,
    pub books_tags: Vec<String>,
}
pub async fn books_by_title_year(
    client: &impl tokio_postgres::GenericClient,
    books_title: &str,
    books_year: &i32,
) -> Result<
    impl Iterator<Item = Result<BooksByTitleYearRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client
        .query(BOOKS_BY_TITLE_YEAR, &[&books_title, &books_year])
        .await?;
    Ok(rows.into_iter().map(|r| {
        Ok(BooksByTitleYearRow {
            books_book_id: r.try_get(0)?,
            books_author_id: r.try_get(1)?,
            books_isbn: r.try_get(2)?,
            books_book_type: r.try_get(3)?,
            books_title: r.try_get(4)?,
            books_year: r.try_get(5)?,
            books_available: r.try_get(6)?,
            books_tags: r.try_get(7)?,
        })
    }))
}
pub const BOOKS_BY_TAGS: &str = r#"-- name: BooksByTags :many
SELECT 
  book_id,
  title,
  name,
  isbn,
  tags
FROM books
LEFT JOIN authors ON books.author_id = authors.author_id
WHERE tags && $1::varchar[]"#;
#[derive(Debug, Clone)]
pub struct BooksByTagsRow {
    pub books_book_id: i32,
    pub books_title: String,
    pub authors_name: Option<String>,
    pub books_isbn: String,
    pub books_tags: Vec<String>,
}
pub async fn books_by_tags(
    client: &impl tokio_postgres::GenericClient,
    column_1: &[String],
) -> Result<
    impl Iterator<Item = Result<BooksByTagsRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client.query(BOOKS_BY_TAGS, &[&column_1]).await?;
    Ok(rows.into_iter().map(|r| {
        Ok(BooksByTagsRow {
            books_book_id: r.try_get(0)?,
            books_title: r.try_get(1)?,
            authors_name: r.try_get(2)?,
            books_isbn: r.try_get(3)?,
            books_tags: r.try_get(4)?,
        })
    }))
}
pub const CREATE_AUTHOR: &str = r#"-- name: CreateAuthor :one
INSERT INTO authors (name) VALUES ($1)
RETURNING author_id, name"#;
#[derive(Debug, Clone)]
pub struct CreateAuthorRow {
    pub authors_author_id: i32,
    pub authors_name: String,
}
pub async fn create_author(
    client: &impl tokio_postgres::GenericClient,
    authors_name: &str,
) -> Result<Option<CreateAuthorRow>, tokio_postgres::Error> {
    let row = client.query_opt(CREATE_AUTHOR, &[&authors_name]).await?;
    let v = match row {
        Some(v) => CreateAuthorRow {
            authors_author_id: v.try_get(0)?,
            authors_name: v.try_get(1)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const CREATE_BOOK: &str = r#"-- name: CreateBook :one
INSERT INTO books (
    author_id,
    isbn,
    book_type,
    title,
    year,
    available,
    tags
) VALUES (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6,
    $7
)
RETURNING book_id, author_id, isbn, book_type, title, year, available, tags"#;
#[derive(Debug, Clone)]
pub struct CreateBookRow {
    pub books_book_id: i32,
    pub books_author_id: i32,
    pub books_isbn: String,
    pub books_book_type: BookType,
    pub books_title: String,
    pub books_year: i32,
    pub books_available: ::std::time::SystemTime,
    pub books_tags: Vec<String>,
}
pub async fn create_book(
    client: &impl tokio_postgres::GenericClient,
    books_author_id: &i32,
    books_isbn: &str,
    books_book_type: &BookType,
    books_title: &str,
    books_year: &i32,
    books_available: &::std::time::SystemTime,
    books_tags: &[String],
) -> Result<Option<CreateBookRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(
            CREATE_BOOK,
            &[
                &books_author_id,
                &books_isbn,
                &books_book_type,
                &books_title,
                &books_year,
                &books_available,
                &books_tags,
            ],
        )
        .await?;
    let v = match row {
        Some(v) => CreateBookRow {
            books_book_id: v.try_get(0)?,
            books_author_id: v.try_get(1)?,
            books_isbn: v.try_get(2)?,
            books_book_type: v.try_get(3)?,
            books_title: v.try_get(4)?,
            books_year: v.try_get(5)?,
            books_available: v.try_get(6)?,
            books_tags: v.try_get(7)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const UPDATE_BOOK: &str = r#"-- name: UpdateBook :exec
UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3"#;
pub async fn update_book(
    client: &impl tokio_postgres::GenericClient,
    books_title: &str,
    books_tags: &[String],
    books_book_id: &i32,
) -> Result<u64, tokio_postgres::Error> {
    client
        .execute(UPDATE_BOOK, &[&books_title, &books_tags, &books_book_id])
        .await
}
pub const UPDATE_BOOK_ISBN: &str = r#"-- name: UpdateBookISBN :exec
UPDATE books
SET title = $1, tags = $2, isbn = $4
WHERE book_id = $3"#;
pub async fn update_book_isbn(
    client: &impl tokio_postgres::GenericClient,
    books_title: &str,
    books_tags: &[String],
    books_book_id: &i32,
    books_isbn: &str,
) -> Result<u64, tokio_postgres::Error> {
    client
        .execute(
            UPDATE_BOOK_ISBN,
            &[&books_title, &books_tags, &books_book_id, &books_isbn],
        )
        .await
}
pub const SAY_HELLO: &str = r#"-- name: SayHello :one
select say_hello from say_hello($1)"#;
#[derive(Debug, Clone)]
pub struct SayHelloRow {
    pub say_hello: Option<String>,
}
pub async fn say_hello(
    client: &impl tokio_postgres::GenericClient,
    s: &str,
) -> Result<Option<SayHelloRow>, tokio_postgres::Error> {
    let row = client.query_opt(SAY_HELLO, &[&s]).await?;
    let v = match row {
        Some(v) => SayHelloRow {
            say_hello: v.try_get(0)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
