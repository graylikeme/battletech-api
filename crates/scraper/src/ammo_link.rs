use std::collections::HashMap;

use anyhow::Context;
use sqlx::Row;
use tracing::{info, warn};

/// Normalize an ammo name by stripping variant suffixes so we can match to a base weapon.
fn normalize_ammo_name(name: &str) -> String {
    let mut s = name.to_string();
    // Strip (R) prefix
    if s.starts_with("(R) ") {
        s = s[4..].to_string();
    }
    // Strip omnipod suffixes
    s = s.replace(" (omnipod)", "").replace(" (OMNIPOD)", "");
    // Strip :OMNI
    if s.ends_with(":OMNI") {
        s.truncate(s.len() - 5);
    }
    // Strip pipe-duplicated names like "X|X"
    if let Some(pos) = s.find('|') {
        s.truncate(pos);
    }
    // Strip colon-suffixed shot counts like :20, :100, :Shots25#
    if let Some(pos) = s.rfind(':') {
        let suffix = &s[pos + 1..];
        if suffix.chars().next().map_or(false, |c| c.is_ascii_digit() || c == 'S') {
            s.truncate(pos);
        }
    }
    // Strip parenthesized shot counts like (48) at end
    if s.ends_with(')') {
        if let Some(pos) = s.rfind('(') {
            let inner = s[pos + 1..s.len() - 1].trim();
            if inner.chars().all(|c| c.is_ascii_digit()) {
                s.truncate(pos);
            }
        }
    }
    // Strip Artemis-capable / Artemis V-capable / Artemis-V-capable
    for pat in &[" Artemis-capable", " Artemis V-capable", " Artemis-V-capable"] {
        if let Some(pos) = s.find(pat) {
            s.truncate(pos);
        }
    }
    // Strip Narc-capable
    if let Some(pos) = s.find(" Narc-capable") {
        s.truncate(pos);
    }
    // Strip (Clan) qualifier
    s = s.replace(" (Clan)", "");
    // Strip (Split)
    s = s.replace(" (Split)", "");
    s.trim().to_string()
}

/// Try to extract a weapon slug from a normalized ammo name.
/// Returns the weapon NAME as it appears in the DB (we look up by name, not slug).
fn match_ammo_to_weapon(base: &str) -> Option<&'static str> {
    // ── IS standard ammo ──
    // IS Ammo AC/2..20
    if base.starts_with("IS Ammo AC/") { return match &base[11..] {
        "2" => Some("Autocannon/2"), "5" => Some("Autocannon/5"),
        "10" => Some("Autocannon/10"), "20" => Some("Autocannon/20"), _ => None,
    }}
    // IS Ammo SRM-X
    if base.starts_with("IS Ammo SRM-") { return match &base[12..] {
        "2" => Some("SRM 2"), "4" => Some("SRM 4"), "6" => Some("SRM 6"), _ => None,
    }}
    // IS Ammo LRM-X
    if base.starts_with("IS Ammo LRM-") { return match &base[12..] {
        "5" => Some("LRM 5"), "10" => Some("LRM 10"), "15" => Some("LRM 15"), "20" => Some("LRM 20"), _ => None,
    }}

    // ── Clan standard ammo ──
    if base.starts_with("Clan Ammo SRM-") {
        let n = &base[14..];
        return match n {
            "1" => Some("CLSRM1"), "2" => Some("CLSRM2"), "3" => Some("CLSRM3"),
            "4" => Some("CLSRM4"), "5" => Some("CLSRM5"), "6" => Some("CLSRM6"), _ => None,
        }
    }
    if base.starts_with("Clan Ammo LRM-") {
        let n = &base[14..];
        return match n {
            "2" => Some("CLLRM2"), "3" => Some("CLLRM3"), "4" => Some("CLLRM4"),
            "5" => Some("CLLRM5"), "6" => Some("CLLRM6"), "10" => Some("CLLRM10"),
            "12" => Some("CLLRM12"), "15" => Some("CLLRM15"), "20" => Some("CLLRM20"), _ => None,
        }
    }

    // ── ATM ──
    if base.starts_with("Clan Ammo ATM-") || base.starts_with("Clan Ammo iATM-") {
        let n = base.rsplit('-').next().unwrap_or("");
        let n = n.split(' ').next().unwrap_or(n); // strip " ER" / " HE"
        return match n {
            "3" => Some("CLATM3"), "6" => Some("CLATM6"),
            "9" => Some("CLATM9"), "12" => Some("CLATM12"), _ => None,
        }
    }
    if base.starts_with("CLATM") {
        let rest = base.strip_prefix("CLATM").unwrap_or("");
        let rest = rest.strip_suffix(" Ammo").unwrap_or(rest);
        let rest = rest.split(' ').next().unwrap_or(rest);
        return match rest {
            "3" => Some("CLATM3"), "6" => Some("CLATM6"),
            "9" => Some("CLATM9"), "12" => Some("CLATM12"), _ => None,
        }
    }

    // Static map for everything else
    static_map(base)
}

fn static_map(base: &str) -> Option<&'static str> {
    Some(match base {
        // Machine guns
        "IS Ammo MG - Full" | "IS Ammo MG - Half" | "IS Machine Gun Ammo"
        | "IS Machine Gun Ammo - Half" | "ISMG Ammo" => "Machine Gun",
        "IS Light Machine Gun Ammo - Full" | "IS Light Machine Gun Ammo - Half"
        | "ISLightMG Ammo" => "Light Machine Gun",
        "IS Heavy Machine Gun Ammo - Full" | "IS Heavy Machine Gun Ammo - Half" => "Heavy Machine Gun",
        "Clan Machine Gun Ammo - Full" | "Clan Machine Gun Ammo - Half"
        | "Clan Machine Gun Ammo - Proto"
        | "CLMG Ammo" => "Machine Gun",
        "Clan Light Machine Gun Ammo - Full" | "Clan Light Machine Gun Ammo - Half"
        | "CLLightMG Ammo" => "Light Machine Gun",
        "Clan Heavy Machine Gun Ammo - Full" | "Clan Heavy Machine Gun Ammo - Half" => "Heavy Machine Gun",

        // Gauss
        "IS Gauss Ammo" | "ISGauss Ammo" => "Gauss Rifle",
        "Clan Gauss Ammo" | "CLGauss Ammo" => "CLGaussRifle",
        "IS Light Gauss Ammo" | "ISLightGauss Ammo" => "Light Gauss Rifle",
        "ISHeavyGauss Ammo" => "ISHeavyGaussRifle",
        "IS Improved Heavy Gauss Rifle Ammo" => "Improved Heavy Gauss Rifle",
        "ISSBGauss Ammo" | "ISSBGaussRifleAmmo" | "Silver Bullet Gauss Ammo" => "Silver Bullet Gauss Rifle",
        "CLAPGaussRifle Ammo" => "CLAPGaussRifle",
        "ISMagshotGR Ammo" => "ISMagshotGR",

        // Ultra ACs
        "IS Ultra AC/2 Ammo" | "ISUltraAC2 Ammo" => "ISUltraAC2",
        "IS Ultra AC/5 Ammo" | "ISUltraAC5 Ammo" => "ISUltraAC5",
        "IS Ultra AC/10 Ammo" | "ISUltraAC10 Ammo" => "ISUltraAC10",
        "IS Ultra AC/20 Ammo" | "ISUltraAC20 Ammo" => "ISUltraAC20",
        "Clan Ultra AC/2 Ammo" | "CLUltraAC2 Ammo" => "CLUltraAC2",
        "Clan Ultra AC/5 Ammo" | "CLUltraAC5 Ammo" => "CLUltraAC5",
        "Clan Ultra AC/10 Ammo" | "CLUltraAC10 Ammo" => "CLUltraAC10",
        "Clan Ultra AC/20 Ammo" | "CLUltraAC20 Ammo" => "CLUltraAC20",

        // LB-X ACs (both standard and cluster ammo link to the same weapon)
        "IS LB 2-X AC Ammo" | "IS LB 2-X Cluster Ammo" | "ISLBXAC2 Ammo" | "ISLBXAC2 CL Ammo" => "ISLBXAC2",
        "IS LB 5-X AC Ammo" | "IS LB 5-X Cluster Ammo" | "ISLBXAC5 CL Ammo" => "ISLBXAC5",
        "IS LB 10-X AC Ammo" | "IS LB 10-X Cluster Ammo" | "ISLBXAC10 Ammo" | "ISLBXAC10 CL Ammo"
        | "IS LB 10-X AC Ammo:Shots5#" | "IS LB 10-X Cluster Ammo:Shots5#" => "ISLBXAC10",
        "IS LB 20-X AC Ammo" | "IS LB 20-X Cluster Ammo" | "ISLBXAC20 Ammo" | "ISLBXAC20 CL Ammo" => "ISLBXAC20",
        "Clan LB 2-X AC Ammo" | "Clan LB 2-X Cluster Ammo" => "CLLBXAC2",
        "Clan LB 5-X AC Ammo" | "Clan LB 5-X Cluster Ammo" => "CLLBXAC5",
        "Clan LB 10-X AC Ammo" | "Clan LB 10-X Cluster Ammo" => "CLLBXAC10",
        "Clan LB 20-X AC Ammo" | "Clan LB 20-X Cluster Ammo" => "CLLBXAC20",

        // Rotary ACs
        "IS Rotary AC/2 Ammo" | "ISRotaryAC2 Ammo" => "ISRotaryAC2",
        "IS Rotary AC/5 Ammo" | "ISRotaryAC5 Ammo" => "ISRotaryAC5",

        // Light ACs
        "ISLAC2 Ammo" | "Light AC/2 Ammo" => "Light AC/2",
        "ISLAC5 Ammo" => "Light AC/5",

        // Streak SRMs
        "IS Streak SRM 2 Ammo" | "ISStreakSRM2 Ammo" => "ISStreakSRM2",
        "IS Streak SRM 4 Ammo" | "ISStreakSRM4 Ammo" | "IS Streak SRM 4 Ammo:Shots25#" => "ISStreakSRM4",
        "IS Streak SRM 6 Ammo" | "ISStreakSRM6 Ammo" | "IS Streak SRM 6 Ammo:Shots15#" => "ISStreakSRM6",
        "Clan Streak SRM 2 Ammo" | "CLStreakSRM2 Ammo" => "CLStreakSRM2",
        "Clan Streak SRM 4 Ammo" | "CLStreakSRM4 Ammo" => "CLStreakSRM4",
        "Clan Streak SRM 6 Ammo" | "CLStreakSRM6 Ammo" => "CLStreakSRM6",

        // MRMs
        "IS MRM 10 Ammo" | "ISMRM10 Ammo" => "MRM 10",
        "IS MRM 20 Ammo" | "ISMRM20 Ammo" => "MRM 20",
        "IS MRM 30 Ammo" | "ISMRM30 Ammo" => "MRM 30",
        "IS MRM 40 Ammo" | "ISMRM40 Ammo" => "MRM 40",

        // MMLs
        "IS Ammo MML-3 SRM" | "IS Ammo MML-3 LRM" | "ISMML3 SRM Ammo" | "ISMML3 LRM Ammo" => "MML 3",
        "IS Ammo MML-5 SRM" | "IS Ammo MML-5 LRM" | "ISMML5 SRM Ammo" | "ISMML5 LRM Ammo" => "MML 5",
        "IS Ammo MML-7 SRM" | "IS Ammo MML-7 LRM" | "ISMML7 SRM Ammo" | "ISMML7 LRM Ammo" => "MML 7",
        "IS Ammo MML-9 SRM" | "IS Ammo MML-9 LRM" | "ISMML9 SRM Ammo" | "ISMML9 LRM Ammo" => "MML 9",

        // AMS
        "ISAMS Ammo" | "IS AMS Ammo" => "Anti-Missile System",
        "CLAMS Ammo" => "CLAntiMissileSystem",

        // Narc
        "IS Ammo iNarc" | "ISiNarcBeacon Ammo" => "iNarc",
        "ISNarc Pods" | "ISNarcBeacon Ammo" => "Narc",

        // Arrow IV
        "ISArrowIV Ammo" | "ISArrowIVAmmo" | "ISArrowIV Homing Ammo" | "ISArrowIVHomingAmmo"
        | "ISArrowIVClusterAmmo" => "ISArrowIV",
        "CLArrowIVAmmo" | "CLArrowIVHomingAmmo" | "CLArrowIVClusterAmmo" => "CLArrowIV",

        // Thunderbolts
        "ISThunderbolt5 Ammo" => "ISThunderbolt5",
        "ISThunderbolt10 Ammo" => "ISThunderbolt10",
        "ISThunderbolt15 Ammo" => "ISThunderbolt15",
        "ISThunderbolt20 Ammo" => "ISThunderbolt20",

        // Artillery
        "ISLongTomAmmo" | "ISLongTomCannonAmmo" => "ISLongTomCannon",
        "ISSniperAmmo" | "ISSniperCannonAmmo" => "ISSniperCannon",
        "ISThumperAmmo" | "ISThumperCannonAmmo" => "ISThumperCannon",

        // Plasma
        "ISPlasmaRifle Ammo" | "ISPlasmaRifleAmmo" => "Plasma Rifle",

        // Flamers
        "IS Vehicle Flamer Ammo" => "Vehicle Flamer",
        "Heavy Flamer Ammo" => "Heavy Flamer",
        "CLMediumChemLaserAmmo" => "CLMediumChemicalLaser",

        // IS alternative naming (ISSRM/ISLRM pattern)
        "ISSRM2 Ammo" => "SRM 2", "ISSRM4 Ammo" => "SRM 4", "ISSRM6 Ammo" => "SRM 6",
        "ISLRM5 Ammo" => "LRM 5", "ISLRM10 Ammo" => "LRM 10",
        "ISLRM15 Ammo" => "LRM 15", "ISLRM20 Ammo" => "LRM 20",

        // IS torpedo variants
        "ISSRT4 Ammo" => "SRM 4",
        "ISLRT15 Ammo" => "LRM 15",

        // Clan torpedo variants → same weapon
        "Clan Ammo SRTorpedo-2" => "CLSRM2",
        "Clan Ammo SRTorpedo-4" => "CLSRM4",
        "Clan Ammo SRTorpedo-6" => "CLSRM6",
        "Clan Ammo LRTorpedo-5" => "CLLRM5",
        "Clan Ammo LRTorpedo-10" => "CLLRM10",
        "Clan Ammo LRTorpedo-15" => "CLLRM15",

        // Clan Protomech LRM → same as regular Clan LRM
        "Clan Ammo Protomech LRM-2" => "CLLRM2",
        "Clan Ammo Protomech LRM-3" => "CLLRM3",
        "Clan Ammo Protomech LRM-4" => "CLLRM4",
        "Clan Ammo Protomech LRM-6" => "CLLRM6",
        "Clan Ammo Protomech LRM-12" => "CLLRM12",

        // Clan Improved LRM
        "ClanImprovedLRM10Ammo" => "CLLRM10",
        "ClanImprovedLRM15Ammo" => "CLLRM15",
        "ClanImprovedLRM20Ammo" => "CLLRM20",

        // Clan SC Mortar
        "Clan Ammo SC Mortar-4" => "Clan Ammo SC Mortar-4",
        "Clan Ammo SC Mortar-8" => "Clan Ammo SC Mortar-8",

        // Mek Taser
        "MekTaserAmmo" | "Taser Ammo" => "Mek Taser",

        // HAGs
        "Hyper-Assault Gauss Rifle/20 Ammo" | "CLHAG20 Ammo" => "CLHAG20",
        "Hyper-Assault Gauss Rifle/30 Ammo" | "CLHAG30 Ammo" => "CLHAG30",
        "Hyper-Assault Gauss Rifle/40 Ammo" | "CLHAG40 Ammo" => "CLHAG40",

        // Extended LRM
        "IS Ammo Extended LRM-5" => "LRM 5",
        "IS Ammo Extended LRM-10" => "LRM 10",
        "IS Ammo Extended LRM-15" => "LRM 15",

        // IS Ammo LAC
        "IS Ammo LAC/2" => "Light AC/2",
        "IS Ammo LAC/5" => "Light AC/5",

        // SC Mortars (no weapon equivalent, skip)
        "Clan Ammo SC Mortar-4" | "Clan Ammo SC Mortar-8" => return None,

        // Cruise missiles
        "ISCruiseMissile50Ammo" | "ISCruiseMissile70Ammo" | "ISCruiseMissile90Ammo"
        | "ISCruiseMissile120Ammo" => return None,

        _ => return None,
    })
}

pub async fn run(database_url: &str, pool_size: u32) -> anyhow::Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(database_url)
        .await
        .context("connecting to database")?;

    // Load all weapons into a name → id map
    let weapons: Vec<(i32, String)> = sqlx::query("SELECT id, name FROM equipment WHERE category != 'ammunition'")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|r| (r.get("id"), r.get("name")))
        .collect();

    let weapon_by_name: HashMap<&str, i32> = weapons.iter().map(|(id, name)| (name.as_str(), *id)).collect();

    // Load all ammo
    let ammo_rows: Vec<(i32, String)> = sqlx::query("SELECT id, name FROM equipment WHERE category = 'ammunition'")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|r| (r.get("id"), r.get("name")))
        .collect();

    info!(ammo_count = ammo_rows.len(), weapon_count = weapons.len(), "loaded equipment");

    let mut linked = 0u32;
    let mut unmatched = 0u32;
    let mut not_found = 0u32;

    for (ammo_id, ammo_name) in &ammo_rows {
        let base = normalize_ammo_name(ammo_name);
        let weapon_name = match match_ammo_to_weapon(&base) {
            Some(w) => w,
            None => {
                unmatched += 1;
                if unmatched <= 30 {
                    warn!(ammo = %ammo_name, base = %base, "no pattern match");
                }
                continue;
            }
        };

        let weapon_id = match weapon_by_name.get(weapon_name) {
            Some(&id) => id,
            None => {
                not_found += 1;
                if not_found <= 20 {
                    warn!(ammo = %ammo_name, weapon = %weapon_name, "weapon not found in DB");
                }
                continue;
            }
        };

        sqlx::query("UPDATE equipment SET ammo_for_id = $1 WHERE id = $2")
            .bind(weapon_id)
            .bind(ammo_id)
            .execute(&pool)
            .await?;

        linked += 1;
    }

    info!(linked, unmatched, not_found, "ammo-link complete");

    Ok(())
}
