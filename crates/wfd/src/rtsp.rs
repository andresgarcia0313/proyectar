//! RTSP 1.0 messages as Wi-Fi Display uses them: one request or response per exchange,
//! `CSeq` matching, and an optional `text/parameters` body sized by `Content-Length`.

pub use crate::start::Start;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub start: Start,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Message {
    pub fn request(method: &str, uri: &str, cseq: u32) -> Self {
        let start = Start::Request { method: method.into(), uri: uri.into() };
        Self { start, headers: vec![], body: String::new() }.header("CSeq", cseq)
    }

    pub fn reply(to: &Message, code: u16, reason: &str) -> Self {
        let start = Start::Response { code, reason: reason.into() };
        let cseq = to.get("CSeq").unwrap_or("0").to_owned();
        Self { start, headers: vec![], body: String::new() }.header("CSeq", cseq)
    }

    pub fn header(mut self, name: &str, value: impl ToString) -> Self {
        self.headers.push((name.into(), value.to_string()));
        self
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self.header("Content-Type", "text/parameters")
    }

    /// Header lookup is case-insensitive, as RFC 2326 requires.
    pub fn get(&self, name: &str) -> Option<&str> {
        let found = self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name));
        found.map(|(_, v)| v.as_str())
    }

    pub fn cseq(&self) -> u32 {
        self.get("CSeq").and_then(|v| v.trim().parse().ok()).unwrap_or(0)
    }

    pub fn method(&self) -> Option<&str> {
        match &self.start {
            Start::Request { method, .. } => Some(method),
            Start::Response { .. } => None,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self.start, Start::Response { code: 200, .. })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = format!("{}\r\n", self.start);
        for (k, v) in &self.headers {
            out += &format!("{k}: {v}\r\n");
        }
        if !self.body.is_empty() {
            out += &format!("Content-Length: {}\r\n", self.body.len());
        }
        out += "\r\n";
        out += &self.body;
        out.into_bytes()
    }
}
