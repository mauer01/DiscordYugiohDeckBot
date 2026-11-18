pub mod commands;
pub mod deck;

use poise::serenity_prelude as serenity;

#[derive(Debug, Default, Clone)]
pub struct Data {} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Constructs framework options with all registered commands.
pub fn create_framework_options() -> poise::FrameworkOptions<Data, Error> {
    poise::FrameworkOptions {
        commands: vec![commands::deck()],
        ..Default::default()
    }
}

/// Builds the Poise framework for the bot.
pub fn create_framework() -> poise::FrameworkBuilder<Data, Error> {
    poise::Framework::builder()
        .options(create_framework_options())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_instantiation_and_traits() {
        let data = Data::default();
        let cloned = data.clone();
        assert_eq!(format!("{data:?}"), "Data");
        assert_eq!(format!("{cloned:?}"), "Data");
    }

    #[test]
    fn test_framework_options_registration() {
        let options = create_framework_options();
        assert_eq!(options.commands.len(), 1);
        assert_eq!(options.commands[0].name, "deck");
    }

    #[test]
    fn test_create_framework_builds() {
        let _framework = create_framework().build();
    }
}
