use crate::deck::{extract_ydke_url, process_ydke};
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Render a Yu-Gi-Oh! deck from a YDKE link into a compound image
#[poise::command(slash_command)]
pub async fn deck(
    ctx: Context<'_>,
    #[description = "The YDKE link (ydke://...) or query"] query: String,
) -> Result<(), Error> {
    if query.contains("ydke://") {
        ctx.defer().await?;
        match process_ydke(&query) {
            Ok(image_bytes) => {
                let ydke_link = extract_ydke_url(&query).unwrap_or(&query);
                let content = format!("```\n{}\n```", ydke_link);
                let attachment = serenity::CreateAttachment::bytes(image_bytes, "deck.png");
                ctx.send(
                    poise::CreateReply::default()
                        .content(content)
                        .attachment(attachment),
                )
                .await?;
            }
            Err(e) => {
                ctx.say(format!("Failed to generate deck image: {e}")).await?;
            }
        }
    } else {
        let response = format!("Received search query: `{}`", query);
        ctx.say(response).await?;
    }
    Ok(())
}
