//! Reading one RTSP message from the sink, with a hard size bound: the peer is a device
//! on the air, so a line without end or a huge `Content-Length` must not grow memory.

use crate::rtsp::Message;
use crate::start::parse_start;
use crate::Error;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

/// Whole message, start line to body; real WFD messages stay under 2 KiB.
pub const MAX_SIZE: u64 = 64 * 1024;

impl Message {
    /// Reads one message. `Ok(None)` means the peer closed the connection between messages.
    pub async fn read<R: AsyncBufRead + Unpin>(r: &mut R) -> Result<Option<Self>, Error> {
        let mut r = r.take(MAX_SIZE);
        let mut line = String::new();
        while line.trim().is_empty() {
            line.clear();
            if r.read_line(&mut line).await? == 0 {
                return if r.limit() == 0 { Err(too_big()) } else { Ok(None) };
            }
        }
        let start = parse_start(line.trim())?;
        let mut headers = Vec::new();
        loop {
            line.clear();
            r.read_line(&mut line).await?;
            if !line.ends_with('\n') {
                return Err(if r.limit() == 0 { too_big() } else { cut_short() });
            }
            let text = line.trim_end();
            if text.is_empty() {
                break;
            }
            let (k, v) = text.split_once(':').ok_or(Error::Malformed(text.into()))?;
            headers.push((k.trim().to_owned(), v.trim().to_owned()));
        }
        let mut msg = Self { start, headers, body: String::new() };
        let len: u64 = msg.get("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
        if len > r.limit() {
            return Err(too_big());
        }
        let mut body = vec![0; len as usize];
        r.read_exact(&mut body).await?;
        msg.body = String::from_utf8_lossy(&body).into_owned();
        Ok(Some(msg))
    }
}

fn too_big() -> Error {
    Error::Malformed(format!("mensaje de más de {MAX_SIZE} bytes"))
}

fn cut_short() -> Error {
    Error::Io(std::io::ErrorKind::UnexpectedEof.into())
}
