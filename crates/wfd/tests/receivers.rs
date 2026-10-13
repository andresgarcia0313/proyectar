//! M1-M7 against scripted receivers, each with the capabilities and quirks of its family.

mod common;
use common::{wanted, Quirks, Sink, SAMSUNG_VIDEO};
use wfd::{AudioCodec, Negotiated, SinkInfo, VideoMode, Wanted};

struct Outcome {
    got: Negotiated,
    info: SinkInfo,
    m4: String,
    asked: String,
}

async fn dialog(quirks: Quirks, caps: &str, wanted: Wanted) -> Outcome {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    let caps = caps.to_owned();
    let sink = tokio::spawn(async move {
        let mut sink = Sink::with(sink_end, quirks);
        let m4 = sink.handshake(caps).await;
        (m4, sink.asked)
    });
    let (session, got) = wfd::Session::negotiate(source_end, &wanted).await.expect("negotiation");
    let (m4, asked) = sink.await.unwrap();
    Outcome { got, info: session.sink_info().clone(), m4, asked }
}

fn mode(o: &Outcome) -> (u32, u32, u32, u8) {
    (o.got.video.width, o.got.video.height, o.got.video.fps, o.got.video.profile)
}

#[tokio::test]
async fn samsung_tizen() {
    let caps = format!("wfd_video_formats: {SAMSUNG_VIDEO}\r\nwfd_audio_codecs: LPCM 00000002 00, AAC 00000001 00\r\nwfd_client_rtp_ports: RTP/AVP/UDP;unicast 1028 0 mode=play\r\nwfd_display_edid: none\r\nwfd_connector_type: 05\r\n");
    let o = dialog(Quirks { dup_len: true, ..Default::default() }, &caps, wanted()).await;
    assert_eq!(mode(&o), (1920, 1080, 60, 0x01));
    assert_eq!((o.got.audio, o.got.rtp_port), (Some(AudioCodec::Aac), 1028));
    assert_eq!((o.info.edid.as_deref(), o.info.connector_type.as_deref()), (None, Some("05")));
    assert!(o.asked.contains("wfd_display_edid") && !o.asked.contains("intel_friendly_name"), "{}", o.asked);
}

#[tokio::test]
async fn lg_webos_chp_only_with_setup_before_m5_reply() {
    let caps = "wfd_video_formats:  00 00 02 10 0001ffff 1fffffff 00000fff 00 0000 0000 00 none none\r\nWFD_Audio_Codecs: LPCM 00000002 00, AAC 00000001 00\r\nwfd_client_rtp_ports: RTP/AVP/UDP; unicast 1990 0 mode=play\r\nwfd_standby_resume_capability: supported\r\n";
    let o = dialog(Quirks { spaces: true, setup_early: true, ..Default::default() }, caps, wanted()).await;
    assert_eq!(mode(&o), (1920, 1080, 60, 0x02));
    assert!(o.m4.contains("wfd_video_formats: 00 00 02 10 00000100 00000000 00000000"), "{}", o.m4);
    assert!(o.info.standby_resume);
}

#[tokio::test]
async fn sony_android_tv_lower_case_and_lf() {
    let caps = "wfd_video_formats: 38 00 01 10 00007fff 00000000 00000000 00 0000 0000 00 none none\nwfd_audio_codecs: AAC 00000001 00\nwfd_client_rtp_ports: RTP/AVP/UDP;unicast 15550 0 mode=play\n";
    let o = dialog(Quirks { lower: true, lf: true, ..Default::default() }, caps, wanted()).await;
    assert_eq!(mode(&o), (1920, 1080, 60, 0x01));
    assert_eq!(o.got.rtp_port, 1028, "SETUP Transport wins over M3");
}

#[tokio::test]
async fn roku_level_4_and_m2_before_m1_reply() {
    let caps = "wfd_video_formats: 00 00 01 04 000001FF 00000000 00000000 00 0000 0000 00 none none\r\nwfd_audio_codecs: AAC 00000001 00\r\nwfd_client_rtp_ports: RTP/AVP/UDP;unicast 1028 0 mode=play\r\n";
    let o = dialog(Quirks { m2_early: true, ..Default::default() }, caps, wanted()).await;
    assert_eq!(mode(&o), (1280, 720, 60, 0x01), "1080p60 needs level 4.2");
    assert_eq!(o.got.video.h264_level(), "4");
}

#[tokio::test]
async fn fire_tv_without_native_field_and_lpcm_only() {
    let caps = "wfd_video_formats: 02 10 0001ffff 00000000 00000000 00 0000 0000 00 none none\nwfd_audio_codecs: LPCM 00000003 00\nwfd_client_rtp_ports: rtp/avp/udp;unicast 19000 0 mode=play\nwfd2_video_formats: 01 00 0001\n";
    let o =
        dialog(Quirks { lower: true, lf: true, dup_len: true, ..Default::default() }, caps, wanted()).await;
    assert_eq!(mode(&o), (1920, 1080, 60, 0x02));
    assert_eq!(o.got.audio, Some(AudioCodec::Lpcm));
    assert_eq!(o.info.other, [("wfd2_video_formats".to_owned(), "01 00 0001".to_owned())]);
}

#[tokio::test]
async fn microsoft_adapter_vesa_and_vendor_names() {
    let caps = "wfd_video_formats: 00 00 03 10 000000ff 00002000 00000000 00 0000 0000 00 none none\r\nwfd_audio_codecs: AAC 00000001 00\r\nwfd_client_rtp_ports: RTP/AVP/UDP;unicast 1028 0 mode=play\r\nintel_friendly_name: Sala\r\nmicrosoft_latency_management_capability: supported\r\n";
    let q = Quirks {
        public: "org.wfa.wfd1.0, GET_PARAMETER, SET_PARAMETER, microsoft_cursor",
        ..Default::default()
    };
    let o = dialog(q, caps, Wanted { max_height: 768, ..wanted() }).await;
    assert_eq!(mode(&o), (1366, 768, 60, 0x01));
    assert!(o.m4.contains("00 00 01 10 00000000 00002000 00000000"), "{}", o.m4);
    assert_eq!(o.info.friendly_name.as_deref(), Some("Sala"));
    assert!(o.asked.contains("intel_friendly_name"), "{}", o.asked);
}

#[tokio::test(start_paused = true)]
async fn anycast_dongle_refuses_extras_and_formats() {
    let caps = "wfd_video_formats: 00 00 01 02 00000041 00000000 00000000 00 0000 0000 00 none none\r\nwfd_audio_codecs: AAC 00000001 00\r\nwfd_client_rtp_ports: RTP/AVP/UDP;unicast 1028 0 mode=play\r\n";
    let q = Quirks { skip_m2: true, refuse_extra: true, refuse_m4: true, ..Default::default() };
    let o = dialog(q, caps, wanted()).await;
    assert_eq!(o.got.video, VideoMode::FALLBACK);
    assert_eq!(o.got.audio, Some(AudioCodec::Lpcm));
    assert_eq!(o.asked.lines().count(), 4, "mandatory names only: {}", o.asked);
    assert!(o.m4.contains("wfd_audio_codecs: LPCM 00000002 00"), "{}", o.m4);
}
