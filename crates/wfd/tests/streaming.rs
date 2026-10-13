//! The session after M7: keep-alives both ways, M5 triggers, standby and teardown.

mod common;
use common::script::{start, STANDBY};
use std::time::Duration;
use wfd::Event;

#[tokio::test(start_paused = true)]
async fn answers_the_sink_keepalive_and_sends_its_own_with_session() {
    let mut session = start("", |mut sink| async move {
        let reply = sink.request("GET_PARAMETER", "").await;
        assert_eq!(reply.get("CSeq"), Some(sink.cseq.to_string().as_str()));
        let ping = sink.read().await;
        assert_eq!((ping.method(), ping.get("Session")), (Some("GET_PARAMETER"), Some("6B8B4567")));
        sink.answer(&ping).await;
        sink.request("TEARDOWN", "").await;
        sink
    })
    .await;
    assert_eq!(session.next_event().await.unwrap(), Event::Teardown);
}

#[tokio::test(start_paused = true)]
async fn two_missed_keepalives_are_forgiven_the_third_is_not() {
    let mut session = start("", |mut sink| async move {
        for _ in 0..2 {
            assert_eq!(sink.read().await.method(), Some("GET_PARAMETER")); // Lost on the air.
        }
        let third = sink.read().await;
        sink.answer(&third).await; // Back in time: the session goes on.
        for _ in 0..3 {
            assert_eq!(sink.read().await.method(), Some("GET_PARAMETER"));
        }
        sink
    })
    .await;
    let begun = tokio::time::Instant::now();
    let err = session.next_event().await.unwrap_err();
    assert!(matches!(err, wfd::Error::Timeout("M16")) && err.is_transient(), "{err}");
    assert!(begun.elapsed() >= Duration::from_secs(55), "gave up after {:?}", begun.elapsed());
}

#[tokio::test(start_paused = true)]
async fn sink_standby_pauses_and_play_resumes() {
    let mut session = start("", |mut sink| async move {
        sink.request("SET_PARAMETER", "wfd_standby\r\n").await;
        sink.request("PLAY", "").await;
        sink
    })
    .await;
    assert_eq!(session.next_event().await.unwrap(), Event::Pause);
    assert_eq!(session.next_event().await.unwrap(), Event::Play);
}

#[tokio::test(start_paused = true)]
async fn pause_trigger_and_source_standby() {
    let mut session = start(STANDBY, |mut sink| async move {
        let m5 = sink.read().await;
        assert!(m5.body.contains("wfd_trigger_method: PAUSE") && m5.get("Session").is_some(), "{m5:?}");
        sink.answer(&m5).await;
        let reply = sink.request("PAUSE", "").await;
        assert_eq!(reply.get("Session"), Some("6B8B4567"));
        let m12 = sink.read().await;
        assert_eq!(m12.body.trim(), "wfd_standby");
        sink.answer(&m12).await;
        sink
    })
    .await;
    session.pause().await.unwrap();
    assert_eq!(session.next_event().await.unwrap(), Event::Pause);
    assert!(session.standby().await.unwrap());
}

#[tokio::test(start_paused = true)]
async fn no_standby_for_a_sink_that_cannot_resume_and_a_polite_close() {
    let mut session = start("", |mut sink| async move {
        let m5 = sink.read().await;
        assert!(m5.body.contains("wfd_trigger_method: TEARDOWN"));
        let teardown = sink.next_request("TEARDOWN", ""); // Sent before answering M5.
        sink.send(teardown).await;
        sink.answer(&m5).await;
        let reply = sink.read().await;
        assert!(reply.is_ok() && reply.cseq() == sink.cseq, "{reply:?}");
        sink
    })
    .await;
    assert!(!session.standby().await.unwrap());
    tokio::time::timeout(Duration::from_secs(3), session.close()).await.unwrap().unwrap();
}
