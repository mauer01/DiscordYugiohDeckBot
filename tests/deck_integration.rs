use discord_yugioh_deck_bot::commands::format_ydke_codeblock;
use discord_yugioh_deck_bot::deck::{
    parse_deck, parse_xml, parse_ydk, parse_ydke, process_deck_input, DeckRenderConfig,
};

/// The YDKE link provided in the task prompt (Elemental HERO deck)
const HERO_YDKE: &str = "ydke://obsGBYc9gwNgUk0BAlkqBAJZKgQCWSoEdgljApJkxASSZMQEiQq7AokKuwKJCrsCTkEDAE5BAwB0RysBppv0AeOwKgNZe2MEWXtjBFl7YwTjNuEF4zbhBcFzBALBcwQCwXMEAl1p3gJdad4CXWneAmbx/wFm8f8BZvH/AV6jwAFeo8ABEA1iAhANYgIQDWICEtmgAhLZoAL7mjED+5oxAyJyzQEics0BInLNAfsEeAIALg0F!9r+/Afa/vwH2v78BjZQ3ACuvHQBkoVABlk8AArVibwL0yfgA9Mn4AFdhZANXYWQDV2FkA9IbQQE=!3PtDBNz7QwTBV6cFLJduBSqVZQEqlWUBKpVlAXRHKwHvJ1EA1fbWAHtkHQJ7ZB0Ce2QdAg==!";

/// The YDK text provided in the task prompt (Mermail deck)
const MERMAIL_YDK: &str = r#"#created by ...
#main
74311226
74311226
74311226
37104630
37104630
706925
706925
706925
78868119
78868119
78868119
23434538
23434538
69293721
69293721
37781520
23899727
23899727
23899727
21954587
21954587
58471134
58471134
22446869
22446869
22446869
22076135
13959634
53129443
19613556
83764718
5318639
5318639
5318639
67169062
60202749
60202749
60202749
78474168
53582587
53582587
#extra
26593852
29071332
73580471
70583986
65749035
90953320
21044178
440556
2766877
10002346
12014404
74371660
59170782
80117527
69610924
!side
40916023
40916023
23434538
93920745
93920745
91133740
37520316
65703851
65703851
70342110
60082869
60082869
60082869
15800838
15800838
"#;

/// The XML deck provided in the task prompt (Mermail deck)
const MERMAIL_XML: &str = r#"<deck name="wizkalina-Mermail">
<main>
<card id="5543" passcode="74311226">Atlantean Dragoons</card>
<card id="5543" passcode="74311226">Atlantean Dragoons</card>
<card id="5543" passcode="74311226">Atlantean Dragoons</card>
<card id="5532" passcode="37104630">Atlantean Heavy Infantry</card>
<card id="5532" passcode="37104630">Atlantean Heavy Infantry</card>
<card id="5531" passcode="0706925">Atlantean Marksman</card>
<card id="5531" passcode="0706925">Atlantean Marksman</card>
<card id="5531" passcode="0706925">Atlantean Marksman</card>
<card id="1079" passcode="78868119">Deep Sea Diva</card>
<card id="1079" passcode="78868119">Deep Sea Diva</card>
<card id="1079" passcode="78868119">Deep Sea Diva</card>
<card id="2797" passcode="23434538">Maxx "C"</card>
<card id="2797" passcode="23434538">Maxx "C"</card>
<card id="5547" passcode="69293721">Mermail Abyssgunde</card>
<card id="5547" passcode="69293721">Mermail Abyssgunde</card>
<card id="5734" passcode="37781520">Mermail Abyssleed</card>
<card id="5568" passcode="23899727">Mermail Abysslinde</card>
<card id="5568" passcode="23899727">Mermail Abysslinde</card>
<card id="5568" passcode="23899727">Mermail Abysslinde</card>
<card id="5550" passcode="21954587">Mermail Abyssmegalo</card>
<card id="5550" passcode="21954587">Mermail Abyssmegalo</card>
<card id="5545" passcode="58471134">Mermail Abysspike</card>
<card id="5545" passcode="58471134">Mermail Abysspike</card>
<card id="5804" passcode="22446869">Mermail Abyssteus</card>
<card id="5804" passcode="22446869">Mermail Abyssteus</card>
<card id="5804" passcode="22446869">Mermail Abyssteus</card>
<card id="5546" passcode="22076135">Mermail Abyssturge</card>
<card id="5576" passcode="13959634">Moulinglacia the Elemental Lord</card>
<card id="989" passcode="53129443">Dark Hole</card>
<card id="2116" passcode="19613556">Heavy Storm</card>
<card id="2952" passcode="83764718">Monster Reborn</card>
<card id="3043" passcode="05318639">Mystical Space Typhoon</card>
<card id="3043" passcode="05318639">Mystical Space Typhoon</card>
<card id="3043" passcode="05318639">Mystical Space Typhoon</card>
<card id="3342" passcode="67169062">Pot of Avarice</card>
<card id="5553" passcode="60202749">Abyss-sphere</card>
<card id="5553" passcode="60202749">Abyss-sphere</card>
<card id="5553" passcode="60202749">Abyss-sphere</card>
<card id="5749" passcode="78474168">Breakthrough Skill</card>
<card id="4429" passcode="53582587">Torrential Tribute</card>
<card id="4429" passcode="53582587">Torrential Tribute</card>
</main>
<side>
<card id="196" passcode="40916023">Aqua Spirit</card>
<card id="196" passcode="40916023">Aqua Spirit</card>
<card id="2797" passcode="23434538">Maxx "C"</card>
<card id="3276" passcode="93920745">Penguin Soldier</card>
<card id="3276" passcode="93920745">Penguin Soldier</card>
<card id="3904" passcode="91133740">Snowman Eater</card>
<card id="2875" passcode="37520316">Mind Control</card>
<card id="1071" passcode="65703851">Debunk</card>
<card id="1071" passcode="65703851">Debunk</card>
<card id="1180" passcode="70342110">Dimensional Prison</card>
<card id="1311" passcode="60082869">Dust Tornado</card>
<card id="1311" passcode="60082869">Dust Tornado</card>
<card id="1311" passcode="60082869">Dust Tornado</card>
<card id="2876" passcode="15800838">Mind Crush</card>
<card id="2876" passcode="15800838">Mind Crush</card>
</side>
<extra>
<card id="85" passcode="26593852">Ally of Justice Catastor</card>
<card id="253" passcode="29071332">Armory Arm</card>
<card id="430" passcode="73580471">Black Rose Dragon</card>
<card id="1155" passcode="70583986">Dewloren, Tiger King of the Ice Barrier</card>
<card id="2057" passcode="65749035">Gungnir, Dragon of the Ice Barrier</card>
<card id="4191" passcode="90953320">T.G. Hyper Librarian</card>
<card id="5684" passcode="21044178">Abyss Dweller</card>
<card id="5632" passcode="0440556">Bahamut Shark</card>
<card id="5117" passcode="02766877">Daigusto Phoenix</card>
<card id="5098" passcode="10002346">Gachi Gachi Gantetsu</card>
<card id="5622" passcode="12014404">Gagaga Cowboy</card>
<card id="5549" passcode="74371660">Mermail Abyssgaios</card>
<card id="5730" passcode="59170782">Mermail Abysstrite</card>
<card id="5124" passcode="80117527">Number 11: Big Eye</card>
<card id="5096" passcode="69610924">Number 17: Leviathan Dragon</card>
</extra>
</deck>
"#;

#[test]
fn test_user_requests_ydke_hero_deck() {
    let (deck, img_bytes) = process_deck_input(HERO_YDKE).expect("should parse HERO YDKE");

    // Main: 45 cards, Extra: 14 cards, Side: 13 cards
    assert_eq!(deck.main.len(), 45);
    assert_eq!(deck.extra.len(), 14);
    assert_eq!(deck.side.len(), 13);

    // Verify card IDs from YDKE
    assert_eq!(deck.main[0], 84327329); // Elemental HERO Clayman
    assert_eq!(deck.main[1], 58932615); // Elemental HERO Burstinatrix
    assert_eq!(deck.main[2], 21844576); // Elemental HERO Avian

    // Re-encoding matches canonical URL
    let generated_url = deck.to_ydke_url();
    assert_eq!(generated_url, HERO_YDKE);

    let codeblock = format_ydke_codeblock(&generated_url);
    assert_eq!(codeblock, format!("```\n{HERO_YDKE}\n```"));

    // Verify image bytes
    assert!(!img_bytes.is_empty());
    assert_eq!(&img_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

    // Verify canvas dimensions
    let config = DeckRenderConfig::default();
    let img = discord_yugioh_deck_bot::deck::render_deck(&deck, &config).expect("render hero deck");
    assert_eq!(img.width(), 1200);
    // 6 rows main + 1 row side + 1 row extra + 2 separators = 1268
    assert_eq!(img.height(), 1268);
}

#[test]
fn test_mermail_deck_integration_ydk_and_xml_consistency() {
    // 1. Parse via explicit format parsers
    let deck_from_ydk = parse_ydk(MERMAIL_YDK).expect("should parse Mermail YDK");
    let deck_from_xml = parse_xml(MERMAIL_XML).expect("should parse Mermail XML");

    // Both parsed decks must have identical contents
    assert_eq!(deck_from_ydk, deck_from_xml);

    // 2. Validate card counts (41 Main, 15 Extra, 15 Side) matching prompt screenshot
    assert_eq!(deck_from_ydk.main.len(), 41);
    assert_eq!(deck_from_ydk.extra.len(), 15);
    assert_eq!(deck_from_ydk.side.len(), 15);

    // Validate specific expected cards in Main deck
    assert_eq!(&deck_from_ydk.main[0..3], &[74311226, 74311226, 74311226]); // Atlantean Dragoons x3
    assert_eq!(&deck_from_ydk.main[3..5], &[37104630, 37104630]); // Atlantean Heavy Infantry x2
    assert_eq!(&deck_from_ydk.main[5..8], &[706925, 706925, 706925]); // Atlantean Marksman x3
    assert_eq!(&deck_from_ydk.main[8..11], &[78868119, 78868119, 78868119]); // Deep Sea Diva x3
    assert_eq!(&deck_from_ydk.main[39..41], &[53582587, 53582587]); // Torrential Tribute x2

    // Validate specific expected cards in Extra deck
    assert_eq!(deck_from_ydk.extra[0], 26593852); // Catastor
    assert_eq!(deck_from_ydk.extra[1], 29071332); // Armory Arm
    assert_eq!(deck_from_ydk.extra[14], 69610924); // Number 17: Leviathan Dragon

    // Validate specific expected cards in Side deck
    assert_eq!(&deck_from_ydk.side[0..2], &[40916023, 40916023]); // Aqua Spirit x2
    assert_eq!(&deck_from_ydk.side[13..15], &[15800838, 15800838]); // Mind Crush x2

    // 3. Parse via automatic detector parse_deck
    let auto_ydk = parse_deck(MERMAIL_YDK).expect("auto parse_deck YDK");
    let auto_xml = parse_deck(MERMAIL_XML).expect("auto parse_deck XML");
    assert_eq!(auto_ydk, deck_from_ydk);
    assert_eq!(auto_xml, deck_from_ydk);

    // 4. Verify canonical re-serialization to YDKE URL and parse back
    let mermail_ydke_url = deck_from_ydk.to_ydke_url();
    assert!(mermail_ydke_url.starts_with("ydke://"));
    let deck_from_mermail_ydke = parse_ydke(&mermail_ydke_url).expect("parse generated Mermail YDKE");
    assert_eq!(deck_from_mermail_ydke, deck_from_ydk);

    // 5. Verify format_ydke_codeblock output
    let codeblock = format_ydke_codeblock(&mermail_ydke_url);
    assert_eq!(codeblock, format!("```\n{mermail_ydke_url}\n```"));

    // 6. Verify compound image generation produces identical PNG bytes for YDK, XML, and generated YDKE
    let (deck1, img_bytes_ydk) = process_deck_input(MERMAIL_YDK).expect("process YDK");
    let (deck2, img_bytes_xml) = process_deck_input(MERMAIL_XML).expect("process XML");
    let (deck3, img_bytes_ydke) = process_deck_input(&mermail_ydke_url).expect("process YDKE");

    assert_eq!(deck1, deck_from_ydk);
    assert_eq!(deck2, deck_from_ydk);
    assert_eq!(deck3, deck_from_ydk);

    // All three formats produce identical rendered images
    assert_eq!(img_bytes_ydk, img_bytes_xml);
    assert_eq!(img_bytes_ydk, img_bytes_ydke);
    assert!(!img_bytes_ydk.is_empty());
    assert_eq!(&img_bytes_ydk[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

    // 7. Verify rendered image canvas dimensions matching the screenshot layout
    let config = DeckRenderConfig::default();
    let img = discord_yugioh_deck_bot::deck::render_deck(&deck_from_ydk, &config)
        .expect("should render deck canvas");

    // Board width is 10 * 120 = 1200
    assert_eq!(img.width(), 1200);

    // Height calculation:
    // Main deck: 6 rows * 172 = 1032
    // Separator 1: 4
    // Side deck: 1 row (15 cards) * 114 = 114
    // Separator 2: 4
    // Extra deck: 1 row (15 cards) * 114 = 114
    // Total height = 1032 + 4 + 114 + 4 + 114 = 1268
    assert_eq!(img.height(), 1268);
}
