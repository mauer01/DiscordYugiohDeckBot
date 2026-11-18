use crate::deck::process_deck_input;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Formats a YDKE link into a copyable Markdown code block.
pub fn format_ydke_codeblock(ydke_link: &str) -> String {
    format!("```\n{}\n```", ydke_link)
}

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
            let content = format_ydke_codeblock(&ydke_link);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ydke_codeblock() {
        let link = "ydke://ABC!DEF!GHI!";
        let formatted = format_ydke_codeblock(link);
        assert_eq!(formatted, "```\nydke://ABC!DEF!GHI!\n```");
    }

    #[test]
    fn test_deck_command_metadata() {
        let cmd = deck();
        assert_eq!(cmd.name, "deck");
        assert!(cmd.slash_action.is_some());
        assert!(cmd.description.as_deref().unwrap_or("").contains("Yu-Gi-Oh!"));
        
        // Verify parameters
        let params = &cmd.parameters;
        assert_eq!(params.len(), 2);
        
        let query_param = &params[0];
        assert_eq!(query_param.name, "query");
        assert!(!query_param.required);
        assert!(query_param.description.as_deref().unwrap_or("").contains("YDKE"));

        let file_param = &params[1];
        assert_eq!(file_param.name, "file");
        assert!(!file_param.required);
        assert!(file_param.description.as_deref().unwrap_or("").contains("attachment"));
    }
}
