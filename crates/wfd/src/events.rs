//! The session once streaming: keep-alives both ways (M16) and what the sink asks for.

use crate::link::REPLY_WAIT;
use crate::rtsp::Message;
use crate::session::{Event, Session, SESSION_TIMEOUT};
use crate::{params, Error};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{sleep_until, Instant};

/// Keep-alive period: half the session timeout, so one lost message is harmless.
const KEEPALIVE: Duration = Duration::from_secs(SESSION_TIMEOUT / 2);
/// A keep-alive without answer by then is a miss, and the next one goes out at once.
const PING_WAIT: Duration = Duration::from_secs(5);
/// Misses in a row that make the sink mute: 15 s + 3 x 5 s, the session timeout.
const MAX_MISSES: u8 = 3;

impl<S: AsyncRead + AsyncWrite> Session<S> {
    /// Waits for the next thing the sink wants, answering its own keep-alives and sending
    /// ours on time. An error means the link is gone: the sink stopped answering
    /// (`Timeout("M16")`, transient) or closed the connection. Requests from the sink do
    /// not count as answers, so a sink that talks but never replies is still caught.
    pub async fn next_event(&mut self) -> Result<Event, Error> {
        loop {
            while let Some(req) = self.link.answered.pop_front() {
                if let Some(event) = classify(&req) {
                    return Ok(event);
                }
            }
            let ping_at = *self.ping_at.get_or_insert_with(|| Instant::now() + KEEPALIVE);
            let deadline = self.pending.map_or(ping_at, |(_, until)| until);
            let ready = tokio::select! {
                ready = self.link.ready() => ready?,
                () = sleep_until(deadline) => {
                    self.keepalive().await?;
                    continue;
                }
            };
            if !ready {
                return Err(Error::Closed("la sesión"));
            }
            let msg = self.link.recv("la sesión", REPLY_WAIT).await?;
            if msg.method().is_some() {
                self.link.handle("la sesión", msg).await?;
            } else if self.pending.is_some_and(|(cseq, _)| cseq >= msg.cseq()) {
                (self.pending, self.misses) = (None, 0);
            }
        }
    }

    /// Sends the keep-alive that is due; the third unanswered in a row ends the session.
    async fn keepalive(&mut self) -> Result<(), Error> {
        if self.pending.is_some() {
            self.misses += 1;
            if self.misses >= MAX_MISSES {
                return Err(Error::Timeout("M16"));
            }
        }
        let cseq = self.link.next_cseq();
        let mut ping = Message::request("GET_PARAMETER", "rtsp://localhost/wfd1.0", cseq);
        if let Some(id) = self.link.session {
            ping = ping.header("Session", id);
        }
        self.link.send(&ping).await?;
        self.pending = Some((cseq, Instant::now() + PING_WAIT));
        self.ping_at = Some(Instant::now() + KEEPALIVE);
        Ok(())
    }
}

/// What a sink request means for the stream; `None` for keep-alives and options.
fn classify(req: &Message) -> Option<Event> {
    match req.method()? {
        "PLAY" => Some(Event::Play),
        "PAUSE" => Some(Event::Pause),
        "TEARDOWN" => Some(Event::Teardown),
        "SET_PARAMETER" if params::mentions(&req.body, "wfd_idr_request") => Some(Event::IdrRequest),
        "SET_PARAMETER" if params::mentions(&req.body, "wfd_standby") => Some(Event::Pause),
        _ => None,
    }
}
