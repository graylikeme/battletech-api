use serde::Deserialize;

/// Top-level QuickList response wrapper.
#[derive(Debug, Deserialize)]
pub struct QuickListResponse {
    #[serde(alias = "Units")]
    pub units: Vec<MulUnit>,
}

/// A single unit from the MUL QuickList JSON endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct MulUnit {
    #[serde(alias = "Id")]
    pub id: u32,
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "Class")]
    pub class: Option<String>,
    #[serde(alias = "Variant")]
    pub variant: Option<String>,
    #[serde(alias = "Tonnage")]
    pub tonnage: f64,
    #[serde(alias = "BattleValue")]
    pub battle_value: Option<i32>,
    #[serde(alias = "Cost")]
    pub cost: Option<i64>,
    #[serde(alias = "Rules")]
    pub rules: Option<String>,
    #[serde(alias = "DateIntroduced")]
    pub date_introduced: Option<String>,
    #[serde(alias = "Technology")]
    pub technology: Option<IdName>,
    #[serde(alias = "Role")]
    pub role: Option<IdName>,
    #[serde(alias = "Type")]
    pub unit_type: Option<IdName>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdName {
    #[serde(alias = "Id")]
    pub id: Option<i32>,
    #[serde(alias = "Name")]
    pub name: Option<String>,
}

impl MulUnit {
    /// Extract the intro year as an i32 from the date_introduced string.
    /// Looks for the first 4-digit sequence in the string.
    pub fn intro_year(&self) -> Option<i32> {
        let s = self.date_introduced.as_deref()?.trim();
        // Find first 4-digit sequence
        let chars: Vec<char> = s.chars().collect();
        for i in 0..chars.len().saturating_sub(3) {
            if chars[i].is_ascii_digit()
                && chars[i + 1].is_ascii_digit()
                && chars[i + 2].is_ascii_digit()
                && chars[i + 3].is_ascii_digit()
            {
                let year_str: String = chars[i..i + 4].iter().collect();
                return year_str.parse().ok();
            }
        }
        None
    }

    /// Get the role name, trimmed.
    pub fn role_name(&self) -> Option<&str> {
        self.role.as_ref()?.name.as_deref().map(|s| s.trim())
    }

    /// BV, treating 0 as None (MUL uses 0 for missing BV).
    pub fn bv(&self) -> Option<i32> {
        self.battle_value.filter(|&v| v > 0)
    }

    /// Cost, treating 0 as None.
    pub fn cost_value(&self) -> Option<i64> {
        self.cost.filter(|&v| v > 0)
    }
}

/// Parse a QuickList JSON file into a list of MUL units.
pub fn parse_quicklist(json: &str) -> anyhow::Result<Vec<MulUnit>> {
    let resp: QuickListResponse = serde_json::from_str(json)?;
    Ok(resp.units)
}
