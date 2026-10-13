//! Requests the sink sends on its own: kept for their turn, or answered at once.

use crate::link::{Link, PUBLIC};
use crate::rtsp::Message;
use crate::Error;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};

impl<S: AsyncRead + AsyncWrite> Link<S> {
    /// A request the sink sent on its own: held, or answered (and queued while streaming).
    /// A TEARDOWN before the session exists means the sink gave up on the negotiation.
    pub async fn handle(&mut self, step: &'static str, req: Message) -> Result<(), Error> {
        let method = req.method().unwrap_or_default().to_owned();
        if self.hold.iter().any(|m| m.eq_ignore_ascii_case(&method)) {
            self.held.push_back(req);
            return Ok(());
        }
        self.send(&self.answer(&req)).await?;
        if self.session.is_none() && method == "TEARDOWN" {
            return Err(Error::Closed(step));
        }
        if self.session.is_some() && self.answered.len() < 32 {
            self.answered.push_back(req);
        }
        Ok(())
    }

    /// Default answer: 200, `Public` for OPTIONS, the session id where RTSP wants it.
    pub fn answer(&self, req: &Message) -> Message {
        let mut reply = Message::reply(req, 200, "OK");
        if req.method() == Some("OPTIONS") {
            reply = reply.header("Public", PUBLIC);
        }
        let in_session =
            matches!(req.method(), Some("PLAY" | "PAUSE" | "TEARDOWN")) || req.get("Session").is_some();
        match self.session {
            Some(id) if in_session => reply.header("Session", id),
            _ => reply,
        }
    }

    /// Waits up to `wait` for the sink's request `method`, taking a held one first.
    pub async fn expect(
        &mut self,
        step: &'static str,
        method: &str,
        wait: Duration,
    ) -> Result<Message, Error> {
        if let Some(i) = self.held.iter().position(|m| m.method() == Some(method)) {
            return Ok(self.held.remove(i).expect("position is in range"));
        }
        let until = tokio::time::Instant::now() + wait;
        loop {
            let msg = self.recv(step, until.saturating_duration_since(tokio::time::Instant::now())).await?;
            match msg.method() {
                Some(m) if m == method => return Ok(msg),
                Some(_) => self.handle(step, msg).await?,
                None => {}
            }
        }
    }
}
