#[allow(warnings)]
pub(crate) mod queries;

use postgres_types::{FromSql, ToSql};

#[derive(Debug, Clone, PartialEq, FromSql, ToSql)]
#[postgres(name = "voiceactor")]
struct VoiceActor {
    name: String,
    age: i32,
}

#[cfg(test)]
mod tests {
    use crate::{queries, VoiceActor};
    use test_context::test_context;
    use test_utils::PgTokioTestContext;

    async fn migrate_db(clinet: &tokio_postgres::Client) {
        clinet
            .batch_execute(include_str!("./schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn queries_works(ctx: &mut PgTokioTestContext) {
        migrate_db(&ctx.client).await;

        let voice_actor = VoiceActor {
            age: 20,
            name: "Foo".to_owned(),
        };
        let character = queries::SpongeBobCharacter::Bob;
        let _ = queries::create_voice_actor(&ctx.client, Some(&voice_actor), Some(&character))
            .await
            .unwrap();
        let actors = queries::get_custom_type(&ctx.client)
            .await
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let first = actors.first().unwrap();
        assert_eq!(first.voice_actor, voice_actor.into());
        assert_eq!(first.character, character.into());
    }
}
