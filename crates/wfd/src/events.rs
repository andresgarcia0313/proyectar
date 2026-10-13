//! The session once streaming: keep-alives (M16) and what the sink asks for meanwhile.

use crate::link::{default_answer, REPLY_WAIT};
use crate::rtsp::Message;
use crate::session::{Event, Session, SESSION_TIMEOUT};
use crate::{params, Error};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{sleep_until, Instant};

/// Keep-alive period: half the session timeout, so one lost message is harmless.
const KEEPALIVE: Duration = Duration::from_secs(SESSION_TIMEOUT / 2);

impl<S: AsyncRead + AsyncWrite> Session<S> {
    /// Waits for the next thing the sink wants, sending keep-alives on time. An error means
    /// the link is gone: the sink stopped answering or closed the connection.
    /// The keep-alive schedule lives in the session, so events in between never reset it.
    pub async fn next_event(&mut self) -> Result<Event, Error> {
        loop {
            let ping_at = *self.ping_at.get_or_insert_with(|| Instant::now() + KEEPALIVE);
            let deadline = self.pending.map_or(ping_at, |(_, until)| until.min(ping_at));
            let msg = tokio::select! {
                msg = self.link.recv("la sesión", Duration::from_secs(SESSION_TIMEOUT * 2)) => msg?,
                () = sleep_until(deadline) => {
                    if self.pending.is_some_and(|(_, until)| Instant::now() >= until) {
                        return Err(Error::Timeout("M16"));
                    }
                    let cseq = self.link.next_cseq();
                    self.link.send(&Message::request("GET_PARAMETER", &self.url, cseq)).await?;
                    self.pending = Some((cseq, Instant::now() + REPLY_WAIT));
                    self.ping_at = Some(Instant::now() + KEEPALIVE);
                    continue;
                }
            };
            if msg.method().is_none() {
                if self.pending.is_some_and(|(cseq, _)| cseq == msg.cseq()) {
                    self.pending = None;
                }
                continue;
            }
            let event = classify(&msg);
            self.link.send(&default_answer(&msg)).await?;
            if let Some(event) = event {
                return Ok(event);
            }
        }
    }

    /// Ends the session politely: asks the sink to send TEARDOWN and answers it.
    /// A sink that already left (closed, silent) counts as closed.
    pub async fn close(mut self) -> Result<(), Error> {
        let done = match self.trigger("cierre", "TEARDOWN").await {
            Ok(()) => self.link.expect("cierre", "TEARDOWN", default_answer).await.map(drop),
            other => other,
        };
        match done {
            Err(Error::Closed(_) | Error::Timeout(_)) => Ok(()),
            other => other,
        }
    }
}

/// What a sink request means for the stream; `None` for keep-alives and options.
fn classify(req: &Message) -> Option<Event> {
    match req.method()? {
        "PLAY" => Some(Event::Play),
        "PAUSE" => Some(Event::Pause),
        "TEARDOWN" => Some(Event::Teardown),
        "SET_PARAMETER" if params::names(&req.body).any(|n| n == "wfd_idr_request") => {
            Some(Event::IdrRequest)
        }
        _ => None,
    }
}
