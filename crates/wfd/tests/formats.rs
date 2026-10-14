//! Capability parsing and choice, with strings taken from real sinks.

use wfd::{audio, ie, params, rtsp::Message, video};

#[test]
fn caps_resolution_by_the_latency_mode() {
    let sink = "00 00 02 10 0001ffff 3fffffff 00000fff 00 0000 0000 11 none none";
    let m = video::choose(sink, 720, 60).unwrap();
    assert_eq!((m.width, m.height, m.fps, m.profile), (1280, 720, 60, 0x02), "CHP-only sink");
    let m = video::choose(sink, 1080, 30).unwrap();
    assert_eq!((m.width, m.height, m.fps), (1920, 1080, 30));
    assert_eq!(m.h264_level(), "4.2");
}

#[test]
fn baseline_only_sink_with_several_codec_entries() {
    let sink = "40 00 01 01 00000021 00000000 00000000 00 0000 0000 00 none none, 01 02 00000040 00000000 00000000 00 0000 0000 00 none none";
    let m = video::choose(sink, 1080, 60).unwrap();
    assert_eq!((m.height, m.fps, m.profile, m.level), (720, 60, 0x01, 0x02));
    assert!(m.to_param().starts_with("00 00 01 02 00000040 "));
}

#[test]
fn audio_prefers_aac_then_lpcm() {
    assert_eq!(audio::choose("LPCM 00000003 00, AAC 00000001 00", false), Some(wfd::AudioCodec::Aac));
    assert_eq!(audio::choose("LPCM 00000002 00", false), Some(wfd::AudioCodec::Lpcm));
    assert_eq!(audio::choose("AC3 00000001 00", false), Some(wfd::AudioCodec::Lpcm), "mandatory LPCM");
    assert_eq!(audio::choose(" NONE ", false), None);
    // Juegos asks for LPCM: only when the sink lists it at 48 kHz stereo (mode bit 1).
    assert_eq!(audio::choose("LPCM 00000003 00, AAC 00000001 00", true), Some(wfd::AudioCodec::Lpcm));
    assert_eq!(audio::choose("LPCM 00000001 00, AAC 00000001 00", true), Some(wfd::AudioCodec::Aac));
}

#[test]
fn information_elements() {
    assert_eq!(
        ie::source_ies("Laptop"),
        [0, 0, 6, 0, 0x10, 0x1c, 0x44, 0, 0xc8, 0x0a, 0, 6, b'L', b'a', b'p', b't', b'o', b'p']
    );
    let tv = [0x00, 0x00, 0x06, 0x01, 0x11, 0x1c, 0x44, 0x00, 0x32];
    assert_eq!(ie::device_type(&tv), Some(ie::DeviceType::PrimarySink));
    assert!(ie::device_type(&tv).unwrap().is_sink());
    assert_eq!(ie::device_type(&[0, 0, 9, 1]), None, "truncated element");
}

#[test]
fn parameters_and_ports() {
    let body = "wfd_client_rtp_ports: RTP/AVP/UDP;unicast 19000 0 mode=play\r\nwfd_idr_request\r\n";
    assert_eq!(params::rtp_port(params::get(body, "WFD_CLIENT_RTP_PORTS").unwrap()), Some(19000));
    assert_eq!(params::rtp_port("RTP/AVP/UDP;unicast;client_port=1028-1029"), Some(1028));
    assert_eq!(params::names(body).collect::<Vec<_>>(), ["wfd_idr_request"]);
}

#[tokio::test]
async fn rtsp_round_trip_and_garbage() {
    let msg = Message::request("SET_PARAMETER", "rtsp://localhost/wfd1.0", 7).with_body("a: b\r\n".into());
    let bytes = msg.to_bytes();
    let back = Message::read(&mut &bytes[..]).await.unwrap().unwrap();
    assert_eq!((back.cseq(), back.body.as_str()), (7, "a: b\r\n"));
    assert!(Message::read(&mut &b"HELLO\r\n\r\n"[..]).await.is_err());
    assert!(Message::read(&mut &b""[..]).await.unwrap().is_none());
}

#[tokio::test]
async fn tolerates_samsung_duplicate_content_length() {
    let raw = b"RTSP/1.0 200 OK\r\nCSeq: 3\r\nContent-Length: 6\r\nContent-Length: 6\r\n\r\na: b\r\nRTSP/1.0 200 OK\r\nCSeq: 4\r\n\r\n";
    let mut r = &raw[..];
    let first = Message::read(&mut r).await.unwrap().unwrap();
    assert_eq!(first.body, "a: b\r\n");
    assert_eq!(Message::read(&mut r).await.unwrap().unwrap().cseq(), 4, "framing kept");
}

#[tokio::test]
async fn bounds_endless_lines_and_huge_bodies() {
    let endless = vec![b'A'; 100_000];
    assert!(Message::read(&mut &endless[..]).await.is_err(), "line without end");
    let huge = b"RTSP/1.0 200 OK\r\nCSeq: 1\r\nContent-Length: 999999999\r\n\r\n";
    assert!(Message::read(&mut &huge[..]).await.is_err(), "body over the bound");
}
