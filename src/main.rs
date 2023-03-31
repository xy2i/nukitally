use chrono::prelude::*;
use chrono::{DateTime, Datelike, Duration, Month, NaiveDate, TimeZone, Timelike, Utc, Weekday};
use dotenv::dotenv;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use sqlx::{query, sqlite::SqlitePoolOptions};
use sqlx::{query_as, SqlitePool};
use std::env;
use std::thread::Thread;

use poise::serenity_prelude::{self as serenity, GatewayIntents};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

fn week_start(dt: NaiveDateTime) -> NaiveDateTime {
    let week = dt.iso_week().week();
    let current_year = Utc::now().year();
    NaiveDate::from_isoywd_opt(current_year, week, Weekday::Mon)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

fn month_start(dt: NaiveDateTime) -> NaiveDateTime {
    let month = dt.month();
    let current_year = Utc::now().year();
    NaiveDate::from_ymd_opt(current_year, month, 1)
        .unwrap()
        .and_time(NaiveTime::MIN)
}

#[derive(Debug)]
struct NukiLog {
    discord_uid: String,
    count: i64,
    timestamp: NaiveDateTime,
    comment: Option<String>,
}

pub struct Data {
    db: SqlitePool,
}

#[poise::command(prefix_command)]
async fn sudo(ctx: Context<'_>) -> Result<(), Error> {
    //poise::builtins::register_application_commands_buttons(ctx).await?;
    ctx.say("fuck you").await?;
    Ok(())
}

/// Log your nukis
#[poise::command(slash_command)]
async fn nuki(
    ctx: Context<'_>,
    #[description = "(Default: 1) The number of times you nuki'd"]
    #[min = 1]
    #[max = 20]
    amount: Option<u8>,
    #[description = "Message to log"] comment: Option<String>,
) -> Result<(), Error> {
    let amount = amount.unwrap_or(1);
    let id = ctx.author().id.to_string();

    let now = Utc::now().naive_utc();
    query!(
        r#"insert into nuki_log(discord_uid, count, timestamp, comment) values (?, ?, ?, ?)"#,
        id,
        amount,
        now,
        comment,
    )
    .execute(&ctx.data().db)
    .await?;

    let plural = if amount != 1 { "s" } else { "" };
    let mut reply = format!("Logged {amount} nuki{plural}!");

    let random = { rand::thread_rng().gen_range(0..500) };
    if random == 0 {
        reply += " ";
        for _ in 0..amount {
            reply += "<:KimoiHuh:1091045264585928784>";
        }
    }

    if let Some(message) = comment {
        reply += format!("\n> {message}").as_str();
    }

    ctx.say(reply).await?;

    Ok(())
}

/// Backlog a nuki
#[poise::command(slash_command)]
async fn nback(
    ctx: Context<'_>,
    #[description = "(Example: 2023-03-31) Time when the nuki was made"] date: NaiveDate,
    #[description = "(Default: 1) The number of times you nuki'd"]
    #[min = 1]
    #[max = 20]
    amount: Option<u8>,
    #[description = "Message to log"] comment: Option<String>,
) -> Result<(), Error> {
    let amount = amount.unwrap_or(1);
    let id = ctx.author().id.to_string();
    let dt = date.and_time(NaiveTime::MIN);

    query!(
        r#"insert into nuki_log(discord_uid, count, timestamp, comment) values (?, ?, ?, ?)"#,
        id,
        amount,
        dt,
        comment,
    )
    .execute(&ctx.data().db)
    .await?;

    let plural = if amount != 1 { "s" } else { "" };
    let mut reply = format!(
        "Back-logged {amount} nuki{plural} at <t:{}:R>!",
        dt.timestamp()
    );

    let random = { rand::thread_rng().gen_range(0..500) };
    if random == 0 {
        reply += " ";
        for _ in 0..amount {
            reply += "<:KimoiHuh:1091045264585928784>";
        }
    }

    if let Some(message) = comment {
        reply += format!("\n> {message}").as_str();
    }

    ctx.say(reply).await?;

    Ok(())
}

/// Undo your last nuki
#[poise::command(slash_command)]
async fn nundo(ctx: Context<'_>) -> Result<(), Error> {
    let uid = ctx.author().id.0.to_string();

    query!(
        r#"delete from nuki_log where discord_uid=? and timestamp=
        (select max(timestamp) from nuki_log where discord_uid=?)"#,
        uid,
        uid,
    )
    .execute(&ctx.data().db)
    .await?;

    ctx.say("Undoed last nuki.").await?;

    Ok(())
}

/// Get your or someone's else nukis
#[poise::command(slash_command)]
async fn nukis(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let uid = u.id.0.to_string();
    let mut nukis = query_as!(
        NukiLog,
        r#"select * from nuki_log where discord_uid=?"#,
        uid
    )
    .fetch_all(&ctx.data().db)
    .await?;

    nukis.sort_by_key(|k| k.timestamp);
    nukis.reverse();

    let count: i64 = nukis.iter().map(|v| v.count).sum();

    let mut reply = if count > 0 {
        format!("<@{uid}>: {count} nukis.")
    } else {
        format!("<@{uid}>: no nukis.")
    };

    let now = Utc::now().naive_utc();

    let this_week = week_start(now);
    let count_week: i64 = nukis
        .iter()
        .filter(|v| v.timestamp > this_week)
        .map(|v| v.count)
        .sum();
    if count_week > 0 {
        reply += format!(" {count_week} this week.").as_str();
    }

    let this_month = month_start(now);
    let count_month: i64 = nukis
        .iter()
        .filter(|v| v.timestamp > this_month)
        .map(|v| v.count)
        .sum();
    if count_month > 0 {
        reply += format!(" {count_month} this month.").as_str();
    }

    if let Some(nuki) = nukis.get(0) {
        let last_nuki_dt = nuki.timestamp;
        reply += format!(" Last nuki <t:{}:R>", last_nuki_dt.timestamp()).as_str();
    }

    ctx.say(reply).await?;
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
            commands: vec![nuki(), nukis(), nback(), nundo()],
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
