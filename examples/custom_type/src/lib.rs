pub(crate) mod queries;

use postgres_types::{FromSql, ToSql};

#[derive(Debug, Clone, FromSql, ToSql)]
struct VoiceActor {
    name: String,
    age: i32,
}

#[derive(Debug, Clone, FromSql, ToSql)]

struct TempStruct {
    val: i32,
}
