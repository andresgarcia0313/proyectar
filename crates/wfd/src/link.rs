//! The RTSP connection with the sink: numbered requests, replies, and the requests the sink
//! sends on its own, which may arrive interleaved with the answers we wait for.

use crate::rtsp::{Message, Start};
use crate::Error;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

/// What the source supports, sent in every OPTIONS reply (M2).
pub const PUBLIC: &str = "org.wfa.wfd1.0, SETUP, TEARDOWN, PLAY, PAUSE, GET_PARAMETER, SET_PARAMETER";
/// Sinks answer within a second; slow TVs while switching input take a few.
pub const REPLY_WAIT: Duration = Duration::from_secs(10);

pub struct Link<S> {
    reader: BufReader<ReadHalf<S>>,
    writer: WriteHalf<S>,
    cseq: u32,
    /// Sink requests kept unanswered for a later `expect`: a sink may send M2 before it
    /// answers M1, or SETUP before it answers the M5 that asked for it.
    pub hold: &'static [&'static str],
    pub(crate) held: VecDeque<Message>,
    /// Requests answered while streaming, still to be reported as events.
    pub answered: VecDeque<Message>,
    /// Session id once M6 created it; also tells negotiation from streaming.
    pub session: Option<&'static str>,
}

impl<S: AsyncRead + AsyncWrite> Link<S> {
    pub fn new(stream: S) -> Self {
        let (r, w) = tokio::io::split(stream);
        let (held, answered) = (VecDeque::new(), VecDeque::new());
        Self { reader: BufReader::new(r), writer: w, cseq: 0, hold: &[], held, answered, session: None }
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

    /// Resolves when bytes are waiting. Safe to drop inside `select!`, unlike a half-read
    /// message; `false` means the sink closed the connection.
    pub async fn ready(&mut self) -> Result<bool, Error> {
        Ok(!self.reader.fill_buf().await?.is_empty())
    }

    /// Next message from the sink, waiting at most `wait`.
    pub async fn recv(&mut self, step: &'static str, wait: Duration) -> Result<Message, Error> {
        match tokio::time::timeout(wait, Message::read(&mut self.reader)).await {
            Err(_) => Err(Error::Timeout(step)),
            Ok(Ok(None)) => Err(Error::Closed(step)),
            Ok(other) => other.map(|m| m.expect("Some checked above")),
        }
    }

    /// Sends a request and returns its reply; any 2xx is success, anything else an error.
    pub async fn call(&mut self, step: &'static str, req: Message) -> Result<Message, Error> {
        let cseq = req.cseq();
        self.send(&req).await?;
        loop {
            let msg = self.recv(step, REPLY_WAIT).await?;
            match msg.start {
                Start::Request { .. } => self.handle(step, msg).await?,
                Start::Response { code: 200..=299, .. } if msg.cseq() == cseq => return Ok(msg),
                Start::Response { code, .. } if msg.cseq() == cseq => {
                    return Err(Error::Refused { step, code })
                }
                Start::Response { .. } => {} // A late reply to an earlier request.
            }
        }
    }
}
