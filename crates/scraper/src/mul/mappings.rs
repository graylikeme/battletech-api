use std::collections::HashMap;

/// Map MUL era display names to our database era slugs.
pub fn era_mappings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("Age of War", "age-of-war");
    m.insert("Star League", "star-league");
    m.insert("Early Succession War", "early-succession-wars");
    m.insert("Early Succession Wars", "early-succession-wars");
    m.insert("Late Succession War - LosTech", "late-succession-wars");
    m.insert("Late Succession War - Renaissance", "renaissance");
    m.insert("Clan Invasion", "clan-invasion");
    m.insert("Civil War", "civil-war");
    m.insert("Jihad", "jihad");
    m.insert("Dark Age", "dark-age");
    m.insert("Early Republic", "dark-age");
    m.insert("Late Republic", "dark-age");
    m.insert("ilClan", "ilclan");
    m
}

/// Map MUL faction display names to our database faction slugs.
/// This covers the 33 factions seeded in seed.rs.
pub fn faction_mappings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // Inner Sphere Great Houses
    m.insert("Lyran Commonwealth", "steiner");
    m.insert("Lyran Alliance", "steiner");
    m.insert("Federated Suns", "davion");
    m.insert("Federated Commonwealth", "davion");
    m.insert("Draconis Combine", "kurita");
    m.insert("Free Worlds League", "marik");
    m.insert("Capellan Confederation", "liao");
    // Star League / Successors
    m.insert("Star League Regular", "star-league");
    m.insert("Star League Royal", "star-league");
    m.insert("Star League", "star-league");
    m.insert("ComStar", "comstar");
    m.insert("Word of Blake", "word-of-blake");
    m.insert("Republic of the Sphere", "republic");
    // Clans
    m.insert("Clan Wolf", "clan-wolf");
    m.insert("Clan Wolf (in Exile)", "clan-wolf");
    m.insert("Clan Jade Falcon", "clan-jade-falcon");
    m.insert("Clan Ghost Bear", "clan-ghost-bear");
    m.insert("Rasalhague Dominion", "clan-ghost-bear");
    m.insert("Clan Smoke Jaguar", "clan-smoke-jaguar");
    m.insert("Clan Nova Cat", "clan-nova-cat");
    m.insert("Clan Steel Viper", "clan-steel-viper");
    m.insert("Clan Diamond Shark", "clan-diamond-shark");
    m.insert("Clan Sea Fox", "clan-diamond-shark");
    m.insert("Clan Goliath Scorpion", "clan-goliath-scorpion");
    m.insert("Clan Ice Hellion", "clan-ice-hellion");
    m.insert("Clan Star Adder", "clan-star-adder");
    m.insert("Clan Hell's Horses", "clan-hell-horses");
    m.insert("Clan Blood Spirit", "clan-blood-spirit");
    m.insert("Clan Coyote", "clan-coyote");
    m.insert("Clan Fire Mandrill", "clan-fire-mandrill");
    m.insert("Clan Mongoose", "clan-mongoose");
    m.insert("Clan Widowmaker", "clan-widowmaker");
    m.insert("Clan Wolverine", "clan-wolverine");
    // Periphery
    m.insert("Taurian Concordat", "taurian-concordat");
    m.insert("Magistracy of Canopus", "magistracy-canopus");
    m.insert("Outworlds Alliance", "outworlds-alliance");
    m.insert("Marian Hegemony", "marian-hegemony");
    // General
    m.insert("Inner Sphere General", "general");
    m.insert("Clan General", "general");
    m.insert("Mercenary", "mercenary");
    m
}

/// Infer a faction type from its name when auto-creating a new faction.
pub fn infer_faction_type(name: &str) -> &'static str {
    if name.starts_with("Clan ") {
        "clan"
    } else if name.contains("Periphery")
        || name.contains("Concordat")
        || name.contains("Canopus")
        || name.contains("Alliance")
        || name.contains("Hegemony")
        || name.contains("Magistracy")
    {
        "periphery"
    } else if name.contains("Mercenary") || name.contains("mercenary") {
        "mercenary"
    } else {
        "other"
    }
}
