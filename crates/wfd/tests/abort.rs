//! A receiver that gives up halfway through the negotiation.

mod common;
use common::{wanted, Sink};
use wfd::Session;

#[tokio::test]
async fn teardown_during_negotiation_is_a_closed_link() {
    let (source_end, sink_end) = tokio::io::duplex(8192);
    tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        let m1 = sink.read().await;
        sink.answer(&m1).await;
        sink.request("OPTIONS", "").await;
        let _m3 = sink.read().await; // The user cancelled on the TV instead of answering.
        let reply = sink.request("TEARDOWN", "").await;
        assert!(reply.is_ok());
    });
    let err = Session::negotiate(source_end, &wanted()).await.err().expect("must fail");
    assert!(matches!(err, wfd::Error::Closed("M3")) && err.is_transient(), "{err}");
}
