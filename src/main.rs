use dotenv::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::env;

use poise::serenity_prelude::{self as serenity, GatewayIntents};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    db: SqlitePool,
}

#[poise::command(slash_command, prefix_command, track_edits)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(
            env::var("DATABASE_URL")
                .expect("DATABASE_URL not found")
                .as_str(),
        )
        .await
        .expect("Could not connect to database");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Couldn't run database migrations");

    let bot = Data { db };

    let intents = GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    serenity::GuildId(1091304319552335892),
                )
                .await?;
                Ok(bot)
            })
        });

    framework.run().await.unwrap();
}
