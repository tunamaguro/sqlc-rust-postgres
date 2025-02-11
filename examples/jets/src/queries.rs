//! Code generated by sqlc. SHOULD NOT EDIT.
//! sqlc version: v1.28.0
//! sqlc-rust-postgres version: 0.1.0
pub const COUNT_PILOTS: &str = r#"-- name: CountPilots :one
SELECT COUNT(*) FROM pilots"#;
#[derive(Debug, Clone)]
pub struct CountPilotsRow {
    count: i64,
}
fn count_pilots(
    client: &impl postgres::GenericClient,
) -> Result<CountPilotsRow, postgres::Error> {
    let row = client.query_one(COUNT_PILOTS, &[]).await?;
    Ok(CountPilotsRow {
        count: row.try_get(0)?,
    })
}
pub const LIST_PILOTS: &str = r#"-- name: ListPilots :many
SELECT id, name FROM pilots LIMIT 5"#;
#[derive(Debug, Clone)]
pub struct ListPilotsRow {
    pilots_id: i32,
    pilots_name: String,
}
fn list_pilots(
    client: &impl postgres::GenericClient,
) -> Result<
    impl Iterator<Item = Result<ListPilotsRow, postgres::Error>>,
    postgres::Error,
> {
    let rows = client.query(LIST_PILOTS, &[]).await?;
    Ok(
        rows
            .into_iter()
            .map(|r| Ok(ListPilotsRow {
                pilots_id: r.try_get(0)?,
                pilots_name: r.try_get(1)?,
            })),
    )
}
pub const DELETE_PILOT: &str = r#"-- name: DeletePilot :exec
DELETE FROM pilots WHERE id = $1"#;
fn delete_pilot(
    client: &impl postgres::GenericClient,
    pilots_id: &i32,
) -> Result<u64, postgres::Error> {
    client.execute(DELETE_PILOT, &[&pilots_id]).await
}
