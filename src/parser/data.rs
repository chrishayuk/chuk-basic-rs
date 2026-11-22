//! DATA statement item lists.

use crate::ast::DataItem;

/// Split the raw text of a DATA statement into items.
pub(super) fn parse_data(raw: &str) -> Vec<DataItem> {
    let chars: Vec<char> = raw.chars().collect();
    let mut items = Vec::new();
    let mut pos = 0;
    loop {
        while chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        if chars.get(pos) == Some(&'"') {
            pos += 1;
            let start = pos;
            while chars.get(pos).is_some_and(|&c| c != '"') {
                pos += 1;
            }
            items.push(DataItem {
                text: chars[start..pos].iter().collect(),
                quoted: true,
            });
            while chars.get(pos).is_some_and(|&c| c != ',') {
                pos += 1;
            }
        } else {
            let start = pos;
            while chars.get(pos).is_some_and(|&c| c != ',') {
                pos += 1;
            }
            let text: String = chars[start..pos].iter().collect();
            items.push(DataItem {
                text: text.trim().to_string(),
                quoted: false,
            });
        }
        if pos >= chars.len() {
            return items;
        }
        pos += 1; // comma
    }
}
