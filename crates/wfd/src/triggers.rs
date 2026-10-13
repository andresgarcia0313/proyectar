//! What the source asks of the sink: M5 triggers and its own standby (M12).

use crate::link::REPLY_WAIT;
use crate::rtsp::Message;
use crate::session::Session;
use crate::{params, Error};
use tokio::io::{AsyncRead, AsyncWrite};

const URI: &str = "rtsp://localhost/wfd1.0";

impl<S: AsyncRead + AsyncWrite> Session<S> {
    /// M5: asks the sink to send SETUP, PLAY, PAUSE or TEARDOWN. Its PLAY or PAUSE then
    /// comes out of `next_event`.
    pub async fn trigger(&mut self, step: &'static str, method: &str) -> Result<(), Error> {
        let body = params::body(&[("wfd_trigger_method", method.into())]);
        self.set_parameter(step, body).await
    }

    pub async fn play(&mut self) -> Result<(), Error> {
        self.trigger("M5 PLAY", "PLAY").await
    }

    pub async fn pause(&mut self) -> Result<(), Error> {
        self.trigger("M5 PAUSE", "PAUSE").await
    }

    /// M12: tells the sink the source goes to standby. Only a sink that declared
    /// `wfd_standby_resume_capability: supported` gets it; `Ok(false)` for the rest.
    pub async fn standby(&mut self) -> Result<bool, Error> {
        if !self.info.standby_resume {
            return Ok(false);
        }
        self.set_parameter("M12", "wfd_standby\r\n".into()).await.map(|()| true)
    }

    /// Ends the session politely: asks the sink to send TEARDOWN and answers it.
    /// A sink that already left (closed, silent) counts as closed.
    pub async fn close(mut self) -> Result<(), Error> {
        self.link.hold = &["TEARDOWN"];
        let done = match self.trigger("cierre", "TEARDOWN").await {
            Ok(()) => match self.link.expect("cierre", "TEARDOWN", REPLY_WAIT).await {
                Ok(req) => self.link.send(&self.link.answer(&req)).await,
                Err(e) => Err(e),
            },
            other => other,
        };
        match done {
            Err(Error::Closed(_) | Error::Timeout(_) | Error::Io(_)) => Ok(()),
            other => other,
        }
    }

    async fn set_parameter(&mut self, step: &'static str, body: String) -> Result<(), Error> {
        let mut req = Message::request("SET_PARAMETER", URI, self.link.next_cseq()).with_body(body);
        if let Some(id) = self.link.session {
            req = req.header("Session", id);
        }
        self.link.call(step, req).await.map(drop)
    }
}
