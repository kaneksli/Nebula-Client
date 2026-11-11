/// Small centroid table for the countries that actually show up in VPN
/// subscriptions in practice. Good enough to place the globe marker in the
/// right country; not survey-grade geodesy.
const CENTROIDS: &[(&str, f64, f64)] = &[
    ("DE", 51.16, 10.45), ("NL", 52.13, 5.29), ("FI", 61.92, 25.75),
    ("FR", 46.60, 1.89), ("ES", 40.46, -3.75), ("PL", 51.92, 19.15),
    ("SE", 60.13, 18.64), ("CH", 46.82, 8.23), ("GB", 55.38, -3.44),
    ("IE", 53.41, -8.24), ("IT", 41.87, 12.57), ("PT", 39.40, -8.22),
    ("AT", 47.52, 14.55), ("CZ", 49.82, 15.47), ("RO", 45.94, 24.97),
    ("BG", 42.73, 25.49), ("GR", 39.07, 21.82), ("TR", 38.96, 35.24),
    ("UA", 48.38, 31.17), ("RU", 61.52, 105.32), ("EE", 58.60, 25.01),
    ("LV", 56.88, 24.60), ("LT", 55.17, 23.88), ("NO", 60.47, 8.47),
    ("DK", 56.26, 9.50), ("US", 37.09, -95.71), ("CA", 56.13, -106.35),
    ("MX", 23.63, -102.55), ("BR", -14.24, -51.93), ("AR", -38.42, -63.62),
    ("JP", 36.20, 138.25), ("KR", 35.91, 127.77), ("CN", 35.86, 104.20),
    ("HK", 22.32, 114.17), ("TW", 23.70, 120.96), ("SG", 1.35, 103.82),
    ("IN", 20.59, 78.96), ("ID", -0.79, 113.92), ("VN", 14.06, 108.28),
    ("TH", 15.87, 100.99), ("MY", 4.21, 101.98), ("PH", 12.88, 121.77),
    ("AE", 23.42, 53.85), ("IL", 31.05, 34.85), ("SA", 23.89, 45.08),
    ("EG", 26.82, 30.80), ("ZA", -30.56, 22.94), ("NG", 9.08, 8.68),
    ("AU", -25.27, 133.78), ("NZ", -40.90, 174.89), ("KZ", 48.02, 66.92),
];

/// Recognizes a two-letter ISO country code in a server remark (e.g.
/// "DE-01 | Frankfurt" or "🇩🇪 Germany"), returning its flag emoji and the
/// code itself; falls back to a globe emoji and no country when nothing
/// matches rather than guessing.
pub fn flag_and_country_from_remark(remark: &str) -> (String, Option<String>) {
    let emoji_flag = remark.chars().find(|c| {
        let cp = *c as u32;
        (0x1F1E6..=0x1F1FF).contains(&cp)
    });
    if emoji_flag.is_some() {
        // remark already carries a flag emoji from the subscription itself;
        // keep the whole remark as the display name and surface that flag.
        let flag: String = remark
            .chars()
            .filter(|c| (0x1F1E6..=0x1F1FF).contains(&(*c as u32)))
            .collect();
        return (flag, Some(remark.to_string()));
    }

    let upper = remark.to_uppercase();
    for (code, _, _) in CENTROIDS {
        if upper.split(|c: char| !c.is_ascii_alphanumeric()).any(|tok| tok == *code) {
            return (code_to_flag(code), Some(remark.to_string()));
        }
    }
    ("🌐".to_string(), None)
}

pub fn lookup_country(remark_or_code: &str) -> Option<(f64, f64)> {
    let upper = remark_or_code.to_uppercase();
    CENTROIDS
        .iter()
        .find(|(code, _, _)| upper.split(|c: char| !c.is_ascii_alphanumeric()).any(|tok| tok == *code))
        .map(|(_, lat, lng)| (*lat, *lng))
}

fn code_to_flag(code: &str) -> String {
    code.chars()
        .map(|c| char::from_u32(0x1F1E6 + (c as u32 - 'A' as u32)).unwrap_or(c))
        .collect()
}
