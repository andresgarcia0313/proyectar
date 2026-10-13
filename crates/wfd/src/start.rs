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

pub(crate) fn parse_start(line: &str) -> Result<Start, Error> {
    let bad = || Error::Malformed(line.to_owned());
    let mut parts = line.splitn(3, ' ');
    let (a, b, c) = (parts.next().ok_or_else(bad)?, parts.next().ok_or_else(bad)?, parts.next());
    if a.starts_with("RTSP/") {
        let code = b.parse().map_err(|_| bad())?;
        return Ok(Start::Response { code, reason: c.unwrap_or("").to_owned() });
    }
    if c.is_some_and(|v| v.starts_with("RTSP/")) {
        return Ok(Start::Request { method: a.to_owned(), uri: b.to_owned() });
    }
    Err(bad())
}
