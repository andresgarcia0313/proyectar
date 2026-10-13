//! A negotiated session whose sink then follows a script, for the streaming tests.

use super::{caps_reply, wanted, Sink, SAMSUNG_VIDEO};
use std::future::Future;
use std::time::Duration;
use tokio::io::DuplexStream;
use wfd::Session;

/// M3 line of a sink that can resume from standby.
pub const STANDBY: &str = "wfd_standby_resume_capability: supported\r\n";

/// Negotiates against a sink that then plays `script`; the sink stays alive afterwards.
pub async fn start<F, Fut>(extra: &'static str, script: F) -> Session<DuplexStream>
where
    F: FnOnce(Sink) -> Fut + Send + 'static,
    Fut: Future<Output = Sink> + Send,
{
    let (source_end, sink_end) = tokio::io::duplex(8192);
    tokio::spawn(async move {
        let mut sink = Sink::new(sink_end);
        sink.handshake(caps_reply(SAMSUNG_VIDEO, "AAC 00000001 00") + extra).await;
        let _alive = (script(sink).await, tokio::time::sleep(Duration::from_secs(600)).await);
    });
    Session::negotiate(source_end, &wanted()).await.expect("negotiation").0
}
