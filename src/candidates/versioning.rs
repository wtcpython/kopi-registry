use std::cmp::Reverse;

pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by_key(|version| Reverse(sort_key(version)));
}

pub fn sort_dedup(mut versions: Vec<String>) -> Vec<String> {
    sort_versions(&mut versions);
    versions.dedup();
    versions
}

pub fn is_version_like(value: &str) -> bool {
    !value.is_empty()
        && value.starts_with(|c: char| c.is_ascii_digit())
        && value.contains('.')
        && value.chars().all(is_version_char)
}

fn is_version_char(value: char) -> bool {
    value.is_ascii_digit()
        || matches!(
            value,
            '.' | '-' | '_' | '+'
                | 'a'..='z'
                | 'A'..='Z'
        )
}

fn sort_key(version: &str) -> (i32, Vec<u32>) {
    let (base, prerelease) = match version.split_once(['-', '+', '_']) {
        Some((base, _)) => (base, 0i32),
        None => (version, 1i32),
    };
    let numbers = base
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect();
    (prerelease, numbers)
}
