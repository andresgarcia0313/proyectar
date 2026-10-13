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

/// RTP port the sink listens on, from `RTP/AVP/UDP;unicast 19000 0 mode=play`
/// or from a SETUP `Transport` header with `client_port=19000[-19001]`.
pub fn rtp_port(value: &str) -> Option<u16> {
    if let Some(i) = value.find("client_port=") {
        let digits = &value[i + "client_port=".len()..];
        let end = digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len());
        return digits[..end].parse().ok();
    }
    value.split_whitespace().nth(1)?.parse().ok().filter(|p| *p != 0)
}

/// Builds a body from `name: value` pairs, CRLF terminated as sinks expect.
pub fn body(pairs: &[(&str, String)]) -> String {
    pairs.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect()
}
