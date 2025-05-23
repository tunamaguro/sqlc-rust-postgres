//! Code generated by sqlc. SHOULD NOT EDIT.
//! sqlc version: v1.28.0
//! sqlc-rust-postgres version: v0.1.4
pub const GET_AUTHOR: &str = r#"-- name: GetAuthor :one
SELECT id, name, bio FROM authors
WHERE id = $1 LIMIT 1"#;
#[derive(Debug, Clone)]
pub struct GetAuthorRow {
    pub authors_id: i64,
    pub authors_name: String,
    pub authors_bio: Option<String>,
}
pub async fn get_author(
    client: &impl tokio_postgres::GenericClient,
    authors_id: &i64,
) -> Result<Option<GetAuthorRow>, tokio_postgres::Error> {
    let row = client.query_opt(GET_AUTHOR, &[&authors_id]).await?;
    let v = match row {
        Some(v) => GetAuthorRow {
            authors_id: v.try_get(0)?,
            authors_name: v.try_get(1)?,
            authors_bio: v.try_get(2)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const LIST_AUTHORS: &str = r#"-- name: ListAuthors :many
SELECT id, name, bio FROM authors
ORDER BY name"#;
#[derive(Debug, Clone)]
pub struct ListAuthorsRow {
    pub authors_id: i64,
    pub authors_name: String,
    pub authors_bio: Option<String>,
}
pub async fn list_authors(
    client: &impl tokio_postgres::GenericClient,
) -> Result<
    impl Iterator<Item = Result<ListAuthorsRow, tokio_postgres::Error>>,
    tokio_postgres::Error,
> {
    let rows = client.query(LIST_AUTHORS, &[]).await?;
    Ok(rows.into_iter().map(|r| {
        Ok(ListAuthorsRow {
            authors_id: r.try_get(0)?,
            authors_name: r.try_get(1)?,
            authors_bio: r.try_get(2)?,
        })
    }))
}
pub const CREATE_AUTHOR: &str = r#"-- name: CreateAuthor :one
INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING id, name, bio"#;
#[derive(Debug, Clone)]
pub struct CreateAuthorRow {
    pub authors_id: i64,
    pub authors_name: String,
    pub authors_bio: Option<String>,
}
pub async fn create_author(
    client: &impl tokio_postgres::GenericClient,
    authors_name: &str,
    authors_bio: Option<&str>,
) -> Result<Option<CreateAuthorRow>, tokio_postgres::Error> {
    let row = client
        .query_opt(CREATE_AUTHOR, &[&authors_name, &authors_bio])
        .await?;
    let v = match row {
        Some(v) => CreateAuthorRow {
            authors_id: v.try_get(0)?,
            authors_name: v.try_get(1)?,
            authors_bio: v.try_get(2)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const DELETE_AUTHOR: &str = r#"-- name: DeleteAuthor :exec
DELETE FROM authors
WHERE id = $1"#;
pub async fn delete_author(
    client: &impl tokio_postgres::GenericClient,
    authors_id: &i64,
) -> Result<u64, tokio_postgres::Error> {
    client.execute(DELETE_AUTHOR, &[&authors_id]).await
}
