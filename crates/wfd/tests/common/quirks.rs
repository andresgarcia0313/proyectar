//! Ways real receivers bend the protocol, and the M1-M7 script that applies them.

use super::Sink;
use wfd::rtsp::{Message, Start};

#[derive(Debug, Clone, Default)]
pub struct Quirks {
    /// Header names and the version in lower case (`cseq`, `rtsp/1.0`).
    pub lower: bool,
    /// Bare LF line ends.
    pub lf: bool,
    /// `Content-Length` written twice, as Samsung does.
    pub dup_len: bool,
    /// Extra spaces after `:` and between the words of the first line.
    pub spaces: bool,
    /// M2 sent before answering M1.
    pub m2_early: bool,
    /// No M2 at all.
    pub skip_m2: bool,
    /// SETUP sent before answering the M5 that asks for it.
    pub setup_early: bool,
    /// 551 to an M3 asking for anything beyond the four mandatory names.
    pub refuse_extra: bool,
    /// 400 to an M4 that is not 640x480p60.
    pub refuse_m4: bool,
    /// `Public` in the M1 reply.
    pub public: &'static str,
}

impl Quirks {
    pub fn render(&self, m: &Message) -> String {
        let nl = if self.lf { "\n" } else { "\r\n" };
        let (sep, gap) = if self.spaces { (":   ", "  ") } else { (": ", " ") };
        let version = if self.lower { "rtsp/1.0" } else { "RTSP/1.0" };
        let mut out = match &m.start {
            Start::Request { method, uri } => format!("{method}{gap}{uri}{gap}{version}{nl}"),
            Start::Response { code, reason } => format!("{version}{gap}{code}{gap}{reason}{nl}"),
        };
        let name = |k: &str| if self.lower { k.to_ascii_lowercase() } else { k.to_owned() };
        for (k, v) in &m.headers {
            out += &format!("{}{sep}{v}{nl}", name(k));
        }
        let body = if self.lf { m.body.replace("\r\n", "\n") } else { m.body.clone() };
        if !body.is_empty() {
            let len = format!("{}{sep}{}{nl}", name("Content-Length"), body.len());
            out += &if self.dup_len { len.repeat(2) } else { len };
        }
        out + nl + &body
    }
}

pub async fn handshake(s: &mut Sink, caps: String) -> String {
    let q = s.quirks.clone();
    let m1 = s.read().await;
    assert_eq!(m1.get("Require"), Some("org.wfa.wfd1.0"));
    if q.m2_early {
        let m2 = s.next_request("OPTIONS", "");
        s.send(m2).await;
    }
    let public = if q.public.is_empty() { "org.wfa.wfd1.0, GET_PARAMETER, SET_PARAMETER" } else { q.public };
    s.send(Message::reply(&m1, 200, "OK").header("Public", public)).await;
    if q.m2_early {
        let reply = s.read().await;
        assert!(reply.cseq() == s.cseq && reply.get("Public").unwrap().contains("SETUP"), "{reply:?}");
    } else if !q.skip_m2 {
        let m2 = s.request("OPTIONS", "").await;
        assert!(m2.get("Public").unwrap().contains("SETUP"));
    }
    let mut m3 = s.read().await;
    assert!(m3.body.contains("wfd_video_formats"));
    if q.refuse_extra && m3.body.lines().count() > 4 {
        s.send(Message::reply(&m3, 551, "Option not supported")).await;
        m3 = s.read().await;
    }
    s.asked = m3.body.clone();
    s.send(Message::reply(&m3, 200, "OK").with_body(caps)).await;
    let mut m4 = s.read().await;
    if q.refuse_m4 && !m4.body.contains(" 00000001 00000000 00000000 ") {
        s.send(Message::reply(&m4, 400, "Bad Request")).await;
        m4 = s.read().await;
    }
    s.answer(&m4).await;
    let m5 = s.read().await;
    assert!(m5.body.contains("wfd_trigger_method: SETUP"));
    if !q.setup_early {
        s.answer(&m5).await;
    }
    let setup = s.next_request("SETUP", "").header("Transport", "RTP/AVP/UDP;unicast;client_port=1028");
    s.send(setup).await;
    if q.setup_early {
        s.answer(&m5).await;
    }
    let m6 = s.read().await;
    assert_eq!(m6.get("Session"), Some("6B8B4567;timeout=30"));
    assert!(m6.get("Transport").unwrap().contains("client_port=1028"));
    let m7 = s.request("PLAY", "").await;
    assert_eq!(m7.get("Session"), Some("6B8B4567"));
    m4.body
}
