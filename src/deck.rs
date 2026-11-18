use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use image::imageops::FilterType;
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Represents a parsed Yu-Gi-Oh! deck from a YDKE link.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct YdkeDeck {
    pub main: Vec<u32>,
    pub extra: Vec<u32>,
    pub side: Vec<u32>,
}

#[derive(Debug)]
pub enum YdkeError {
    InvalidScheme,
    InvalidSections,
    Base64DecodeError(String),
    InvalidByteLength(usize),
}

impl std::fmt::Display for YdkeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YdkeError::InvalidScheme => write!(f, "YDKE URL must start with 'ydke://'"),
            YdkeError::InvalidSections => write!(f, "YDKE URL must contain main, extra, and side deck sections"),
            YdkeError::Base64DecodeError(e) => write!(f, "Failed to decode base64: {e}"),
            YdkeError::InvalidByteLength(len) => write!(f, "Decoded byte length ({len}) is not a multiple of 4"),
        }
    }
}

impl std::error::Error for YdkeError {}

/// Extracts the YDKE URL string from an input text.
pub fn extract_ydke_url(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    let ydke_start = trimmed.find("ydke://")?;
    let ydke_str = &trimmed[ydke_start..];
    let ydke_url = ydke_str.split_whitespace().next().unwrap_or(ydke_str);
    Some(ydke_url)
}

/// Scans and extracts a YDKE URL from a text query/message and parses it into `YdkeDeck`.
pub fn parse_ydke(input: &str) -> Result<YdkeDeck, YdkeError> {
    let ydke_url = extract_ydke_url(input).ok_or(YdkeError::InvalidScheme)?;
    let ydke_content = &ydke_url["ydke://".len()..];

    // YDKE sections are separated by '!'
    // Example: <main_b64>!<extra_b64>!<side_b64>!
    let parts: Vec<&str> = ydke_content.split('!').collect();
    if parts.len() < 3 {
        return Err(YdkeError::InvalidSections);
    }

    let main = decode_card_ids(parts[0])?;
    let extra = decode_card_ids(parts[1])?;
    let side = decode_card_ids(parts[2])?;

    Ok(YdkeDeck { main, extra, side })
}

/// Decodes a base64 string into a list of 32-bit little-endian card IDs.
fn decode_card_ids(b64_str: &str) -> Result<Vec<u32>, YdkeError> {
    if b64_str.is_empty() {
        return Ok(Vec::new());
    }

    // Try standard base64 and URL safe variants, with or without padding
    let decoded = STANDARD
        .decode(b64_str)
        .or_else(|_| STANDARD_NO_PAD.decode(b64_str))
        .or_else(|_| URL_SAFE.decode(b64_str))
        .or_else(|_| URL_SAFE_NO_PAD.decode(b64_str))
        .map_err(|e| YdkeError::Base64DecodeError(e.to_string()))?;

    if decoded.len() % 4 != 0 {
        return Err(YdkeError::InvalidByteLength(decoded.len()));
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
            .unwrap_or_else(|_| PathBuf::from("/home/vince/Programme/ProjectIgnis/pics"));

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
}
