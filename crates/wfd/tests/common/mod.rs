//! Sink side of the RTSP exchange for the tests, with the quirks of real receivers.
#![allow(dead_code)]

mod quirks;
pub mod script;
pub use quirks::Quirks;

use tokio::io::{AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf};
use wfd::rtsp::Message;

/// Video capabilities a UN65TU7000 reports: CBP and CHP, level 4.2, CEA up to 1080p60.
pub const SAMSUNG_VIDEO: &str = "00 00 03 10 0001ffff 3fffffff 00000fff 00 0000 0000 11 none none";

pub fn caps_reply(video: &str, audio: &str) -> String {
    format!(
        "wfd_video_formats: {video}\r\nwfd_audio_codecs: {audio}\r\n\
         wfd_client_rtp_ports: RTP/AVP/UDP;unicast 1028 0 mode=play\r\n"
    )
}

pub fn wanted() -> wfd::Wanted {
    wfd::Wanted { max_height: 1080, max_fps: 60, audio: true, local_ip: "192.168.49.1".into() }
}

pub struct Sink {
    r: BufReader<ReadHalf<DuplexStream>>,
    w: WriteHalf<DuplexStream>,
    pub cseq: u32,
    pub quirks: Quirks,
    /// Body of the last M3, to check what the source asked for.
    pub asked: String,
}

impl Sink {
    pub fn new(io: DuplexStream) -> Self {
        Self::with(io, Quirks::default())
    }

    pub fn with(io: DuplexStream, quirks: Quirks) -> Self {
        let (r, w) = tokio::io::split(io);
        Self { r: BufReader::new(r), w, cseq: 0, quirks, asked: String::new() }
    }

    pub async fn read(&mut self) -> Message {
        Message::read(&mut self.r).await.unwrap().expect("source closed")
    }

    /// Writes `m` the way this receiver would: letter case, line ends, repeated headers.
    pub async fn send(&mut self, m: Message) {
        let text = self.quirks.render(&m);
        self.w.write_all(text.as_bytes()).await.unwrap();
    }

    pub async fn answer(&mut self, req: &Message) {
        self.send(Message::reply(req, 200, "OK")).await;
    }

    pub fn next_request(&mut self, method: &str, body: &str) -> Message {
        self.cseq += 1;
        let m = Message::request(method, "rtsp://192.168.49.1/wfd1.0/streamid=0", self.cseq);
        if body.is_empty() {
            m
        } else {
            m.with_body(body.into())
        }
    }

    /// Sends a request and returns the source's reply, skipping keep-alives meanwhile.
    pub async fn request(&mut self, method: &str, body: &str) -> Message {
        let m = self.next_request(method, body);
        self.send(m).await;
        loop {
            let reply = self.read().await;
            if reply.method() == Some("GET_PARAMETER") {
                self.answer(&reply).await;
                continue;
            }
            assert!(reply.is_ok() && reply.cseq() == self.cseq, "{reply:?}");
            return reply;
        }
    }

    /// Plays M1-M7 as a sink and returns the M4 body the source chose.
    pub async fn handshake(&mut self, caps: String) -> String {
        quirks::handshake(self, caps).await
    }
}
