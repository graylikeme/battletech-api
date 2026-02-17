/// Common parsed representation of any unit (mech or vehicle).
#[derive(Debug, Clone)]
pub struct ParsedUnit {
    pub chassis: String,
    pub model: String,
    pub mul_id: Option<i32>,
    pub unit_type: UnitType,
    pub tech_base: TechBase,
    pub rules_level: RulesLevel,
    pub intro_year: Option<i32>,
    pub source: Option<String>,
    pub tonnage: f64,
    /// Armor by location
    pub locations: Vec<ParsedLocation>,
    /// Weapon/equipment loadout (name, location, qty, rear_facing)
    pub loadout: Vec<ParsedLoadoutEntry>,
    /// Quirk slugs
    pub quirks: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedLocation {
    pub location: &'static str,
    pub armor: Option<i32>,
    pub rear_armor: Option<i32>,
    pub structure: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ParsedLoadoutEntry {
    pub equipment: String,
    pub location: Option<&'static str>,
    pub quantity: i32,
    pub is_rear: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    Mek,
    Vehicle,
    Fighter,
    Other,
}

impl UnitType {
    pub fn as_str(self) -> &'static str {
        match self {
            UnitType::Mek => "mek",
            UnitType::Vehicle => "vehicle",
            UnitType::Fighter => "fighter",
            UnitType::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechBase {
    InnerSphere,
    Clan,
    Mixed,
    Primitive,
}

impl TechBase {
    pub fn as_str(self) -> &'static str {
        match self {
            TechBase::InnerSphere => "inner_sphere",
            TechBase::Clan => "clan",
            TechBase::Mixed => "mixed",
            TechBase::Primitive => "primitive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("clan") && !lower.contains("inner") {
            TechBase::Clan
        } else if lower.contains("mixed") {
            TechBase::Mixed
        } else if lower.contains("primitive") {
            TechBase::Primitive
        } else {
            TechBase::InnerSphere
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulesLevel {
    Introductory,
    Standard,
    Advanced,
    Experimental,
    Unofficial,
}

impl RulesLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            RulesLevel::Introductory => "introductory",
            RulesLevel::Standard => "standard",
            RulesLevel::Advanced => "advanced",
            RulesLevel::Experimental => "experimental",
            RulesLevel::Unofficial => "unofficial",
        }
    }

    /// Parse from the integer used in MTF `rules level:N`
    pub fn from_int(n: i32) -> Self {
        match n {
            0 => RulesLevel::Introductory,
            1 => RulesLevel::Standard,
            2 => RulesLevel::Advanced,
            3 => RulesLevel::Experimental,
            4 | 5 => RulesLevel::Unofficial,
            _ => RulesLevel::Standard,
        }
    }

    /// Parse from the BLK `<type>` string like "IS Level 2"
    pub fn from_type_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("level 1") {
            RulesLevel::Standard
        } else if lower.contains("level 2") {
            RulesLevel::Advanced
        } else if lower.contains("level 3") {
            RulesLevel::Experimental
        } else if lower.contains("unofficial") {
            RulesLevel::Unofficial
        } else {
            RulesLevel::Standard
        }
    }
}

// ── MTF parser ─────────────────────────────────────────────────────────────

pub fn parse_mtf(content: &str) -> Option<ParsedUnit> {
    let mut chassis = String::new();
    let mut model = String::new();
    let mut mul_id: Option<i32> = None;
    let mut config = String::new(); // Biped, Quad, etc.
    let mut tech_base = TechBase::InnerSphere;
    let mut rules_level = RulesLevel::Standard;
    let mut intro_year: Option<i32> = None;
    let mut source: Option<String> = None;
    let mut tonnage: Option<f64> = None;
    let mut description: Option<String> = None;
    let mut quirks: Vec<String> = Vec::new();

    // Armor values keyed by short location code
    let mut armor: std::collections::HashMap<String, (Option<i32>, Option<i32>)> =
        std::collections::HashMap::new();

    // Weapons loadout
    let mut loadout: Vec<ParsedLoadoutEntry> = Vec::new();

    // Location section parsing
    let mut in_weapons = false;
    let mut current_loc: Option<&'static str> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();

        // Skip comments
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        // Check for location section header (e.g. "Left Arm:", "Right Torso:")
        if !line.contains(':') {
            // Could be a continuation line in a location section
            if let Some(loc) = current_loc {
                let equip = line.trim_end_matches(" (R)").trim().to_string();
                let is_rear = line.ends_with("(R)");
                if !equip.is_empty()
                    && equip != "-Empty-"
                    && !is_structural_component(&equip)
                {
                    // Find if already in loadout at same loc+rear
                    if let Some(entry) = loadout.iter_mut().find(|e| {
                        e.equipment == equip && e.location == Some(loc) && e.is_rear == is_rear
                    }) {
                        entry.quantity += 1;
                    } else {
                        loadout.push(ParsedLoadoutEntry {
                            equipment: equip,
                            location: Some(loc),
                            quantity: 1,
                            is_rear,
                        });
                    }
                }
            }
            continue;
        }

        // Find the colon separator; guard against values that also contain colons
        let colon = line.find(':').unwrap();
        let key = line[..colon].trim().to_lowercase();
        let val = line[colon + 1..].trim().to_string();

        // Location section headers end with ":" and have a canonical name
        if val.is_empty() {
            current_loc = mtf_location_header(&key);
            in_weapons = false;
            continue;
        }

        current_loc = None; // reset when we see a key:value pair
        in_weapons = false;

        match key.as_str() {
            "chassis" => chassis = val.clone(),
            "model" => model = val.clone(),
            "mul id" => mul_id = val.parse().ok(),
            "config" => config = val.clone(),
            "techbase" | "tech base" => tech_base = TechBase::from_str(&val),
            "era" => intro_year = val.parse().ok(),
            "source" => source = Some(val.clone()),
            "rules level" => {
                rules_level = val
                    .parse::<i32>()
                    .map(RulesLevel::from_int)
                    .unwrap_or(RulesLevel::Standard)
            }
            "mass" => tonnage = val.parse().ok(),
            "quirk" => quirks.push(to_slug(&val)),
            "overview" => {
                description = Some(val.trim_matches('"').to_string());
            }
            _ => {}
        }

        // Armor value lines like "LA armor:34"
        if let Some(rest) = key.strip_suffix(" armor") {
            let loc_code = rest.trim().to_uppercase();
            match loc_code.as_str() {
                "RTL" => {
                    armor.entry("LT".to_string()).or_default().1 = val.parse().ok();
                }
                "RTR" => {
                    armor.entry("RT".to_string()).or_default().1 = val.parse().ok();
                }
                "RTC" => {
                    armor.entry("CT".to_string()).or_default().1 = val.parse().ok();
                }
                other => {
                    armor.entry(other.to_string()).or_default().0 = val.parse().ok();
                }
            }
        }

        // Weapons section: "weapons:7"
        if key == "weapons" {
            in_weapons = true;
        }
    }

    // Second pass for weapons lines (they follow "Weapons:N" with no leading key)
    let mut reading_weapons = false;
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("weapons:") {
            reading_weapons = true;
            continue;
        }
        if reading_weapons {
            // Weapon lines: "Medium Laser, Left Arm" or "2 LRM 20, Left Torso"
            // Stop at next key:value pair or location header
            if line.contains(':') && !line.contains(',') {
                reading_weapons = false;
                continue;
            }
            if !line.contains(',') {
                // Might be location header with no value, stop
                reading_weapons = false;
                continue;
            }
            parse_weapon_line(line, &mut loadout);
        }
    }

    if chassis.is_empty() || tonnage.is_none() {
        return None;
    }

    let unit_type = match config.to_lowercase().as_str() {
        "biped" | "quad" | "biped omnimech" | "quad omnimech" | "tripod" => UnitType::Mek,
        _ => UnitType::Mek,
    };

    let locations = build_mek_locations(&armor);

    Some(ParsedUnit {
        chassis,
        model,
        mul_id,
        unit_type,
        tech_base,
        rules_level,
        intro_year,
        source,
        tonnage: tonnage.unwrap(),
        locations,
        loadout: dedup_loadout(loadout),
        quirks,
        description,
    })
}

fn parse_weapon_line(line: &str, loadout: &mut Vec<ParsedLoadoutEntry>) {
    // Format: "[qty] equipment_name, location [, Ammo:N]"
    let parts: Vec<&str> = line.splitn(3, ',').collect();
    if parts.is_empty() {
        return;
    }

    // First part: optional quantity prefix + name
    let name_part = parts[0].trim();
    let (qty, equip_name) = if let Some(rest) = name_part
        .split_once(' ')
        .filter(|(a, _)| a.parse::<i32>().is_ok())
    {
        (rest.0.parse::<i32>().unwrap_or(1), rest.1.trim().to_string())
    } else {
        (1, name_part.to_string())
    };

    if equip_name.is_empty() || equip_name == "-Empty-" {
        return;
    }

    // Location
    let raw_loc = if parts.len() >= 2 {
        parts[1].trim().to_string()
    } else {
        return;
    };
    let is_rear = raw_loc.ends_with("(R)");
    let loc_clean = raw_loc.trim_end_matches("(R)").trim();
    let loc = mtf_weapon_location(loc_clean);

    for _ in 0..qty {
        if let Some(entry) = loadout
            .iter_mut()
            .find(|e| e.equipment == equip_name && e.location == loc && e.is_rear == is_rear)
        {
            entry.quantity += 1;
        } else {
            loadout.push(ParsedLoadoutEntry {
                equipment: equip_name.clone(),
                location: loc,
                quantity: 1,
                is_rear,
            });
        }
    }
}

fn mtf_weapon_location(loc: &str) -> Option<&'static str> {
    match loc.to_lowercase().trim() {
        "left arm" | "la" => Some("left_arm"),
        "right arm" | "ra" => Some("right_arm"),
        "left torso" | "lt" => Some("left_torso"),
        "right torso" | "rt" => Some("right_torso"),
        "center torso" | "ct" => Some("center_torso"),
        "head" | "hd" => Some("head"),
        "left leg" | "ll" => Some("left_leg"),
        "right leg" | "rl" => Some("right_leg"),
        _ => None,
    }
}

fn mtf_location_header(key: &str) -> Option<&'static str> {
    match key {
        "left arm" => Some("left_arm"),
        "right arm" => Some("right_arm"),
        "left torso" => Some("left_torso"),
        "right torso" => Some("right_torso"),
        "center torso" => Some("center_torso"),
        "head" => Some("head"),
        "left leg" => Some("left_leg"),
        "right leg" => Some("right_leg"),
        _ => None,
    }
}

fn build_mek_locations(
    armor: &std::collections::HashMap<String, (Option<i32>, Option<i32>)>,
) -> Vec<ParsedLocation> {
    let mapping: &[(&str, &str)] = &[
        ("LA", "left_arm"),
        ("RA", "right_arm"),
        ("LT", "left_torso"),
        ("RT", "right_torso"),
        ("CT", "center_torso"),
        ("HD", "head"),
        ("LL", "left_leg"),
        ("RL", "right_leg"),
    ];
    mapping
        .iter()
        .filter_map(|(code, loc)| {
            let (front, rear) = armor.get(*code)?;
            Some(ParsedLocation {
                location: loc,
                armor: *front,
                rear_armor: *rear,
                structure: None,
            })
        })
        .collect()
}

fn is_structural_component(s: &str) -> bool {
    matches!(
        s,
        "Shoulder"
            | "Upper Arm Actuator"
            | "Lower Arm Actuator"
            | "Hand Actuator"
            | "Hip"
            | "Upper Leg Actuator"
            | "Lower Leg Actuator"
            | "Foot Actuator"
            | "Life Support"
            | "Sensors"
            | "Cockpit"
            | "Gyro"
            | "Compact Gyro"
            | "Heavy Duty Gyro"
            | "XL Gyro"
            | "Fusion Engine"
            | "XL Engine"
            | "Light Engine"
            | "Compact Engine"
            | "Primitive Fusion Engine"
            | "ICE Engine"
            | "-Empty-"
    ) || s.contains("Engine")
        || s.contains("Endo Steel")
        || s.contains("Ferro-Fibrous")
        || s.contains("Reactive Armor")
        || s.contains("Stealth Armor")
        || s.contains("CASE")
}

// ── BLK parser ─────────────────────────────────────────────────────────────

pub fn parse_blk(content: &str, default_unit_type: UnitType) -> Option<ParsedUnit> {
    // Build a tag -> value map
    let mut tags: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut equipment_by_loc: Vec<(String, String)> = Vec::new(); // (location_tag, equip_line)

    let mut current_tag: Option<String> = None;
    let mut current_value = String::new();

    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with('<') && line.ends_with('>') && !line.starts_with("</") {
            let tag = line[1..line.len() - 1].to_string();
            current_tag = Some(tag);
            current_value = String::new();
        } else if line.starts_with("</") {
            if let Some(tag) = current_tag.take() {
                let v = current_value.trim().to_string();
                if tag.to_lowercase().ends_with("equipment") {
                    let loc = tag
                        .to_lowercase()
                        .strip_suffix("equipment")
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    for eq_line in v.lines() {
                        let eq = eq_line.trim();
                        if !eq.is_empty() {
                            equipment_by_loc.push((loc.clone(), eq.to_string()));
                        }
                    }
                } else {
                    tags.insert(tag, v);
                }
            }
        } else if current_tag.is_some() {
            if !current_value.is_empty() {
                current_value.push('\n');
            }
            current_value.push_str(line);
        }
    }

    let chassis = tags.get("Name")?.trim().to_string();
    let model = tags
        .get("Model")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mul_id = tags
        .get("mul id:")
        .or_else(|| tags.get("mul id"))
        .and_then(|s| s.trim().parse().ok());
    let tonnage: f64 = tags.get("tonnage")?.trim().parse().ok()?;
    let intro_year: Option<i32> = tags.get("year").and_then(|s| s.trim().parse().ok());
    let source = tags.get("source").map(|s| s.trim().to_string());

    let blk_unit_type = tags
        .get("UnitType")
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    let unit_type = match blk_unit_type.as_str() {
        "tank" | "vtol" | "naval" | "wheeled vehicle" | "tracked vehicle" => UnitType::Vehicle,
        "aero" | "aerospacespacefighter" | "conv_fighter" | "conventional fighter" => {
            UnitType::Fighter
        }
        _ => default_unit_type,
    };

    let type_str = tags.get("type").map(|s| s.trim().to_string()).unwrap_or_default();
    let tech_base = TechBase::from_str(&type_str);
    let rules_level = RulesLevel::from_type_str(&type_str);

    let description = tags.get("overview").map(|s| {
        s.trim().trim_matches('"').to_string()
    });

    // Build loadout from equipment tags
    let mut loadout: Vec<ParsedLoadoutEntry> = Vec::new();
    for (loc_tag, equip_name) in &equipment_by_loc {
        let loc = blk_location(loc_tag.trim());
        if let Some(entry) = loadout.iter_mut().find(|e| {
            e.equipment == *equip_name && e.location == loc && !e.is_rear
        }) {
            entry.quantity += 1;
        } else {
            loadout.push(ParsedLoadoutEntry {
                equipment: equip_name.clone(),
                location: loc,
                quantity: 1,
                is_rear: false,
            });
        }
    }

    Some(ParsedUnit {
        chassis,
        model,
        mul_id,
        unit_type,
        tech_base,
        rules_level,
        intro_year,
        source,
        tonnage,
        locations: Vec::new(), // BLK armor parsing skipped for now
        loadout: dedup_loadout(loadout),
        quirks: Vec::new(),
        description,
    })
}

fn blk_location(loc: &str) -> Option<&'static str> {
    match loc {
        "front" => Some("front"),
        "rear" => Some("rear"),
        "right" => Some("right_side"),
        "left" => Some("left_side"),
        "turret" => Some("turret"),
        "body" => Some("body"),
        "left arm" => Some("left_arm"),
        "right arm" => Some("right_arm"),
        _ => None,
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn dedup_loadout(mut entries: Vec<ParsedLoadoutEntry>) -> Vec<ParsedLoadoutEntry> {
    let mut out: Vec<ParsedLoadoutEntry> = Vec::new();
    for entry in entries.drain(..) {
        if let Some(existing) = out.iter_mut().find(|e| {
            e.equipment == entry.equipment
                && e.location == entry.location
                && e.is_rear == entry.is_rear
        }) {
            existing.quantity += entry.quantity;
        } else {
            out.push(entry);
        }
    }
    out
}

/// Convert a display name to a URL/DB slug.
pub fn to_slug(s: &str) -> String {
    let mut slug = String::new();
    let mut prev_hyphen = true; // start true to trim leading hyphens
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen && !slug.is_empty() {
            slug.push('-');
            prev_hyphen = true;
        }
    }
    // trim trailing hyphen
    slug.trim_end_matches('-').to_string()
}

/// Infer equipment category from its display name.
pub fn categorize_equipment(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("ammo") || lower.starts_with("is ammo") || lower.starts_with("clan ammo") {
        return "ammunition";
    }
    if lower.contains("heat sink") || lower == "double heat sink" {
        return "heat_sink";
    }
    if lower.contains("jump jet") || lower.contains("improved jump jet") {
        return "jump_jet";
    }
    if lower.contains("targeting computer") {
        return "targeting_computer";
    }
    if lower.contains("gyro") {
        return "gyro";
    }
    if lower.contains("cockpit") {
        return "cockpit";
    }
    if lower.contains("endo steel") || lower.contains("structure") {
        return "structure";
    }
    if lower.contains("ferro") || lower.contains("reactive armor") || lower.contains("stealth") {
        return "armor";
    }
    if lower.contains("engine") {
        return "engine";
    }
    if lower.contains("laser")
        || lower.contains("ppc")
        || lower.contains("flamer")
        || lower.contains("plasma rifle")
    {
        return "energy_weapon";
    }
    if lower.contains("lrm")
        || lower.contains("srm")
        || lower.contains("streak")
        || lower.contains("narc")
        || lower.contains("ams")
        || lower.contains("mml")
        || lower.contains("atm")
        || lower.contains("rocket")
        || lower.contains("arrow")
        || lower.contains("thunderbolt")
    {
        return "missile_weapon";
    }
    if lower.contains("autocannon")
        || lower.contains("ac/")
        || lower.contains("gauss")
        || lower.contains("rifle")
        || lower.contains("lbx")
        || lower.contains("ultra")
        || lower.contains("rotary")
        || lower.contains("hag")
    {
        return "ballistic_weapon";
    }
    "equipment"
}

/// Infer tech_base of a piece of equipment from its name prefix.
pub fn equipment_tech_base(name: &str) -> &'static str {
    if name.starts_with("CL") || name.to_lowercase().starts_with("clan") {
        "clan"
    } else {
        "inner_sphere"
    }
}
