#![feature(async_closure)]

use anyhow::{Context, Result};
use chrono::Duration;
use dotenv::dotenv;
use log::info;
use serenity::{client::validate_token, http::client::Http};
use std::env;
use tokio::time;

mod bot;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let discord_token = env::var("DISCORD_TOKEN").context("DISCORD_TOKEN is unset")?;
    let channel_retention = env::var("CHANNEL_RETENTION")
        .context("CHANNEL_RETENTION is unset")
        .and_then(config::parse_channel_retention)
        .context("Could not parse channel retention")?;
    let def_retention = Duration::weeks(50);
    let min_retention = channel_retention.values().min()
        .unwrap_or(&def_retention);
    info!("Minimum retention interval {} hours", min_retention.num_hours());
    let delete_pinned = env::var("DELETE_PINNED")
        .map(|val| val == "true")
        .unwrap_or(false);
    validate_token(&discord_token).context("Token is invalid")?;

    let client = Http::new_with_token(&discord_token);

    let tick_duration = Duration::minutes(min_retention.num_minutes() / 10);
    let mut interval = time::interval(tick_duration.to_std()?);
    interval.tick().await; // the first tick completes immediately

    loop {
        bot::run(&client, &channel_retention, delete_pinned).await?;
        info!("Sleeping until the time interval is up");
        interval.tick().await;
    }
}
