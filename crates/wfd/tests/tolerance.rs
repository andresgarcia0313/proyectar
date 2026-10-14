//! Format choice and parsing against the odd strings receivers send.

use wfd::{audio, params, rtsp::Message, video, Table, VideoMode};

#[test]
fn handheld_modes_and_their_place_in_m4() {
    let phone = "00 00 01 02 00000001 00000000 00000300 00 0000 0000 00 none none";
    let m = video::choose(phone, 1080, 60).unwrap();
    assert_eq!((m.width, m.height, m.fps, m.table()), (960, 540, 60, Table::Hh));
    assert!(m.to_param().starts_with("00 00 01 02 00000000 00000000 00000200 "), "{}", m.to_param());
}

#[test]
fn modes_beyond_the_declared_level_are_skipped() {
    let lax = "00 00 01 01 000001e0 00000000 00000000 00 0000 0000 00 none none";
    let m = video::choose(lax, 1080, 60).unwrap();
    assert_eq!((m.width, m.height, m.fps), (1280, 720, 30), "level 3.1 carries 720p30 at most");
    let vesa_1200 = "00 00 01 10 00000001 30000000 00000000 00 0000 0000 00 none none";
    assert_eq!(video::choose(vesa_1200, 1200, 60).unwrap().height, 480, "1920x1200 needs level 5");
}

#[test]
fn mandatory_fallbacks() {
    assert_eq!(video::choose("", 1080, 60).unwrap(), VideoMode::FALLBACK);
    assert_eq!(video::choose("garbage, more", 1080, 60).unwrap(), VideoMode::FALLBACK);
    let only_4k = "00 00 01 10 00020000 00000000 00000000 00 0000 0000 00 none none";
    assert_eq!(video::choose(only_4k, 1080, 60).unwrap(), VideoMode::FALLBACK);
    let no_profile = "00 00 00 00 00000040 00000000 00000000 00 0000 0000 00 none none";
    assert_eq!(video::choose(no_profile, 720, 60).unwrap().profile, 0x01, "CBP is mandatory");
    assert!(matches!(video::choose(" None ", 1080, 60), Err(wfd::Error::NoCommonFormat("video"))));
    assert_eq!(audio::choose("LPCM 00000001 00", false), Some(wfd::AudioCodec::Lpcm), "44.1 kHz only");
    assert_eq!(audio::choose("", false), None);
}

#[test]
fn rtp_port_variants() {
    for (text, port) in [
        ("RTP/AVP/UDP;unicast 19000 0 mode=play", Some(19000)),
        ("rtp/avp/udp;unicast   1028   0   mode=play", Some(1028)),
        ("RTP/AVP/UDP; unicast 1990 0 mode=play", Some(1990)),
        ("RTP/AVP/UDP;unicast 19000-19001 0 mode=play", Some(19000)),
        ("RTP/AVP/UDP;unicast;Client_Port=5004-5005", Some(5004)),
        ("RTP/AVP/UDP;unicast 0 0 mode=play", None),
        ("none", None),
    ] {
        assert_eq!(params::rtp_port(text), port, "{text}");
    }
}

#[tokio::test]
async fn lax_framing_still_reads() {
    let raw = b"rtsp/1.0  200   OK\ncseq:  9\nX-Vendor junk without colon\ncontent-length: 7\n\na: b\nc\nget_parameter  rtsp://localhost/wfd1.0  RTSP/1.0\r\nCSeq: 2\r\n\r\n";
    let mut r = &raw[..];
    let first = Message::read(&mut r).await.unwrap().unwrap();
    assert!(first.is_ok());
    assert_eq!((first.cseq(), params::get(&first.body, "A")), (9, Some("b")));
    let second = Message::read(&mut r).await.unwrap().unwrap();
    assert_eq!((second.method(), second.cseq()), (Some("GET_PARAMETER"), 2));
}

#[test]
fn vendor_parameters_are_ignored_for_the_choice() {
    let caps = "microsoft_cursor: full\r\nwfd_video_formats: 00 00 01 10 00000040 00000000 00000000 00 0000 0000 00 none none\r\nintel_sink_version: product_ID=XYZ\r\nwfd2_audio_codecs: AAC 00000001 00\r\n";
    let m = video::choose(params::get(caps, "WFD_VIDEO_FORMATS").unwrap(), 1080, 60).unwrap();
    assert_eq!((m.width, m.fps), (1280, 60));
    assert!(params::mentions("WFD_IDR_REQUEST\r\n", "wfd_idr_request"));
}
