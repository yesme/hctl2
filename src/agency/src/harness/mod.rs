//! Minimum versions. A harness is cataloged only after a startup smoke check.
//! Herdr does not use this gate; it uses the locked binary and protocol 20.
pub mod claude;

pub const CLAUDE_MINIMUM: &str = "2.1.263";
pub const CODEX_MINIMUM: &str = "0.153.4";

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
