//! First line of an RTSP message: `METHOD uri RTSP/1.0` or `RTSP/1.0 code reason`.

use crate::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    Request { method: String, uri: String },
    Response { code: u16, reason: String },
}

impl fmt::Display for Start {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Start::Request { method, uri } => write!(f, "{method} {uri} RTSP/1.0"),
            Start::Response { code, reason } => write!(f, "RTSP/1.0 {code} {reason}"),
        }
    }
}

/// Tolerant of extra spaces and of `rtsp/1.0` in lower case; methods come out upper case
/// so `get_parameter` from a lax sink is still a GET_PARAMETER.
pub(crate) fn parse_start(line: &str) -> Result<Start, Error> {
    let bad = || Error::Malformed(line.to_owned());
    let mut parts = line.split_whitespace();
    let (a, b) = (parts.next().ok_or_else(bad)?, parts.next().ok_or_else(bad)?);
    let rest = parts.collect::<Vec<_>>().join(" ");
    if is_version(a) {
        let code = b.parse().map_err(|_| bad())?;
        return Ok(Start::Response { code, reason: rest });
    }
    if is_version(&rest) {
        return Ok(Start::Request { method: a.to_ascii_uppercase(), uri: b.to_owned() });
    }
    Err(bad())
}

fn is_version(word: &str) -> bool {
    word.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("RTSP/"))
}
