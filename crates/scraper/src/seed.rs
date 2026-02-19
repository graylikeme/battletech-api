/// Standard BattleTech eras and factions — seeded before unit import.
use sqlx::PgPool;

pub struct Era {
    pub slug: &'static str,
    pub name: &'static str,
    pub start_year: i32,
    pub end_year: Option<i32>,
    pub description: &'static str,
}

pub const ERAS: &[Era] = &[
    Era {
        slug: "age-of-war",
        name: "Age of War",
        start_year: 2398,
        end_year: Some(2570),
        description: "The period of interstellar warfare that preceded the Star League.",
    },
    Era {
        slug: "star-league",
        name: "Star League",
        start_year: 2571,
        end_year: Some(2780),
        description: "The golden age of humanity spanning the Star League era.",
    },
    Era {
        slug: "early-succession-wars",
        name: "Early Succession Wars",
        start_year: 2781,
        end_year: Some(2900),
        description: "The First and Second Succession Wars; rapid technological decline.",
    },
    Era {
        slug: "late-succession-wars",
        name: "Late Succession Wars (LosTech)",
        start_year: 2901,
        end_year: Some(3019),
        description: "Era of LosTech; Third and early Fourth Succession Wars.",
    },
    Era {
        slug: "renaissance",
        name: "Renaissance",
        start_year: 3020,
        end_year: Some(3049),
        description: "Technological renaissance; Helm Memory Core; Fourth Succession War.",
    },
    Era {
        slug: "clan-invasion",
        name: "Clan Invasion",
        start_year: 3050,
        end_year: Some(3061),
        description: "Clan forces attack the Inner Sphere; Operation Revival.",
    },
    Era {
        slug: "civil-war",
        name: "Civil War",
        start_year: 3062,
        end_year: Some(3067),
        description: "FedCom Civil War; growing tensions across the Inner Sphere.",
    },
    Era {
        slug: "jihad",
        name: "Jihad",
        start_year: 3068,
        end_year: Some(3080),
        description: "Word of Blake Jihad; widespread destruction across known space.",
    },
    Era {
        slug: "dark-age",
        name: "Dark Age",
        start_year: 3081,
        end_year: Some(3150),
        description: "The Republic era and the collapse of HPG communications.",
    },
    Era {
        slug: "ilclan",
        name: "ilClan",
        start_year: 3151,
        end_year: None,
        description: "Recognition of a new ilClan; reshaping of the Inner Sphere.",
    },
];

pub struct Faction {
    pub slug: &'static str,
    pub name: &'static str,
    pub short_name: Option<&'static str>,
    pub faction_type: &'static str,
    pub is_clan: bool,
}

pub const FACTIONS: &[Faction] = &[
    // Inner Sphere Great Houses
    Faction { slug: "steiner", name: "Lyran Commonwealth", short_name: Some("LC"), faction_type: "great_house", is_clan: false },
    Faction { slug: "davion", name: "Federated Suns", short_name: Some("FS"), faction_type: "great_house", is_clan: false },
    Faction { slug: "kurita", name: "Draconis Combine", short_name: Some("DC"), faction_type: "great_house", is_clan: false },
    Faction { slug: "marik", name: "Free Worlds League", short_name: Some("FWL"), faction_type: "great_house", is_clan: false },
    Faction { slug: "liao", name: "Capellan Confederation", short_name: Some("CC"), faction_type: "great_house", is_clan: false },
    // Star League / Successors
    Faction { slug: "star-league", name: "Star League", short_name: Some("SL"), faction_type: "star_league", is_clan: false },
    Faction { slug: "comstar", name: "ComStar", short_name: Some("CS"), faction_type: "independent", is_clan: false },
    Faction { slug: "word-of-blake", name: "Word of Blake", short_name: Some("WoB"), faction_type: "independent", is_clan: false },
    Faction { slug: "republic", name: "Republic of the Sphere", short_name: Some("RS"), faction_type: "inner_sphere", is_clan: false },
    // Clan Invaders
    Faction { slug: "clan-wolf", name: "Clan Wolf", short_name: Some("CW"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-jade-falcon", name: "Clan Jade Falcon", short_name: Some("CJF"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-ghost-bear", name: "Clan Ghost Bear", short_name: Some("CGB"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-smoke-jaguar", name: "Clan Smoke Jaguar", short_name: Some("CSJ"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-nova-cat", name: "Clan Nova Cat", short_name: Some("CNC"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-steel-viper", name: "Clan Steel Viper", short_name: Some("CSV"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-diamond-shark", name: "Clan Diamond Shark", short_name: Some("CDS"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-goliath-scorpion", name: "Clan Goliath Scorpion", short_name: Some("CGS"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-ice-hellion", name: "Clan Ice Hellion", short_name: Some("CIH"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-star-adder", name: "Clan Star Adder", short_name: Some("CSA"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-hell-horses", name: "Clan Hell's Horses", short_name: Some("CHH"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-blood-spirit", name: "Clan Blood Spirit", short_name: Some("CBS"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-coyote", name: "Clan Coyote", short_name: Some("CCY"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-fire-mandrill", name: "Clan Fire Mandrill", short_name: Some("CFM"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-mongoose", name: "Clan Mongoose", short_name: Some("CMG"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-widowmaker", name: "Clan Widowmaker", short_name: Some("CWM"), faction_type: "clan", is_clan: true },
    Faction { slug: "clan-wolverine", name: "Clan Wolverine", short_name: Some("CWOV"), faction_type: "clan", is_clan: true },
    // Periphery
    Faction { slug: "periphery-general", name: "Periphery (General)", short_name: Some("PER"), faction_type: "periphery", is_clan: false },
    Faction { slug: "taurian-concordat", name: "Taurian Concordat", short_name: Some("TC"), faction_type: "periphery", is_clan: false },
    Faction { slug: "magistracy-canopus", name: "Magistracy of Canopus", short_name: Some("MOC"), faction_type: "periphery", is_clan: false },
    Faction { slug: "outworlds-alliance", name: "Outworlds Alliance", short_name: Some("OA"), faction_type: "periphery", is_clan: false },
    Faction { slug: "marian-hegemony", name: "Marian Hegemony", short_name: Some("MH"), faction_type: "periphery", is_clan: false },
    // Mercenaries / General
    Faction { slug: "mercenary", name: "Mercenary", short_name: Some("MER"), faction_type: "mercenary", is_clan: false },
    Faction { slug: "general", name: "General (All)", short_name: Some("GEN"), faction_type: "general", is_clan: false },
];

pub async fn seed_eras(pool: &PgPool) -> anyhow::Result<usize> {
    let mut count = 0usize;
    for era in ERAS {
        let rows = sqlx::query(
            r#"INSERT INTO eras (slug, name, start_year, end_year, description)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (slug) DO NOTHING"#,
        )
        .bind(era.slug)
        .bind(era.name)
        .bind(era.start_year)
        .bind(era.end_year)
        .bind(era.description)
        .execute(pool)
        .await?;
        count += rows.rows_affected() as usize;
    }
    Ok(count)
}

pub async fn seed_factions(pool: &PgPool) -> anyhow::Result<usize> {
    let mut count = 0usize;
    for f in FACTIONS {
        let rows = sqlx::query(
            r#"INSERT INTO factions (slug, name, short_name, faction_type, is_clan)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (slug) DO NOTHING"#,
        )
        .bind(f.slug)
        .bind(f.name)
        .bind(f.short_name)
        .bind(f.faction_type)
        .bind(f.is_clan)
        .execute(pool)
        .await?;
        count += rows.rows_affected() as usize;
    }
    Ok(count)
}

pub async fn seed_metadata(pool: &PgPool, version: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM dataset_metadata WHERE version = $1")
        .bind(version)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"INSERT INTO dataset_metadata (version, schema_version, description)
           VALUES ($1, 1, 'Imported from MegaMek ' || $1)"#,
    )
    .bind(version)
    .execute(pool)
    .await?;
    Ok(())
}
