//! Semver version requirement resolver for `fav add` / `fav update` (v17.8.0).

/// A parsed semantic version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// A version constraint parsed from a `fav.toml` dependency string.
#[derive(Debug, Clone)]
pub enum VersionReq {
    /// `^2.0.0` — same major (or same minor when major == 0).
    Caret(SemVer),
    /// `~2.1.0` — same major.minor.
    Tilde(SemVer),
    /// `=2.1.3` — exact version.
    Exact(SemVer),
    /// `*` — any version.
    Any,
}

/// Parse `"2.1.0"` → `Some(SemVer { 2, 1, 0 })`.
pub fn parse_semver(s: &str) -> Option<SemVer> {
    let s = s.trim();
    let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() < 3 {
        return None;
    }
    Some(SemVer {
        major: parts[0],
        minor: parts[1],
        patch: parts[2],
    })
}

/// Parse a version constraint string.
/// Handles `^`, `~`, `=`, `*`, and bare versions (treated as `^`).
pub fn parse_version_req(s: &str) -> Option<VersionReq> {
    let s = s.trim();
    if s == "*" {
        return Some(VersionReq::Any);
    }
    if let Some(rest) = s.strip_prefix('^') {
        return parse_semver(rest).map(VersionReq::Caret);
    }
    if let Some(rest) = s.strip_prefix('~') {
        return parse_semver(rest).map(VersionReq::Tilde);
    }
    if let Some(rest) = s.strip_prefix('=') {
        return parse_semver(rest).map(VersionReq::Exact);
    }
    // Bare version string — treat as `^`
    parse_semver(s).map(VersionReq::Caret)
}

/// Return true if `v` satisfies `req`.
pub fn matches_req(req: &VersionReq, v: &SemVer) -> bool {
    match req {
        VersionReq::Any => true,
        VersionReq::Exact(base) => v == base,
        VersionReq::Tilde(base) => {
            v.major == base.major && v.minor == base.minor && *v >= *base
        }
        VersionReq::Caret(base) => {
            if base.major > 0 {
                // ^x.y.z (x >= 1): same major, version >= base
                v.major == base.major && *v >= *base
            } else if base.minor > 0 {
                // ^0.y.z (y >= 1): same minor, version >= base
                v.major == 0 && v.minor == base.minor && *v >= *base
            } else {
                // ^0.0.z: exact match
                v == base
            }
        }
    }
}

/// Pick the highest version from `available` that satisfies `req`.
/// Returns `None` if no version satisfies the requirement.
pub fn resolve_best(req: &VersionReq, available: &[SemVer]) -> Option<SemVer> {
    available
        .iter()
        .filter(|v| matches_req(req, v))
        .max()
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_major_one() {
        let req = parse_version_req("^2.0.0").unwrap();
        let avail = vec![
            parse_semver("1.9.9").unwrap(),
            parse_semver("2.0.0").unwrap(),
            parse_semver("2.1.0").unwrap(),
            parse_semver("3.0.0").unwrap(),
        ];
        let best = resolve_best(&req, &avail).unwrap();
        assert_eq!(best, SemVer { major: 2, minor: 1, patch: 0 });
    }

    #[test]
    fn caret_minor_zero() {
        let req = parse_version_req("^0.3.0").unwrap();
        let avail = vec![
            parse_semver("0.3.0").unwrap(),
            parse_semver("0.3.5").unwrap(),
            parse_semver("0.4.0").unwrap(),
        ];
        let best = resolve_best(&req, &avail).unwrap();
        assert_eq!(best, SemVer { major: 0, minor: 3, patch: 5 });
    }

    #[test]
    fn tilde_req() {
        let req = parse_version_req("~1.2.0").unwrap();
        let avail = vec![
            parse_semver("1.2.0").unwrap(),
            parse_semver("1.2.3").unwrap(),
            parse_semver("1.3.0").unwrap(),
        ];
        let best = resolve_best(&req, &avail).unwrap();
        assert_eq!(best, SemVer { major: 1, minor: 2, patch: 3 });
    }

    #[test]
    fn exact_req() {
        let req = parse_version_req("=2.1.3").unwrap();
        let avail = vec![
            parse_semver("2.1.2").unwrap(),
            parse_semver("2.1.3").unwrap(),
            parse_semver("2.1.4").unwrap(),
        ];
        let best = resolve_best(&req, &avail).unwrap();
        assert_eq!(best, SemVer { major: 2, minor: 1, patch: 3 });
    }

    #[test]
    fn wildcard_req() {
        let req = parse_version_req("*").unwrap();
        let avail = vec![
            parse_semver("1.0.0").unwrap(),
            parse_semver("3.5.2").unwrap(),
            parse_semver("2.0.0").unwrap(),
        ];
        let best = resolve_best(&req, &avail).unwrap();
        assert_eq!(best, SemVer { major: 3, minor: 5, patch: 2 });
    }

    #[test]
    fn no_match_returns_none() {
        let req = parse_version_req("^5.0.0").unwrap();
        let avail = vec![parse_semver("1.0.0").unwrap(), parse_semver("4.9.9").unwrap()];
        assert!(resolve_best(&req, &avail).is_none());
    }
}
