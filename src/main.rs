use discord_yugioh_deck_bot::{create_framework, Error};
use poise::serenity_prelude as serenity;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN in environment or .env file");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = create_framework().build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}
