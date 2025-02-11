pub(crate) mod queries;

use postgres_types::{FromSql, ToSql};

#[derive(Debug, Clone, PartialEq, FromSql, ToSql)]
struct VoiceActor {
    name: String,
    age: i32,
}
