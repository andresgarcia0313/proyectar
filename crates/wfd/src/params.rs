//! `text/parameters` bodies: one `name: value` per line, or bare names in a GET_PARAMETER.

/// Value of `name` in a parameters body, trimmed.
pub fn get<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    body.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim())
    })
}

/// Bare parameter names a GET_PARAMETER asks for.
pub fn names(body: &str) -> impl Iterator<Item = &str> {
    body.lines().map(str::trim).filter(|l| !l.is_empty() && !l.contains(':'))
}

/// Whether a body names `name`, bare or with a value, in any letter case.
pub fn mentions(body: &str, name: &str) -> bool {
    body.lines().any(|l| l.split(':').next().unwrap_or("").trim().eq_ignore_ascii_case(name))
}

/// RTP port the sink listens on, from `RTP/AVP/UDP;unicast 19000 0 mode=play` (also with
/// extra spaces, `; unicast`, `19000-19001` or lower case) or from a SETUP `Transport`
/// header with `client_port=19000[-19001]`. `None` for `none` or port 0.
pub fn rtp_port(value: &str) -> Option<u16> {
    let lower = value.to_ascii_lowercase();
    let digits = match lower.find("client_port=") {
        Some(i) => &lower[i + "client_port=".len()..],
        None => lower
            .split(|c: char| c.is_whitespace() || c == ';')
            .find(|t| t.starts_with(|c: char| c.is_ascii_digit()))?,
    };
    let end = digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len());
    digits[..end].parse().ok().filter(|p| *p != 0)
}

/// Builds a body from `name: value` pairs, CRLF terminated as sinks expect.
pub fn body(pairs: &[(&str, String)]) -> String {
    pairs.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect()
}
