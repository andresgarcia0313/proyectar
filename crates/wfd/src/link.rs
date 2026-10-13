//! The RTSP connection with the sink: numbered requests, replies, and the requests the sink
//! sends on its own, which may arrive interleaved with the answers we wait for.

use crate::rtsp::Message;
use crate::Error;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

/// What the source supports, sent in every OPTIONS reply (M2).
pub const PUBLIC: &str = "org.wfa.wfd1.0, SETUP, TEARDOWN, PLAY, PAUSE, GET_PARAMETER, SET_PARAMETER";
/// Sinks answer within a second; slow TVs while switching input take a few.
pub const REPLY_WAIT: Duration = Duration::from_secs(10);

pub struct Link<S> {
    reader: BufReader<ReadHalf<S>>,
    writer: WriteHalf<S>,
    cseq: u32,
}

impl<S: AsyncRead + AsyncWrite> Link<S> {
    pub fn new(stream: S) -> Self {
        let (r, w) = tokio::io::split(stream);
        Self { reader: BufReader::new(r), writer: w, cseq: 0 }
    }

    pub async fn send(&mut self, msg: &Message) -> Result<(), Error> {
        self.writer.write_all(&msg.to_bytes()).await?;
        Ok(self.writer.flush().await?)
    }

    /// Next request number; the source keeps its own sequence, apart from the sink's.
    pub fn next_cseq(&mut self) -> u32 {
        self.cseq += 1;
        self.cseq
    }

    /// Next message from the sink, waiting at most `wait`.
    pub async fn recv(&mut self, step: &'static str, wait: Duration) -> Result<Message, Error> {
        match tokio::time::timeout(wait, Message::read(&mut self.reader)).await {
            Err(_) => Err(Error::Timeout(step)),
            Ok(Ok(None)) => Err(Error::Closed(step)),
            Ok(other) => other.map(|m| m.expect("Some checked above")),
        }
    }

    /// Sends a request and returns the matching reply, answering whatever the sink asks
    /// meanwhile with `answer`. A non-200 reply is an error for `step`.
    pub async fn call<F>(&mut self, step: &'static str, req: Message, mut answer: F) -> Result<Message, Error>
    where
        F: FnMut(&Message) -> Message,
    {
        let cseq = req.cseq();
        self.send(&req).await?;
        loop {
            let msg = self.recv(step, REPLY_WAIT).await?;
            if msg.method().is_some() {
                let reply = answer(&msg);
                self.send(&reply).await?;
            } else if msg.cseq() == cseq {
                return match msg.start {
                    crate::rtsp::Start::Response { code: 200, .. } => Ok(msg),
                    crate::rtsp::Start::Response { code, .. } => Err(Error::Refused { step, code }),
                    crate::rtsp::Start::Request { .. } => unreachable!("requests handled above"),
                };
            }
        }
    }

    /// Waits for the sink's request `method`, answering other requests with `answer`.
    pub async fn expect<F>(
        &mut self,
        step: &'static str,
        method: &str,
        mut answer: F,
    ) -> Result<Message, Error>
    where
        F: FnMut(&Message) -> Message,
    {
        loop {
            let msg = self.recv(step, REPLY_WAIT).await?;
            match msg.method() {
                Some(m) if m.eq_ignore_ascii_case(method) => return Ok(msg),
                Some(_) => {
                    let reply = answer(&msg);
                    self.send(&reply).await?;
                }
                None => {} // A late reply to an earlier request: nothing to do.
            }
        }
    }
}

/// Default answer to a sink request outside the step being negotiated.
pub fn default_answer(req: &Message) -> Message {
    let reply = Message::reply(req, 200, "OK");
    match req.method() {
        Some("OPTIONS") => reply.header("Public", PUBLIC),
        _ => reply,
    }
}
