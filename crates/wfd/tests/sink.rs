//! A scripted sink, shaped after a 2020 Samsung TV, drives the source through M1-M7.

mod common;
use common::{caps_reply, Sink, SAMSUNG_VIDEO};
use std::time::Duration;
use wfd::{AudioCodec, Event, Session, Wanted};

fn wanted() -> Wanted {
    Wanted { max_height: 1080, max_fps: 60, audio: true, lpcm: false, local_ip: "192.168.49.1".into() }
}

#[tokio::test]
async fn negotiates_like_a_samsung_tv() {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    let sink = tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        let m4 = sink.handshake(caps_reply(SAMSUNG_VIDEO, "LPCM 00000002 00, AAC 00000007 00")).await;
        assert!(m4.contains("wfd_video_formats: 00 00 01 10 00000100"), "{m4}");
        assert!(m4.contains("wfd_audio_codecs: AAC 00000001 00"), "{m4}");
        assert!(m4.contains("rtsp://192.168.49.1/wfd1.0/streamid=0 none"), "{m4}");
        sink
    });
    let (session, got) = Session::negotiate(source_end, &wanted()).await.expect("negotiation");
    assert_eq!((got.video.width, got.video.height, got.video.fps), (1920, 1080, 60));
    assert_eq!(got.audio, Some(AudioCodec::Aac));
    assert_eq!(got.rtp_port, 1028);
    drop((session, sink.await.unwrap()));
}

#[tokio::test]
async fn reports_idr_requests_and_teardown() {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    let sink = tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        sink.handshake(caps_reply(SAMSUNG_VIDEO, "AAC 00000001 00")).await;
        sink.request("SET_PARAMETER", "wfd_idr_request\r\n").await;
        sink.request("TEARDOWN", "").await;
        sink
    });
    let (mut session, _) = Session::negotiate(source_end, &wanted()).await.unwrap();
    assert_eq!(session.next_event().await.unwrap(), Event::IdrRequest);
    assert_eq!(session.next_event().await.unwrap(), Event::Teardown);
    drop(sink.await.unwrap());
}

#[tokio::test(start_paused = true)]
async fn sends_keepalives_and_detects_a_silent_sink() {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    let sink = tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        sink.handshake(caps_reply(SAMSUNG_VIDEO, "AAC 00000001 00")).await;
        let ping = sink.read().await;
        assert_eq!(ping.method(), Some("GET_PARAMETER"));
        sink.answer(&ping).await;
        let second = sink.read().await; // Not answered: the sink "froze".
        assert_eq!(second.method(), Some("GET_PARAMETER"));
        tokio::time::sleep(Duration::from_secs(120)).await;
        sink
    });
    let (mut session, _) = Session::negotiate(source_end, &wanted()).await.unwrap();
    let err = session.next_event().await.unwrap_err();
    assert!(matches!(err, wfd::Error::Timeout("M16")), "{err}");
    sink.abort();
}

#[tokio::test]
async fn refuses_a_sink_without_common_video() {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        let _ = sink.handshake(caps_reply("none", "AAC 00000001 00")).await;
    });
    let err = Session::negotiate(source_end, &wanted()).await.err().expect("must fail");
    assert!(matches!(err, wfd::Error::NoCommonFormat("video")), "{err}");
    assert!(!err.is_transient());
}

#[tokio::test(start_paused = true)]
async fn idr_requests_do_not_postpone_the_keepalive_deadline() {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    let sink = tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        sink.handshake(caps_reply(SAMSUNG_VIDEO, "AAC 00000001 00")).await;
        let ping = sink.read().await; // Never answered.
        assert_eq!(ping.method(), Some("GET_PARAMETER"));
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            sink.cseq += 1;
            let req = wfd::rtsp::Message::request("SET_PARAMETER", "rtsp://x/wfd1.0", sink.cseq)
                .with_body("wfd_idr_request\r\n".into());
            sink.send(req).await;
        }
        sink
    });
    let (mut session, _) = Session::negotiate(source_end, &wanted()).await.unwrap();
    let mut idr = 0;
    let err = loop {
        match session.next_event().await {
            Ok(Event::IdrRequest) => idr += 1,
            Ok(other) => panic!("{other:?}"),
            Err(e) => break e,
        }
    };
    assert!(matches!(err, wfd::Error::Timeout("M16")), "{err}");
    assert!(idr < 30, "timed out while the sink kept talking ({idr} IDR)");
    sink.abort();
}
