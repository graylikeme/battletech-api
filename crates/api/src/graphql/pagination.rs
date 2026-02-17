use base64::{engine::general_purpose::STANDARD, Engine};
use async_graphql::SimpleObject;

/// Encode a cursor from a sort value and row id.
pub fn encode_cursor(sort_val: &str, id: i32) -> String {
    let raw = format!("{}|id:{}", sort_val, id);
    STANDARD.encode(raw.as_bytes())
}

/// Decode a cursor into (sort_val, id).
pub fn decode_cursor(cursor: &str) -> Option<(String, i32)> {
    let bytes = STANDARD.decode(cursor).ok()?;
    let s = String::from_utf8(bytes).ok()?;
    let (sort_val, id_part) = s.split_once("|id:")?;
    let id: i32 = id_part.parse().ok()?;
    Some((sort_val.to_owned(), id))
}

#[derive(SimpleObject)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub total_count: i64,
}
