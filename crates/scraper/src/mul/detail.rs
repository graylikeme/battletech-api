use html_scraper::{Html, Selector};
use tracing::warn;

/// A single availability record: (era_name, faction_name).
/// Availability is binary on MUL — if listed, the unit is available to that faction in that era.
#[derive(Debug, Clone)]
pub struct AvailabilityRecord {
    pub era_name: String,
    pub faction_name: String,
}

/// Parse the availability section from a MUL detail page HTML.
///
/// The MUL uses an accordion layout:
/// - Each panel = one era, with the era name in the panel heading link
/// - Inside each panel = a table with one row per faction
/// - Faction name is in the link text within the row
pub fn parse_availability(html: &str) -> Vec<AvailabilityRecord> {
    let document = Html::parse_document(html);

    let panel_sel = Selector::parse(".panel.panel-default").unwrap();
    let heading_link_sel = Selector::parse(".panel-heading .media-body a").unwrap();
    let body_sel = Selector::parse(".panel-body").unwrap();
    let row_sel = Selector::parse("tbody tr").unwrap();
    let link_sel = Selector::parse("a").unwrap();

    let mut records = Vec::new();

    for panel in document.select(&panel_sel) {
        // Extract era name from the heading link text
        let era_name = match panel.select(&heading_link_sel).next() {
            Some(link) => {
                let raw = link.text().collect::<String>();
                // Strip year range suffix: "Star League (2571 - 2780)" → "Star League"
                let name = raw.trim();
                if let Some(paren) = name.find('(') {
                    name[..paren].trim().to_string()
                } else {
                    name.to_string()
                }
            }
            None => continue,
        };

        // Extract faction names from the panel body table rows
        let body = match panel.select(&body_sel).next() {
            Some(b) => b,
            None => continue,
        };

        for row in body.select(&row_sel) {
            if let Some(link) = row.select(&link_sel).next() {
                let faction_raw = link.text().collect::<String>();
                let faction_name = faction_raw.trim().to_string();
                if !faction_name.is_empty() {
                    records.push(AvailabilityRecord {
                        era_name: era_name.clone(),
                        faction_name,
                    });
                }
            }
        }
    }

    if records.is_empty() {
        // Check if the page looks like a valid detail page
        let title_sel = Selector::parse("h2").unwrap();
        if document.select(&title_sel).next().is_some() {
            warn!("parsed detail page but found zero availability records");
        }
    }

    records
}
