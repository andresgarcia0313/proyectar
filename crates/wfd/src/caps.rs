//! M3 and M4: what the sink can take and the single format the source picks.

use crate::link::Link;
use crate::rtsp::Message;
use crate::session::{Negotiated, Wanted};
use crate::{audio, params, video, AudioCodec, Error, VideoMode};
use tokio::io::{AsyncRead, AsyncWrite};

/// Same order as Android's source, which every certified sink was tested against.
pub const MANDATORY: [&str; 4] =
    ["wfd_content_protection", "wfd_video_formats", "wfd_audio_codecs", "wfd_client_rtp_ports"];
/// What Windows also asks; a sink may omit them or answer `none`.
const OPTIONAL: [&str; 4] =
    ["wfd_display_edid", "wfd_connector_type", "wfd_preferred_display_mode", "wfd_standby_resume_capability"];
/// Vendor extensions, asked only of a sink that names its vendor in M1 or M2.
const VENDOR: [&str; 3] =
    ["intel_friendly_name", "microsoft_latency_management_capability", "microsoft_format_change_capability"];
const URI: &str = "rtsp://localhost/wfd1.0";

/// M3. A sink that refuses names it does not know (some dongles answer 400 or 551) is
/// asked again for the mandatory ones only.
pub async fn ask<S: AsyncRead + AsyncWrite>(link: &mut Link<S>, vendor: bool) -> Result<String, Error> {
    let extra = OPTIONAL.iter().chain(VENDOR.iter().filter(|_| vendor));
    let full: Vec<&str> = MANDATORY.iter().chain(extra).copied().collect();
    match get(link, &full).await {
        Err(Error::Refused { .. }) => get(link, &MANDATORY).await,
        other => other,
    }
}

async fn get<S: AsyncRead + AsyncWrite>(link: &mut Link<S>, names: &[&str]) -> Result<String, Error> {
    let body: String = names.iter().map(|n| format!("{n}\r\n")).collect();
    let m3 = Message::request("GET_PARAMETER", URI, link.next_cseq()).with_body(body);
    Ok(link.call("M3", m3).await?.body)
}

/// The best common format; a missing parameter counts as `none` for audio and as the
/// mandatory 640x480p60 for video, since every sink must decode that.
pub fn choose(caps: &str, wanted: &Wanted) -> Result<Negotiated, Error> {
    let formats = params::get(caps, "wfd_video_formats").unwrap_or("");
    let video = video::choose(formats, wanted.max_height, wanted.max_fps)?;
    let codecs = params::get(caps, "wfd_audio_codecs").unwrap_or("none");
    let audio = if wanted.audio { audio::choose(codecs, wanted.lpcm) } else { None };
    let ports = params::get(caps, "wfd_client_rtp_ports").and_then(params::rtp_port);
    Ok(Negotiated { video, audio, rtp_port: ports.unwrap_or(19000) })
}

/// M4. A sink that refuses the pick gets the mandatory formats once: 640x480p60 CBP 3.1
/// and LPCM 48 kHz stereo.
pub async fn offer<S: AsyncRead + AsyncWrite>(
    link: &mut Link<S>,
    url: &str,
    got: &mut Negotiated,
) -> Result<(), Error> {
    match set(link, url, got).await {
        Err(Error::Refused { .. })
            if got.video != VideoMode::FALLBACK || got.audio == Some(AudioCodec::Aac) =>
        {
            got.video = VideoMode::FALLBACK;
            got.audio = got.audio.map(|_| AudioCodec::Lpcm);
            set(link, url, got).await
        }
        other => other,
    }
}

async fn set<S: AsyncRead + AsyncWrite>(
    link: &mut Link<S>,
    url: &str,
    got: &Negotiated,
) -> Result<(), Error> {
    let mut chosen = vec![("wfd_video_formats", got.video.to_param())];
    if let Some(a) = got.audio {
        chosen.push(("wfd_audio_codecs", a.to_param().into()));
    }
    chosen.push(("wfd_presentation_URL", format!("{url} none")));
    chosen.push(("wfd_client_rtp_ports", format!("RTP/AVP/UDP;unicast {} 0 mode=play", got.rtp_port)));
    let m4 = Message::request("SET_PARAMETER", URI, link.next_cseq()).with_body(params::body(&chosen));
    link.call("M4", m4).await.map(drop)
}

/// Whether the sink names Intel or Microsoft in a header of M1's reply or of M2.
pub fn names_vendor(msg: &Message) -> bool {
    let vendor = |v: &str| ["microsoft", "intel"].iter().any(|n| v.to_ascii_lowercase().contains(n));
    msg.headers.iter().any(|(k, v)| vendor(k) || vendor(v))
}
