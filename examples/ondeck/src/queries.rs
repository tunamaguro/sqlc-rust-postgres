//! Code generated by sqlc. SHOULD NOT EDIT.
//! sqlc version: v1.28.0
//! sqlc-rust-postgres version: v0.1.2
#[derive(Debug, Clone, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "status")]
pub enum Status {
    #[postgres(name = "op!en")]
    Open,
    #[postgres(name = "clo@sed")]
    Closed,
}
pub const LIST_CITIES: &str = r#"-- name: ListCities :many
SELECT slug, name
FROM city
ORDER BY name"#;
#[derive(Debug, Clone)]
pub struct ListCitiesRow {
    pub city_slug: String,
    pub city_name: String,
}
pub async fn list_cities(
    client: &impl deadpool_postgres::GenericClient,
) -> Result<
    impl Iterator<Item = Result<ListCitiesRow, deadpool_postgres::tokio_postgres::Error>>,
    deadpool_postgres::tokio_postgres::Error,
> {
    let rows = client.query(LIST_CITIES, &[]).await?;
    Ok(rows.into_iter().map(|r| {
        Ok(ListCitiesRow {
            city_slug: r.try_get(0)?,
            city_name: r.try_get(1)?,
        })
    }))
}
pub const GET_CITY: &str = r#"-- name: GetCity :one
SELECT slug, name
FROM city
WHERE slug = $1"#;
#[derive(Debug, Clone)]
pub struct GetCityRow {
    pub city_slug: String,
    pub city_name: String,
}
pub async fn get_city(
    client: &impl deadpool_postgres::GenericClient,
    city_slug: &str,
) -> Result<Option<GetCityRow>, deadpool_postgres::tokio_postgres::Error> {
    let row = client.query_opt(GET_CITY, &[&city_slug]).await?;
    let v = match row {
        Some(v) => GetCityRow {
            city_slug: v.try_get(0)?,
            city_name: v.try_get(1)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const CREATE_CITY: &str = r#"-- name: CreateCity :one
INSERT INTO city (
    name,
    slug
) VALUES (
    $1,
    $2
) RETURNING slug, name"#;
#[derive(Debug, Clone)]
pub struct CreateCityRow {
    pub city_slug: String,
    pub city_name: String,
}
pub async fn create_city(
    client: &impl deadpool_postgres::GenericClient,
    city_name: &str,
    city_slug: &str,
) -> Result<Option<CreateCityRow>, deadpool_postgres::tokio_postgres::Error> {
    let row = client
        .query_opt(CREATE_CITY, &[&city_name, &city_slug])
        .await?;
    let v = match row {
        Some(v) => CreateCityRow {
            city_slug: v.try_get(0)?,
            city_name: v.try_get(1)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const UPDATE_CITY_NAME: &str = r#"-- name: UpdateCityName :exec
UPDATE city
SET name = $2
WHERE slug = $1"#;
pub async fn update_city_name(
    client: &impl deadpool_postgres::GenericClient,
    city_slug: &str,
    city_name: &str,
) -> Result<u64, deadpool_postgres::tokio_postgres::Error> {
    client
        .execute(UPDATE_CITY_NAME, &[&city_slug, &city_name])
        .await
}
pub const LIST_VENUES: &str = r#"-- name: ListVenues :many
SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE city = $1
ORDER BY name"#;
#[derive(Debug, Clone)]
pub struct ListVenuesRow {
    pub venue_id: i32,
    pub venue_status: Status,
    pub venue_statuses: Option<Vec<Status>>,
    pub venue_slug: String,
    pub venue_name: String,
    pub venue_city: String,
    pub venue_spotify_playlist: String,
    pub venue_songkick_id: Option<String>,
    pub venue_tags: Option<Vec<String>>,
    pub venue_created_at: ::std::time::SystemTime,
}
pub async fn list_venues(
    client: &impl deadpool_postgres::GenericClient,
    venue_city: &str,
) -> Result<
    impl Iterator<Item = Result<ListVenuesRow, deadpool_postgres::tokio_postgres::Error>>,
    deadpool_postgres::tokio_postgres::Error,
> {
    let rows = client.query(LIST_VENUES, &[&venue_city]).await?;
    Ok(rows.into_iter().map(|r| {
        Ok(ListVenuesRow {
            venue_id: r.try_get(0)?,
            venue_status: r.try_get(1)?,
            venue_statuses: r.try_get(2)?,
            venue_slug: r.try_get(3)?,
            venue_name: r.try_get(4)?,
            venue_city: r.try_get(5)?,
            venue_spotify_playlist: r.try_get(6)?,
            venue_songkick_id: r.try_get(7)?,
            venue_tags: r.try_get(8)?,
            venue_created_at: r.try_get(9)?,
        })
    }))
}
pub const DELETE_VENUE: &str = r#"-- name: DeleteVenue :exec
DELETE FROM venue
WHERE slug = $1 AND slug = $1"#;
pub async fn delete_venue(
    client: &impl deadpool_postgres::GenericClient,
    venue_slug: &str,
) -> Result<u64, deadpool_postgres::tokio_postgres::Error> {
    client.execute(DELETE_VENUE, &[&venue_slug]).await
}
pub const GET_VENUE: &str = r#"-- name: GetVenue :one
SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE slug = $1 AND city = $2"#;
#[derive(Debug, Clone)]
pub struct GetVenueRow {
    pub venue_id: i32,
    pub venue_status: Status,
    pub venue_statuses: Option<Vec<Status>>,
    pub venue_slug: String,
    pub venue_name: String,
    pub venue_city: String,
    pub venue_spotify_playlist: String,
    pub venue_songkick_id: Option<String>,
    pub venue_tags: Option<Vec<String>>,
    pub venue_created_at: ::std::time::SystemTime,
}
pub async fn get_venue(
    client: &impl deadpool_postgres::GenericClient,
    venue_slug: &str,
    venue_city: &str,
) -> Result<Option<GetVenueRow>, deadpool_postgres::tokio_postgres::Error> {
    let row = client
        .query_opt(GET_VENUE, &[&venue_slug, &venue_city])
        .await?;
    let v = match row {
        Some(v) => GetVenueRow {
            venue_id: v.try_get(0)?,
            venue_status: v.try_get(1)?,
            venue_statuses: v.try_get(2)?,
            venue_slug: v.try_get(3)?,
            venue_name: v.try_get(4)?,
            venue_city: v.try_get(5)?,
            venue_spotify_playlist: v.try_get(6)?,
            venue_songkick_id: v.try_get(7)?,
            venue_tags: v.try_get(8)?,
            venue_created_at: v.try_get(9)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const CREATE_VENUE: &str = r#"-- name: CreateVenue :one
INSERT INTO venue (
    slug,
    name,
    city,
    created_at,
    spotify_playlist,
    status,
    statuses,
    tags
) VALUES (
    $1,
    $2,
    $3,
    NOW(),
    $4,
    $5,
    $6,
    $7
) RETURNING id"#;
#[derive(Debug, Clone)]
pub struct CreateVenueRow {
    pub venue_id: i32,
}
pub async fn create_venue(
    client: &impl deadpool_postgres::GenericClient,
    venue_slug: &str,
    venue_name: &str,
    venue_city: &str,
    venue_spotify_playlist: &str,
    venue_status: &Status,
    venue_statuses: Option<&[Status]>,
    venue_tags: Option<&[String]>,
) -> Result<Option<CreateVenueRow>, deadpool_postgres::tokio_postgres::Error> {
    let row = client
        .query_opt(
            CREATE_VENUE,
            &[
                &venue_slug,
                &venue_name,
                &venue_city,
                &venue_spotify_playlist,
                &venue_status,
                &venue_statuses,
                &venue_tags,
            ],
        )
        .await?;
    let v = match row {
        Some(v) => CreateVenueRow {
            venue_id: v.try_get(0)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const UPDATE_VENUE_NAME: &str = r#"-- name: UpdateVenueName :one
UPDATE venue
SET name = $2
WHERE slug = $1
RETURNING id"#;
#[derive(Debug, Clone)]
pub struct UpdateVenueNameRow {
    pub venue_id: i32,
}
pub async fn update_venue_name(
    client: &impl deadpool_postgres::GenericClient,
    venue_slug: &str,
    venue_name: &str,
) -> Result<Option<UpdateVenueNameRow>, deadpool_postgres::tokio_postgres::Error> {
    let row = client
        .query_opt(UPDATE_VENUE_NAME, &[&venue_slug, &venue_name])
        .await?;
    let v = match row {
        Some(v) => UpdateVenueNameRow {
            venue_id: v.try_get(0)?,
        },
        None => return Ok(None),
    };
    Ok(Some(v))
}
pub const VENUE_COUNT_BY_CITY: &str = r#"-- name: VenueCountByCity :many
SELECT
    city,
    count(*)
FROM venue
GROUP BY 1
ORDER BY 1"#;
#[derive(Debug, Clone)]
pub struct VenueCountByCityRow {
    pub venue_city: String,
    pub count: i64,
}
pub async fn venue_count_by_city(
    client: &impl deadpool_postgres::GenericClient,
) -> Result<
    impl Iterator<Item = Result<VenueCountByCityRow, deadpool_postgres::tokio_postgres::Error>>,
    deadpool_postgres::tokio_postgres::Error,
> {
    let rows = client.query(VENUE_COUNT_BY_CITY, &[]).await?;
    Ok(rows.into_iter().map(|r| {
        Ok(VenueCountByCityRow {
            venue_city: r.try_get(0)?,
            count: r.try_get(1)?,
        })
    }))
}
