//! Minimum versions and smoke checks. A failed smoke check stays off the catalog.
mod claude_code;
mod codex;

pub use claude_code::minimum as claude_minimum;
pub use codex::minimum as codex_minimum;

pub fn version_at_least(actual: &str, minimum: &str) -> bool {
    let parse = |text: &str| {
        text.split(|c: char| !c.is_ascii_digit() && c != '.')
            .find(|part| part.contains('.'))
            .unwrap_or(text)
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let left = parse(actual);
    let right = parse(minimum);
    let width = left.len().max(right.len());
    let at = |values: &[u64], index: usize| values.get(index).copied().unwrap_or(0);
    (0..width)
        .find_map(|index| match at(&left, index).cmp(&at(&right, index)) {
            std::cmp::Ordering::Equal => None,
            other => Some(other == std::cmp::Ordering::Greater),
        })
        .unwrap_or(true)
}
