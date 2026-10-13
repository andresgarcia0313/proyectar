//! Capability negotiation as the source, M1 to M7 (WFD spec v2.1, section 6.4).

use crate::link::{Link, REPLY_WAIT};
use crate::rtsp::Message;
use crate::sink_info::SinkInfo;
use crate::{caps, params, AudioCodec, Error, VideoMode};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

/// Seconds the sink may stay silent, announced in M6; keep-alives go out at half of it.
pub const SESSION_TIMEOUT: u64 = 30;
pub(crate) const SESSION_ID: &str = "6B8B4567";
/// M2 is mandatory, but a sink that skips it is not worth losing: after this we go on.
const M2_WAIT: Duration = Duration::from_secs(5);

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
    /// The sink sent PLAY, also when it comes back from standby.
    Play,
    /// The sink sent PAUSE or announced `wfd_standby`: it stopped showing the stream.
    Pause,
    /// The sink lost a frame and needs a full picture now.
    IdrRequest,
    Teardown,
}

pub struct Session<S> {
    pub(crate) link: Link<S>,
    pub(crate) info: SinkInfo,
    /// When the next keep-alive goes out, the one still waiting for its answer, and how
    /// many in a row went unanswered.
    pub(crate) ping_at: Option<tokio::time::Instant>,
    pub(crate) pending: Option<(u32, tokio::time::Instant)>,
    pub(crate) misses: u8,
}

impl<S: AsyncRead + AsyncWrite> Session<S> {
    /// Runs M1-M7 on a connection the sink opened to our port 7236. Requests the sink sends
    /// ahead of time (M2 before answering M1, SETUP before answering M5) are kept for their
    /// turn instead of being lost.
    pub async fn negotiate(stream: S, wanted: &Wanted) -> Result<(Self, Negotiated), Error> {
        let mut link = Link::new(stream);
        let url = format!("rtsp://{}/wfd1.0/streamid=0", wanted.local_ip);

        link.hold = &["OPTIONS", "SETUP", "PLAY"];
        let m1 = Message::request("OPTIONS", "*", link.next_cseq()).header("Require", "org.wfa.wfd1.0");
        let mut vendor = caps::names_vendor(&link.call("M1", m1).await?);
        link.hold = &["SETUP", "PLAY"];
        match link.expect("M2", "OPTIONS", M2_WAIT).await {
            Ok(m2) => {
                vendor |= caps::names_vendor(&m2);
                link.send(&link.answer(&m2)).await?;
            }
            Err(Error::Timeout(_)) => {}
            Err(e) => return Err(e),
        }

        let found = caps::ask(&mut link, vendor).await?;
        let mut got = caps::choose(&found, wanted)?;
        caps::offer(&mut link, &url, &mut got).await?;

        let info = SinkInfo::from_caps(&found);
        let mut session = Self { link, info, ping_at: None, pending: None, misses: 0 };
        session.trigger("M5", "SETUP").await?;
        let m6 = session.link.expect("M6", "SETUP", REPLY_WAIT).await?;
        let transport = m6.get("Transport").unwrap_or("RTP/AVP/UDP;unicast").to_owned();
        got.rtp_port = params::rtp_port(&transport).unwrap_or(got.rtp_port);
        let reply = Message::reply(&m6, 200, "OK")
            .header("Session", format!("{SESSION_ID};timeout={SESSION_TIMEOUT}"))
            .header("Transport", format!("{transport};server_port=5000-5001"));
        session.link.send(&reply).await?;

        session.link.hold = &[];
        let m7 = session.link.expect("M7", "PLAY", REPLY_WAIT).await?;
        session.link.session = Some(SESSION_ID);
        session.link.send(&session.link.answer(&m7)).await?;
        Ok((session, got))
    }

    /// What the sink told about itself in M3: name, connector, EDID, vendor parameters.
    pub fn sink_info(&self) -> &SinkInfo {
        &self.info
    }
}
