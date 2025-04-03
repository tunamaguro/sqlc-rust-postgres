//! Code generated by sqlc. SHOULD NOT EDIT.
//! sqlc version: v1.28.0
//! sqlc-rust-postgres version: v0.1.3
pub const COUNT_PILOTS: &str = r#"-- name: CountPilots :one
SELECT COUNT(*) FROM pilots"#;
#[derive(Debug, Clone)]
pub struct CountPilotsRow {
    pub count: i64,
}
pub fn count_pilots(
    client: &mut impl postgres::GenericClient,
) -> Result<Option<CountPilotsRow>, postgres::Error> {
    let row = client.query_opt(COUNT_PILOTS, &[])?;
    let v = match row {
        Some(v) => CountPilotsRow {
            count: v.try_get(0)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const LIST_PILOTS: &str = r#"-- name: ListPilots :many
SELECT id, name FROM pilots LIMIT 5"#;
#[derive(Debug, Clone)]
pub struct ListPilotsRow {
    pub pilots_id: i32,
    pub pilots_name: String,
}
pub fn list_pilots(
    client: &mut impl postgres::GenericClient,
) -> Result<impl Iterator<Item = Result<ListPilotsRow, postgres::Error>>, postgres::Error> {
    let rows = client.query(LIST_PILOTS, &[])?;
    Ok(rows.into_iter().map(|r| {
        Ok(ListPilotsRow {
            pilots_id: r.try_get(0)?,
            pilots_name: r.try_get(1)?,
        })
    }))
}
pub const DELETE_PILOT: &str = r#"-- name: DeletePilot :exec
DELETE FROM pilots WHERE id = $1"#;
pub fn delete_pilot(
    client: &mut impl postgres::GenericClient,
    pilots_id: &i32,
) -> Result<u64, postgres::Error> {
    client.execute(DELETE_PILOT, &[&pilots_id])
}
