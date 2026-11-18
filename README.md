# Discord Yu-Gi-Oh! Deck Bot

A Discord bot written in Rust that converts Yu-Gi-Oh! `ydke://` links, `.ydk` files/text, and XML deck formats into visual compound deck images (Main, Extra, and Side decks) and echoes the generated `ydke://` link in an easy-to-copy codeblock.

### [Invite Bot to Discord](https://discord.com/oauth2/authorize?client_id=1572430428839739563)

---

## Features

- **Slash Command (`/deck`)**:
  - Accepts a `ydke://` link, raw `.ydk` text, or XML deck markup via the `query` option.
  - Accepts attached `.ydk` or `.xml` deck files via the optional `file` upload option.
- **Compound Image Rendering**:
  - **Main Deck**: 10 cards per row (up to 6 rows, 60 card capacity).
  - **Extra Deck**: 15 cards per row.
  - **Side Deck**: 15 cards per row.
- **Copyable YDKE Echo**: Automatically converts any input format (YDK, XML, or YDKE) to a canonical `ydke://` URL and replies with it inside a Markdown codeblock.

---

## Supported Deck Formats

1. **YDKE URLs**:
   ```
   ydke://<main_b64>!<extra_b64>!<side_b64>!
   ```
2. **YDK Files (`.ydk`)**:
   ```
   #main
   74311226
   37104630
   #extra
   26593852
   !side
   40916023
   ```
3. **XML Deck Files (`.xml`)**:
   ```xml
   <deck name="DeckName">
     <main>
       <card id="5543" passcode="74311226">Atlantean Dragoons</card>
     </main>
     <extra>
       <card id="85" passcode="26593852">Ally of Justice Catastor</card>
     </extra>
     <side>
       <card id="196" passcode="40916023">Aqua Spirit</card>
     </side>
   </deck>
   ```

---

## Environment Variables

Create a `.env` file in the project root or configure your runtime environment:

| Variable | Description | Default |
| --- | --- | --- |
| `DISCORD_TOKEN` | Discord bot token (**required**) | - |
| `CARD_IMAGES_PATH` | Path to the directory containing `<card_id>.jpg` images | `/app/pics` |

---

## Running with Cargo

```bash
# Run tests
cargo test

# Run the bot
cargo run --release
```

---

## Building with Docker

The `Dockerfile` supports downloading and extracting card images during build time:

```bash
docker build \
  --build-arg PICS_URL="https://example.com/pics.tar.gz" \
  -t discord-yugioh-deck-bot .

docker run -d \
  -e DISCORD_TOKEN="your_bot_token" \
  discord-yugioh-deck-bot
```
