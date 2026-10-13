//! Capability negotiation as the source, M1 to M7 (WFD spec, section 6.4).

use crate::link::{default_answer, Link, PUBLIC};
use crate::rtsp::Message;
use crate::{audio, params, video, AudioCodec, Error, VideoMode};
use tokio::io::{AsyncRead, AsyncWrite};

/// Seconds the sink may stay silent; we send keep-alives well before it expires.
pub const SESSION_TIMEOUT: u64 = 30;
const SESSION_ID: &str = "6B8B4567";
/// Same order as Android's source, which every certified sink was tested against.
const M3_ASK: &str =
    "wfd_content_protection\r\nwfd_video_formats\r\nwfd_audio_codecs\r\nwfd_client_rtp_ports\r\n";

/// Limits the user chose (latency mode) and where the stream comes from.
#[derive(Debug, Clone)]
pub struct Wanted {
    pub max_height: u32,
    pub max_fps: u32,
    pub audio: bool,
    /// Our address on the Wi-Fi Direct link, for the presentation URL.
    pub local_ip: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Negotiated {
    pub video: VideoMode,
    pub audio: Option<AudioCodec>,
    pub rtp_port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Play,
    Pause,
    /// The sink lost a frame and needs a full picture now.
    IdrRequest,
    Teardown,
}

pub struct Session<S> {
    pub(crate) link: Link<S>,
    pub(crate) url: String,
    /// When the next keep-alive goes out, and the one still waiting for its answer.
    pub(crate) ping_at: Option<tokio::time::Instant>,
    pub(crate) pending: Option<(u32, tokio::time::Instant)>,
}

impl<S: AsyncRead + AsyncWrite> Session<S> {
    /// Runs M1-M7 on a connection the sink opened to our port 7236.
    pub async fn negotiate(stream: S, wanted: &Wanted) -> Result<(Self, Negotiated), Error> {
        let mut link = Link::new(stream);
        let url = format!("rtsp://{}/wfd1.0/streamid=0", wanted.local_ip);

        let m1 = Message::request("OPTIONS", "*", link.next_cseq()).header("Require", "org.wfa.wfd1.0");
        link.call("M1", m1, default_answer).await?;
        let m2 = link.expect("M2", "OPTIONS", default_answer).await?;
        link.send(&Message::reply(&m2, 200, "OK").header("Public", PUBLIC)).await?;

        let m3 = Message::request("GET_PARAMETER", "rtsp://localhost/wfd1.0", link.next_cseq())
            .with_body(M3_ASK.into());
        let caps = link.call("M3", m3, default_answer).await?.body;
        let formats = params::get(&caps, "wfd_video_formats").unwrap_or("none");
        let video = video::choose(formats, wanted.max_height, wanted.max_fps)?;
        let audio = wanted.audio.then(|| audio::choose(params::get(&caps, "wfd_audio_codecs")?)).flatten();
        let ports = params::get(&caps, "wfd_client_rtp_ports").unwrap_or("");
        let mut rtp_port = params::rtp_port(ports).unwrap_or(19000);

        let mut chosen = vec![
            ("wfd_video_formats", video.to_param()),
            ("wfd_presentation_URL", format!("{url} none")),
            ("wfd_client_rtp_ports", format!("RTP/AVP/UDP;unicast {rtp_port} 0 mode=play")),
        ];
        if let Some(a) = audio {
            chosen.insert(1, ("wfd_audio_codecs", a.to_param().into()));
        }
        let m4 = Message::request("SET_PARAMETER", "rtsp://localhost/wfd1.0", link.next_cseq())
            .with_body(params::body(&chosen));
        link.call("M4", m4, default_answer).await?;

        let mut session = Self { link, url, ping_at: None, pending: None };
        session.trigger("M5", "SETUP").await?;
        let m6 = session.link.expect("M6", "SETUP", default_answer).await?;
        let transport = m6.get("Transport").unwrap_or("RTP/AVP/UDP;unicast").to_owned();
        rtp_port = params::rtp_port(&transport).unwrap_or(rtp_port);
        let reply = Message::reply(&m6, 200, "OK")
            // No `;timeout=`: several sinks misread it (gnome-network-displays drops it too).
            .header("Session", SESSION_ID)
            .header("Transport", format!("{transport};server_port=5000-5001"));
        session.link.send(&reply).await?;

        let m7 = session.link.expect("M7", "PLAY", default_answer).await?;
        let reply = Message::reply(&m7, 200, "OK").header("Session", SESSION_ID);
        session.link.send(&reply).await?;
        Ok((session, Negotiated { video, audio, rtp_port }))
    }

    /// M5: asks the sink to send SETUP, PLAY, PAUSE or TEARDOWN.
    pub async fn trigger(&mut self, step: &'static str, method: &str) -> Result<(), Error> {
        let body = params::body(&[("wfd_trigger_method", method.into())]);
        let req = Message::request("SET_PARAMETER", "rtsp://localhost/wfd1.0", self.link.next_cseq())
            .with_body(body);
        self.link.call(step, req, default_answer).await.map(drop)
    }
}
