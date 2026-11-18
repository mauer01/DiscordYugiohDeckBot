use crate::deck::process_deck_input;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Render a Yu-Gi-Oh! deck from a YDKE link, YDK text/file, or XML into a compound image
#[poise::command(slash_command)]
pub async fn deck(
    ctx: Context<'_>,
    #[description = "The YDKE link (ydke://...), YDK text, or XML deck"] query: Option<String>,
    #[description = "Optional .ydk or .xml file attachment"] file: Option<serenity::Attachment>,
) -> Result<(), Error> {
    let input_text = if let Some(att) = file {
        let bytes = att.download().await?;
        String::from_utf8_lossy(&bytes).to_string()
    } else if let Some(q) = query {
        q
    } else {
        ctx.say("Please provide a YDKE link, YDK/XML text, or attach a .ydk/.xml file.")
            .await?;
        return Ok(());
    };

    ctx.defer().await?;
    match process_deck_input(&input_text) {
        Ok((deck, image_bytes)) => {
            let ydke_link = deck.to_ydke_url();
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
    Ok(())
}
