use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use image::imageops::FilterType;
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Represents a parsed Yu-Gi-Oh! deck from a YDKE link, YDK file, or XML.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct YdkeDeck {
    pub main: Vec<u32>,
    pub extra: Vec<u32>,
    pub side: Vec<u32>,
}

impl YdkeDeck {
    /// Encodes the deck into a standard `ydke://` URL.
    pub fn to_ydke_url(&self) -> String {
        let encode_section = |cards: &[u32]| -> String {
            let mut bytes = Vec::with_capacity(cards.len() * 4);
            for card in cards {
                bytes.extend_from_slice(&card.to_le_bytes());
            }
            STANDARD.encode(&bytes)
        };

        let main_b64 = encode_section(&self.main);
        let extra_b64 = encode_section(&self.extra);
        let side_b64 = encode_section(&self.side);

        format!("ydke://{main_b64}!{extra_b64}!{side_b64}!")
    }
}

#[derive(Debug)]
pub enum DeckError {
    InvalidScheme,
    InvalidSections,
    Base64DecodeError(String),
    InvalidByteLength(usize),
    EmptyDeckOrInvalidFormat,
    RenderError(String),
}

pub type YdkeError = DeckError;

impl std::fmt::Display for DeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeckError::InvalidScheme => write!(f, "YDKE URL must start with 'ydke://'"),
            DeckError::InvalidSections => {
                write!(f, "YDKE URL must contain main, extra, and side deck sections")
            }
            DeckError::Base64DecodeError(e) => write!(f, "Failed to decode base64: {e}"),
            DeckError::InvalidByteLength(len) => {
                write!(f, "Decoded byte length ({len}) is not a multiple of 4")
            }
            DeckError::EmptyDeckOrInvalidFormat => {
                write!(f, "No valid cards found in input (invalid YDKE, YDK, or XML format)")
            }
            DeckError::RenderError(e) => write!(f, "Failed to render deck image: {e}"),
        }
    }
}

impl std::error::Error for DeckError {}

/// Extracts the YDKE URL string from an input text.
pub fn extract_ydke_url(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    let ydke_start = trimmed.find("ydke://")?;
    let ydke_str = &trimmed[ydke_start..];
    let ydke_url = ydke_str.split_whitespace().next().unwrap_or(ydke_str);
    Some(ydke_url)
}

/// Scans and extracts a YDKE URL from a text query/message and parses it into `YdkeDeck`.
pub fn parse_ydke(input: &str) -> Result<YdkeDeck, DeckError> {
    let ydke_url = extract_ydke_url(input).ok_or(DeckError::InvalidScheme)?;
    let ydke_content = &ydke_url["ydke://".len()..];

    // YDKE sections are separated by '!'
    // Example: <main_b64>!<extra_b64>!<side_b64>!
    let parts: Vec<&str> = ydke_content.split('!').collect();
    if parts.len() < 3 {
        return Err(DeckError::InvalidSections);
    }

    let main = decode_card_ids(parts[0])?;
    let extra = decode_card_ids(parts[1])?;
    let side = decode_card_ids(parts[2])?;

    Ok(YdkeDeck { main, extra, side })
}

/// Parses a `.ydk` deck format containing `#main`, `#extra`, and `!side` / `#side` sections.
pub fn parse_ydk(input: &str) -> Result<YdkeDeck, DeckError> {
    #[derive(Copy, Clone, PartialEq, Eq)]
    enum Section {
        Main,
        Extra,
        Side,
    }

    let mut current_section: Option<Section> = None;
    let mut main = Vec::new();
    let mut extra = Vec::new();
    let mut side = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower == "#main" {
            current_section = Some(Section::Main);
            continue;
        } else if lower == "#extra" {
            current_section = Some(Section::Extra);
            continue;
        } else if lower == "!side" || lower == "#side" {
            current_section = Some(Section::Side);
            continue;
        } else if trimmed.starts_with('#') || trimmed.starts_with('!') || trimmed.starts_with('$') {
            // Comment or metadata line like "#created by ...", "$whitelist", etc.
            continue;
        }

        if let Some(section) = current_section {
            // Extract the first token / number on the line
            let token = trimmed.split_whitespace().next().unwrap_or(trimmed);
            if let Ok(card_id) = token.parse::<u32>() {
                if card_id > 0 {
                    match section {
                        Section::Main => main.push(card_id),
                        Section::Extra => extra.push(card_id),
                        Section::Side => side.push(card_id),
                    }
                }
            }
        }
    }

    if main.is_empty() && extra.is_empty() && side.is_empty() {
        return Err(DeckError::EmptyDeckOrInvalidFormat);
    }

    Ok(YdkeDeck { main, extra, side })
}

/// Parses an XML deck format containing `<main>`, `<extra>`, `<side>`, and `<card passcode="...">` tags.
pub fn parse_xml(input: &str) -> Result<YdkeDeck, DeckError> {
    #[derive(Copy, Clone, PartialEq, Eq)]
    enum Section {
        Main,
        Extra,
        Side,
    }

    let mut current_section: Option<Section> = None;
    let mut main = Vec::new();
    let mut extra = Vec::new();
    let mut side = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if lower.contains("<main>") || lower.starts_with("<main ") || lower == "<main>" {
            current_section = Some(Section::Main);
        }
        if lower.contains("</main>") {
            current_section = None;
        }
        if lower.contains("<extra>") || lower.starts_with("<extra ") || lower == "<extra>" {
            current_section = Some(Section::Extra);
        }
        if lower.contains("</extra>") {
            current_section = None;
        }
        if lower.contains("<side>") || lower.starts_with("<side ") || lower == "<side>" {
            current_section = Some(Section::Side);
        }
        if lower.contains("</side>") {
            current_section = None;
        }

        if lower.contains("<card") {
            if let Some(section) = current_section {
                // Look for passcode attribute first, then id
                if let Some(card_id) = extract_xml_attribute(trimmed, "passcode")
                    .or_else(|| extract_xml_attribute(trimmed, "id"))
                {
                    match section {
                        Section::Main => main.push(card_id),
                        Section::Extra => extra.push(card_id),
                        Section::Side => side.push(card_id),
                    }
                }
            }
        }
    }

    if main.is_empty() && extra.is_empty() && side.is_empty() {
        return Err(DeckError::EmptyDeckOrInvalidFormat);
    }

    Ok(YdkeDeck { main, extra, side })
}

fn extract_xml_attribute(line: &str, attr: &str) -> Option<u32> {
    let pattern = format!("{attr}=\"");
    let pattern_single = format!("{attr}='");

    let val_str = if let Some(idx) = line.to_ascii_lowercase().find(&pattern) {
        let start = idx + pattern.len();
        let end = line[start..].find('"')? + start;
        &line[start..end]
    } else if let Some(idx) = line.to_ascii_lowercase().find(&pattern_single) {
        let start = idx + pattern_single.len();
        let end = line[start..].find('\'')? + start;
        &line[start..end]
    } else {
        return None;
    };

    val_str.trim().parse::<u32>().ok()
}

/// Automatically detects and parses a deck from YDKE, YDK, or XML input.
pub fn parse_deck(input: &str) -> Result<YdkeDeck, DeckError> {
    let trimmed = input.trim();
    if trimmed.contains("ydke://") {
        parse_ydke(trimmed)
    } else if trimmed.to_ascii_lowercase().contains("<deck")
        || trimmed.to_ascii_lowercase().contains("<main>")
        || trimmed.to_ascii_lowercase().contains("<card ")
        || trimmed.to_ascii_lowercase().contains("<card>")
    {
        parse_xml(trimmed)
    } else if trimmed.to_ascii_lowercase().contains("#main")
        || trimmed.to_ascii_lowercase().contains("#extra")
        || trimmed.to_ascii_lowercase().contains("!side")
        || trimmed.to_ascii_lowercase().contains("#side")
    {
        parse_ydk(trimmed)
    } else {
        // Fallback trial in order
        parse_ydke(trimmed)
            .or_else(|_| parse_ydk(trimmed))
            .or_else(|_| parse_xml(trimmed))
    }
}

/// Decodes a base64 string into a list of 32-bit little-endian card IDs.
fn decode_card_ids(b64_str: &str) -> Result<Vec<u32>, DeckError> {
    if b64_str.is_empty() {
        return Ok(Vec::new());
    }

    // Try standard base64 and URL safe variants, with or without padding
    let decoded = STANDARD
        .decode(b64_str)
        .or_else(|_| STANDARD_NO_PAD.decode(b64_str))
        .or_else(|_| URL_SAFE.decode(b64_str))
        .or_else(|_| URL_SAFE_NO_PAD.decode(b64_str))
        .map_err(|e| DeckError::Base64DecodeError(e.to_string()))?;

    if decoded.len() % 4 != 0 {
        return Err(DeckError::InvalidByteLength(decoded.len()));
    }

    let card_ids = decoded
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(card_ids)
}

/// Options and layout configuration for rendering a deck image.
pub struct DeckRenderConfig {
    pub card_images_dir: PathBuf,
    pub main_cards_per_row: u32,
    pub main_rows: u32,
    pub extra_side_cards_per_row: u32,
    pub main_card_width: u32,
    pub main_card_height: u32,
}

impl Default for DeckRenderConfig {
    fn default() -> Self {
        let card_images_dir = std::env::var("CARD_IMAGES_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/app/pics"));

        // Base main card dimensions matching standard Yu-Gi-Oh card ratio (~177x254)
        // With main_card_width = 120 and height = 172:
        // Main deck width (10 cards) = 1200px.
        // Extra/Side deck width (15 cards) = 15 * 80px = 1200px.
        // Extra/Side card size = 80 x 115 (same aspect ratio).
        Self {
            card_images_dir,
            main_cards_per_row: 10,
            main_rows: 6,
            extra_side_cards_per_row: 15,
            main_card_width: 120,
            main_card_height: 172,
        }
    }
}

/// Renders a compound image of the given YDKE deck.
pub fn render_deck(deck: &YdkeDeck, config: &DeckRenderConfig) -> Result<RgbaImage, Box<dyn std::error::Error + Send + Sync>> {
    let main_w = config.main_card_width;
    let main_h = config.main_card_height;
    
    // Width of the full deck board: 10 * main_card_width
    let board_w = config.main_cards_per_row * main_w;
    
    // Calculate width and height of extra/side cards to fit 15 per row across board_w
    let small_w = board_w / config.extra_side_cards_per_row;
    let small_h = (main_h * small_w) / main_w;

    let extra_rows = if deck.extra.is_empty() {
        1
    } else {
        ((deck.extra.len() as u32).saturating_sub(1) / config.extra_side_cards_per_row) + 1
    };

    let side_rows = if deck.side.is_empty() {
        1
    } else {
        ((deck.side.len() as u32).saturating_sub(1) / config.extra_side_cards_per_row) + 1
    };

    let separator_h = 4u32;
    let main_section_h = config.main_rows * main_h;
    let side_section_h = side_rows * small_h;
    let extra_section_h = extra_rows * small_h;

    let total_h = main_section_h + separator_h + side_section_h + separator_h + extra_section_h;

    // Background color: dark purple/slate matching deck builder canvas
    let bg_color = Rgba([28, 20, 36, 255]);
    let mut canvas: RgbaImage = ImageBuffer::from_pixel(board_w, total_h, bg_color);

    // Draw Main Deck (6 rows of 10 cards = 60 slots)
    let total_main_slots = config.main_rows * config.main_cards_per_row;
    for slot_idx in 0..total_main_slots {
        let row = slot_idx / config.main_cards_per_row;
        let col = slot_idx % config.main_cards_per_row;
        let x = col * main_w;
        let y = row * main_h;

        if let Some(&card_id) = deck.main.get(slot_idx as usize) {
            draw_card_or_placeholder(&mut canvas, &config.card_images_dir, card_id, x, y, main_w, main_h);
        } else {
            draw_empty_slot(&mut canvas, x, y, main_w, main_h, Rgba([48, 36, 62, 255]), Rgba([80, 60, 100, 255]));
        }
    }

    // Draw Side Deck separator (cyan tint bar)
    let side_y_start = main_section_h;
    draw_separator(&mut canvas, 0, side_y_start, board_w, separator_h, Rgba([0, 190, 220, 255]));

    // Draw Side Deck (15 cards per row)
    let side_cards_y = side_y_start + separator_h;
    let total_side_slots = side_rows * config.extra_side_cards_per_row;
    for slot_idx in 0..total_side_slots {
        let row = slot_idx / config.extra_side_cards_per_row;
        let col = slot_idx % config.extra_side_cards_per_row;
        let x = col * small_w;
        let y = side_cards_y + row * small_h;

        if let Some(&card_id) = deck.side.get(slot_idx as usize) {
            draw_card_or_placeholder(&mut canvas, &config.card_images_dir, card_id, x, y, small_w, small_h);
        } else {
            draw_empty_slot(&mut canvas, x, y, small_w, small_h, Rgba([30, 48, 60, 255]), Rgba([50, 90, 110, 255]));
        }
    }

    // Draw Extra Deck separator (orange tint bar)
    let extra_y_start = side_cards_y + side_section_h;
    draw_separator(&mut canvas, 0, extra_y_start, board_w, separator_h, Rgba([230, 110, 20, 255]));

    // Draw Extra Deck (15 cards per row)
    let extra_cards_y = extra_y_start + separator_h;
    let total_extra_slots = extra_rows * config.extra_side_cards_per_row;
    for slot_idx in 0..total_extra_slots {
        let row = slot_idx / config.extra_side_cards_per_row;
        let col = slot_idx % config.extra_side_cards_per_row;
        let x = col * small_w;
        let y = extra_cards_y + row * small_h;

        if let Some(&card_id) = deck.extra.get(slot_idx as usize) {
            draw_card_or_placeholder(&mut canvas, &config.card_images_dir, card_id, x, y, small_w, small_h);
        } else {
            draw_empty_slot(&mut canvas, x, y, small_w, small_h, Rgba([48, 32, 24, 255]), Rgba([100, 60, 40, 255]));
        }
    }

    Ok(canvas)
}

/// Renders the deck and encodes the resulting compound image as PNG bytes.
pub fn generate_deck_image_bytes(deck: &YdkeDeck, config: &DeckRenderConfig) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let img = render_deck(deck, config)?;
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    Ok(bytes)
}

/// Automatically parses a deck from any supported format (YDKE, YDK, XML) and generates the compound image PNG bytes.
pub fn process_deck_input(input: &str) -> Result<(YdkeDeck, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let deck = parse_deck(input)?;
    let config = DeckRenderConfig::default();
    let image_bytes = generate_deck_image_bytes(&deck, &config)?;
    Ok((deck, image_bytes))
}

/// Scans, validates, transforms a YDKE URL and generates the compound image PNG bytes.
pub fn process_ydke(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let deck = parse_ydke(input)?;
    let config = DeckRenderConfig::default();
    generate_deck_image_bytes(&deck, &config)
}

fn draw_separator(canvas: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for cy in y..(y + h).min(canvas.height()) {
        for cx in x..(x + w).min(canvas.width()) {
            canvas.put_pixel(cx, cy, color);
        }
    }
}

fn draw_empty_slot(canvas: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, bg: Rgba<u8>, border: Rgba<u8>) {
    let padding = 1u32;
    for cy in 0..h {
        for cx in 0..w {
            let px = x + cx;
            let py = y + cy;
            if px >= canvas.width() || py >= canvas.height() {
                continue;
            }

            let is_border = cx < padding || cx >= w - padding || cy < padding || cy >= h - padding;
            if is_border {
                canvas.put_pixel(px, py, border);
            } else {
                canvas.put_pixel(px, py, bg);
            }
        }
    }
}

fn draw_card_or_placeholder(
    canvas: &mut RgbaImage,
    pics_dir: &Path,
    card_id: u32,
    x: u32,
    y: u32,
    target_w: u32,
    target_h: u32,
) {
    let card_file = pics_dir.join(format!("{card_id}.jpg"));
    let card_img = image::open(&card_file).ok();

    if let Some(img) = card_img {
        let resized = img.resize_exact(target_w, target_h, FilterType::Triangle);
        let resized_rgba = resized.to_rgba8();
        for cy in 0..target_h {
            for cx in 0..target_w {
                let px = x + cx;
                let py = y + cy;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, *resized_rgba.get_pixel(cx, cy));
                }
            }
        }
    } else {
        // Fallback placeholder with card border
        draw_empty_slot(
            canvas,
            x,
            y,
            target_w,
            target_h,
            Rgba([60, 60, 70, 255]),
            Rgba([140, 140, 160, 255]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ydke_url() {
        let input = "Check out this deck: ydke://YIAAAA==!YIAAAA!YIAAAA==! and tell me what you think";
        let extracted = extract_ydke_url(input).expect("should extract ydke URL");
        assert_eq!(extracted, "ydke://YIAAAA==!YIAAAA!YIAAAA==!");
    }

    #[test]
    fn test_parse_ydke_single_card() {
        // 32864 in little endian = [0x60, 0x80, 0x00, 0x00] -> base64 is "YIAAAA==" or unpadded "YIAAAA"
        let ydke_url = "ydke://YIAAAA==!YIAAAA!YIAAAA==!";
        let deck = parse_ydke(ydke_url).expect("should parse valid ydke");
        assert_eq!(deck.main, vec![32864]);
        assert_eq!(deck.extra, vec![32864]);
        assert_eq!(deck.side, vec![32864]);
    }

    #[test]
    fn test_parse_ydke_multiple_cards() {
        let card1: u32 = 32864;
        let card2: u32 = 46986420;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&card1.to_le_bytes());
        bytes.extend_from_slice(&card2.to_le_bytes());
        let b64 = STANDARD.encode(&bytes);

        let ydke_url = format!("ydke://{b64}!YIAAAA==!YIAAAA==!");
        let deck = parse_ydke(&ydke_url).expect("should parse valid ydke with multiple cards");
        assert_eq!(deck.main, vec![card1, card2]);
        assert_eq!(deck.extra, vec![card1]);
        assert_eq!(deck.side, vec![card1]);
    }

    #[test]
    fn test_parse_ydke_empty_sections() {
        let ydke_url = "ydke://!!";
        let deck = parse_ydke(ydke_url).expect("should parse empty ydke");
        assert!(deck.main.is_empty());
        assert!(deck.extra.is_empty());
        assert!(deck.side.is_empty());
    }

    #[test]
    fn test_parse_ydke_invalid_scheme() {
        let err = parse_ydke("http://example.com/test").unwrap_err();
        match err {
            YdkeError::InvalidScheme => (),
            _ => panic!("Expected InvalidScheme error, got {:?}", err),
        }
    }

    #[test]
    fn test_process_ydke_end_to_end() {
        let ydke_url = "ydke://YIAAAA==!YIAAAA==!YIAAAA==!";
        let bytes = process_ydke(ydke_url).expect("should process ydke to png bytes");
        assert!(!bytes.is_empty());
        // Verify it starts with PNG header [137, 80, 78, 71, 13, 10, 26, 10]
        assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_to_ydke_url() {
        let deck = YdkeDeck {
            main: vec![32864],
            extra: vec![32864],
            side: vec![32864],
        };
        let encoded = deck.to_ydke_url();
        let parsed = parse_ydke(&encoded).expect("should parse re-encoded ydke url");
        assert_eq!(deck, parsed);
    }

    #[test]
    fn test_parse_ydk_format() {
        let ydk_content = r#"
#created by Mermail Player
#main
74311226
37104630
0706925
#extra
26593852
29071332
!side
40916023
23434538
"#;
        let deck = parse_ydk(ydk_content).expect("should parse ydk format");
        assert_eq!(deck.main, vec![74311226, 37104630, 706925]);
        assert_eq!(deck.extra, vec![26593852, 29071332]);
        assert_eq!(deck.side, vec![40916023, 23434538]);

        // Test auto-detect parse_deck
        let auto_deck = parse_deck(ydk_content).expect("should auto-detect ydk");
        assert_eq!(deck, auto_deck);
    }

    #[test]
    fn test_parse_xml_format() {
        let xml_content = r#"
<deck name="wizkalina-Mermail">
<main>
<card id="5543" passcode="74311226">Atlantean Dragoons</card>
<card id="5532" passcode="37104630">Atlantean Heavy Infantry</card>
<card id="5531" passcode="0706925">Atlantean Marksman</card>
</main>
<side>
<card id="196" passcode="40916023">Aqua Spirit</card>
<card id="2797" passcode="23434538">Maxx "C"</card>
</side>
<extra>
<card id="85" passcode="26593852">Ally of Justice Catastor</card>
<card id="253" passcode="29071332">Armory Arm</card>
</extra>
</deck>
"#;
        let deck = parse_xml(xml_content).expect("should parse xml format");
        assert_eq!(deck.main, vec![74311226, 37104630, 706925]);
        assert_eq!(deck.extra, vec![26593852, 29071332]);
        assert_eq!(deck.side, vec![40916023, 23434538]);

        // Test auto-detect parse_deck
        let auto_deck = parse_deck(xml_content).expect("should auto-detect xml");
        assert_eq!(deck, auto_deck);
    }

    #[test]
    fn test_process_deck_input_xml_and_ydk() {
        let xml_content = r#"
<deck name="test">
<main>
<card id="1" passcode="32864">The 13th Grave</card>
</main>
<extra></extra>
<side></side>
</deck>
"#;
        let (deck, bytes) = process_deck_input(xml_content).expect("should process xml deck");
        assert_eq!(deck.main, vec![32864]);
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_deck_error_display() {
        assert_eq!(
            DeckError::InvalidScheme.to_string(),
            "YDKE URL must start with 'ydke://'"
        );
        assert_eq!(
            DeckError::InvalidSections.to_string(),
            "YDKE URL must contain main, extra, and side deck sections"
        );
        assert_eq!(
            DeckError::Base64DecodeError("err".into()).to_string(),
            "Failed to decode base64: err"
        );
        assert_eq!(
            DeckError::InvalidByteLength(5).to_string(),
            "Decoded byte length (5) is not a multiple of 4"
        );
        assert_eq!(
            DeckError::EmptyDeckOrInvalidFormat.to_string(),
            "No valid cards found in input (invalid YDKE, YDK, or XML format)"
        );
        assert_eq!(
            DeckError::RenderError("render failed".into()).to_string(),
            "Failed to render deck image: render failed"
        );
    }

    #[test]
    fn test_extract_ydke_url_none() {
        assert!(extract_ydke_url("no ydke link here").is_none());
        assert!(extract_ydke_url("").is_none());
    }

    #[test]
    fn test_extract_ydke_url_whitespace_and_newlines() {
        let text = "  \n  ydke://YIAAAA==!YIAAAA!YIAAAA==!  \n some trailing text ";
        let url = extract_ydke_url(text).expect("should extract url with whitespace");
        assert_eq!(url, "ydke://YIAAAA==!YIAAAA!YIAAAA==!");
    }

    #[test]
    fn test_parse_ydke_invalid_sections() {
        let err = parse_ydke("ydke://onlyone").unwrap_err();
        match err {
            DeckError::InvalidSections => (),
            other => panic!("Expected InvalidSections, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ydke_invalid_base64() {
        let err = parse_ydke("ydke://???invalid???!!!").unwrap_err();
        match err {
            DeckError::Base64DecodeError(_) => (),
            other => panic!("Expected Base64DecodeError, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ydke_invalid_byte_length() {
        // "AQID" decodes to 3 bytes [1, 2, 3], which is not divisible by 4
        let err = parse_ydke("ydke://AQID!!!").unwrap_err();
        match err {
            DeckError::InvalidByteLength(3) => (),
            other => panic!("Expected InvalidByteLength(3), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ydke_url_safe_base64() {
        // Generate bytes with high bits producing '-' and '_' in URL_SAFE base64
        let card: u32 = 0xFBFF_FEFD;
        let bytes = card.to_le_bytes();
        let b64 = URL_SAFE_NO_PAD.encode(&bytes);
        let ydke_url = format!("ydke://{b64}!!!",);
        let deck = parse_ydke(&ydke_url).expect("should parse url-safe base64");
        assert_eq!(deck.main, vec![card]);
    }

    #[test]
    fn test_parse_ydk_with_hash_side_and_comments() {
        let ydk = r#"
#created by test
$whitelist
#main
87910978 0 --Brain Control
#extra
50321796 --Brionac
#side
1475311 1 --Allure of Darkness
"#;
        let deck = parse_ydk(ydk).expect("should parse ydk with #side and comments");
        assert_eq!(deck.main, vec![87910978]);
        assert_eq!(deck.extra, vec![50321796]);
        assert_eq!(deck.side, vec![1475311]);
    }

    #[test]
    fn test_parse_ydk_empty_or_invalid() {
        let ydk = "#created by nobody\n\n";
        let err = parse_ydk(ydk).unwrap_err();
        match err {
            DeckError::EmptyDeckOrInvalidFormat => (),
            other => panic!("Expected EmptyDeckOrInvalidFormat, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_xml_single_quotes_and_id_fallback() {
        let xml = r#"
<deck>
  <main>
    <card passcode='74311226'>Atlantean Dragoons</card>
    <card id="37104630">Atlantean Heavy Infantry</card>
  </main>
  <side>
    <card id='40916023'>Aqua Spirit</card>
  </side>
  <extra>
    <card passcode="26593852">Catastor</card>
  </extra>
</deck>
"#;
        let deck = parse_xml(xml).expect("should parse xml with quotes and id fallback");
        assert_eq!(deck.main, vec![74311226, 37104630]);
        assert_eq!(deck.side, vec![40916023]);
        assert_eq!(deck.extra, vec![26593852]);
    }

    #[test]
    fn test_parse_xml_empty_fails() {
        let xml = "<deck><main></main><extra></extra><side></side></deck>";
        let err = parse_xml(xml).unwrap_err();
        match err {
            DeckError::EmptyDeckOrInvalidFormat => (),
            other => panic!("Expected EmptyDeckOrInvalidFormat, got {:?}", other),
        }
    }

    #[test]
    fn test_deck_render_config_default() {
        let config = DeckRenderConfig::default();
        assert_eq!(config.main_cards_per_row, 10);
        assert_eq!(config.main_rows, 6);
        assert_eq!(config.extra_side_cards_per_row, 15);
        assert_eq!(config.main_card_width, 120);
        assert_eq!(config.main_card_height, 172);
    }

    #[test]
    fn test_render_deck_multi_row_extra_and_side() {
        let config = DeckRenderConfig::default();
        // 16 cards in extra deck -> 2 rows in extra deck
        // 31 cards in side deck -> 3 rows in side deck
        let extra_cards = vec![32864; 16];
        let side_cards = vec![32864; 31];
        let deck = YdkeDeck {
            main: vec![32864; 60],
            extra: extra_cards,
            side: side_cards,
        };

        let canvas = render_deck(&deck, &config).expect("should render deck with multiple rows");
        assert_eq!(canvas.width(), 1200);

        // Expected height calculation:
        // main: 6 * 172 = 1032
        // sep 1: 4
        // side: 3 * (172 * 80 / 120) = 3 * 114 = 342
        // sep 2: 4
        // extra: 2 * 114 = 228
        // total: 1032 + 4 + 342 + 4 + 228 = 1610
        assert_eq!(canvas.height(), 1610);
    }
}
