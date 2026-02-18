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

/// Pagination metadata for cursor-based (keyset) pagination.
#[derive(SimpleObject)]
pub struct PageInfo {
    /// True if there are more items after the last edge in this page.
    pub has_next_page: bool,
    /// True if there are items before the first edge in this page (i.e. a cursor was provided).
    pub has_previous_page: bool,
    /// Opaque cursor pointing to the first edge in this page. Null if the page is empty.
    pub start_cursor: Option<String>,
    /// Opaque cursor pointing to the last edge in this page. Pass this as the "after" parameter to fetch the next page. Null if the page is empty.
    pub end_cursor: Option<String>,
    /// Total number of items matching the query filters, across all pages.
    pub total_count: i64,
}
