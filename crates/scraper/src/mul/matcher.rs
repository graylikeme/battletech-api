use std::collections::HashMap;

use crate::parse::to_slug;

/// A matched MUL unit to DB unit association.
pub struct MatchResult {
    pub db_slug: String,
    pub db_id: i32,
}

/// Unmatched MUL unit info for CSV output.
pub struct UnmatchedUnit {
    pub mul_id: u32,
    pub mul_name: String,
    pub computed_slug: String,
    pub tonnage: f64,
}

/// Matcher resolves MUL unit names to DB unit slugs.
pub struct Matcher {
    /// Manual overrides: MUL ID → DB slug
    overrides: HashMap<u32, String>,
    /// DB units by slug: slug → unit id
    units_by_slug: HashMap<String, i32>,
    /// DB units by lowercased full_name: name → (slug, id)
    units_by_name: HashMap<String, (String, i32)>,
}

impl Matcher {
    pub fn new(
        overrides: HashMap<u32, String>,
        units_by_slug: HashMap<String, i32>,
        units_by_name: HashMap<String, (String, i32)>,
    ) -> Self {
        Self {
            overrides,
            units_by_slug,
            units_by_name,
        }
    }

    /// Try to match a MUL unit to a DB unit.
    pub fn match_unit(&self, mul_id: u32, mul_name: &str, tonnage: f64) -> Result<MatchResult, UnmatchedUnit> {
        // 1. Manual override
        if let Some(slug) = self.overrides.get(&mul_id) {
            if let Some(&db_id) = self.units_by_slug.get(slug.as_str()) {
                return Ok(MatchResult {
                    db_slug: slug.clone(),
                    db_id,
                });
            }
        }

        // 2. Exact slug match
        let slug = to_slug(mul_name);
        if let Some(&db_id) = self.units_by_slug.get(&slug) {
            return Ok(MatchResult { db_slug: slug, db_id });
        }

        // 3. Dual Clan/IS name match: "Dasher (Fire Moth) A" → try "Dasher A" and "Fire Moth A"
        for alt in dual_name_alternatives(mul_name) {
            let alt_slug = to_slug(&alt);
            if let Some(&db_id) = self.units_by_slug.get(&alt_slug) {
                return Ok(MatchResult { db_slug: alt_slug, db_id });
            }
        }

        // 4. Normalized slug match: strip parenthetical suffixes, collapse whitespace
        let normalized = normalize_name(mul_name);
        let norm_slug = to_slug(&normalized);
        if norm_slug != slug {
            if let Some(&db_id) = self.units_by_slug.get(&norm_slug) {
                return Ok(MatchResult {
                    db_slug: norm_slug,
                    db_id,
                });
            }
        }

        // 5. Case-insensitive full_name match
        let lower_name = mul_name.to_lowercase();
        if let Some((ref db_slug, db_id)) = self.units_by_name.get(&lower_name) {
            return Ok(MatchResult {
                db_slug: db_slug.clone(),
                db_id: *db_id,
            });
        }

        // Also try normalized name for case-insensitive match
        let lower_norm = normalized.to_lowercase();
        if lower_norm != lower_name {
            if let Some((ref db_slug, db_id)) = self.units_by_name.get(&lower_norm) {
                return Ok(MatchResult {
                    db_slug: db_slug.clone(),
                    db_id: *db_id,
                });
            }
        }

        // Also try dual-name alternatives for case-insensitive match
        for alt in dual_name_alternatives(mul_name) {
            let alt_lower = alt.to_lowercase();
            if let Some((ref db_slug, db_id)) = self.units_by_name.get(&alt_lower) {
                return Ok(MatchResult {
                    db_slug: db_slug.clone(),
                    db_id: *db_id,
                });
            }
        }

        Err(UnmatchedUnit {
            mul_id,
            mul_name: mul_name.to_string(),
            computed_slug: slug,
            tonnage,
        })
    }
}

/// Extract the Clan/IS alternate name from a dual-name MUL unit.
/// For "Dasher (Fire Moth) A" returns `Some("Fire Moth A")`.
/// Only extracts when there is a variant suffix after the closing paren,
/// which distinguishes dual names like "Dasher (Fire Moth) A" from
/// named/character variants like "Awesome AWS-8Q (Smith)".
pub fn extract_clan_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let open = trimmed.find('(')?;
    let close = open + trimmed[open..].find(')')?;

    let inside = trimmed[open + 1..close].trim();
    let after = trimmed[close + 1..].trim();

    // Only extract when there's text after the closing paren (variant suffix).
    // Trailing parentheticals like "(Smith)" are pilot/character names, not dual names.
    if inside.is_empty() || after.is_empty() {
        return None;
    }

    Some(format!("{} {}", inside, after))
}

/// Extract alternative names from dual Clan/IS naming patterns.
/// MUL uses "IS Name (Clan Name) Variant" format, e.g. "Dasher (Fire Moth) A".
/// Returns alternatives: ["Dasher A", "Fire Moth A"].
/// Also handles reversed "Clan Name (IS Name) Variant" patterns.
fn dual_name_alternatives(name: &str) -> Vec<String> {
    let trimmed = name.trim();

    // Find the parenthetical: "Dasher (Fire Moth) A"
    let open = match trimmed.find('(') {
        Some(i) => i,
        None => return vec![],
    };
    let close = match trimmed[open..].find(')') {
        Some(i) => open + i,
        None => return vec![],
    };

    let before = trimmed[..open].trim(); // "Dasher"
    let inside = trimmed[open + 1..close].trim(); // "Fire Moth"
    let after = trimmed[close + 1..].trim(); // "A"

    // Skip if either part is empty
    if before.is_empty() || inside.is_empty() {
        return vec![];
    }

    let mut alts = Vec::new();

    // "Dasher A" (outer name + suffix)
    if after.is_empty() {
        alts.push(before.to_string());
    } else {
        alts.push(format!("{} {}", before, after));
    }

    // "Fire Moth A" (inner name + suffix)
    if after.is_empty() {
        alts.push(inside.to_string());
    } else {
        alts.push(format!("{} {}", inside, after));
    }

    alts
}

/// Normalize a MUL unit name for matching:
/// - Strip parenthetical suffixes like "(Hanssen)", "(Custom)"
/// - Collapse whitespace
fn normalize_name(name: &str) -> String {
    let trimmed = name.trim();

    // Strip trailing parenthetical content (common named variants)
    let base = if let Some(idx) = trimmed.rfind('(') {
        trimmed[..idx].trim()
    } else {
        trimmed
    };

    // Collapse multiple whitespace
    base.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Load overrides from a JSON file. Format: {"142": "atlas-as7-d-dc", "143": "atlas-as7-dr"}
pub fn load_overrides(path: &std::path::Path) -> anyhow::Result<HashMap<u32, String>> {
    let content = std::fs::read_to_string(path)?;
    let raw: HashMap<String, String> = serde_json::from_str(&content)?;
    let mut overrides = HashMap::new();
    for (k, v) in raw {
        let mul_id: u32 = k.parse()?;
        overrides.insert(mul_id, v);
    }
    Ok(overrides)
}

/// Write unmatched units to a CSV file for review.
pub fn write_unmatched_csv(
    path: &std::path::Path,
    unmatched: &[UnmatchedUnit],
) -> anyhow::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    writeln!(f, "mul_id,mul_name,computed_slug,tonnage")?;
    for u in unmatched {
        writeln!(
            f,
            "{},{},{},{}",
            u.mul_id,
            escape_csv(&u.mul_name),
            u.computed_slug,
            u.tonnage
        )?;
    }
    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
