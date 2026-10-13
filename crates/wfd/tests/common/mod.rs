//! Sink side of the RTSP exchange for the tests.
#![allow(dead_code)]

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

pub struct Sink {
    r: BufReader<ReadHalf<DuplexStream>>,
    w: WriteHalf<DuplexStream>,
    pub cseq: u32,
}

impl Sink {
    pub fn new(io: DuplexStream) -> Self {
        let (r, w) = tokio::io::split(io);
        Self { r: BufReader::new(r), w, cseq: 0 }
    }

    pub async fn read(&mut self) -> Message {
        Message::read(&mut self.r).await.unwrap().expect("source closed")
    }

    pub async fn send(&mut self, m: Message) {
        self.w.write_all(&m.to_bytes()).await.unwrap();
    }

    pub async fn answer(&mut self, req: &Message) {
        self.send(Message::reply(req, 200, "OK")).await;
    }

    /// Sends a request and returns the source's reply.
    pub async fn request(&mut self, method: &str, body: &str) -> Message {
        self.cseq += 1;
        let mut m = Message::request(method, "rtsp://192.168.49.1/wfd1.0/streamid=0", self.cseq);
        if !body.is_empty() {
            m = m.with_body(body.into());
        }
        self.send(m).await;
        let reply = self.read().await;
        assert!(reply.is_ok() && reply.cseq() == self.cseq, "{reply:?}");
        reply
    }

    /// Plays M1-M7 as a sink and returns the M4 body the source chose.
    pub async fn handshake(&mut self, caps: String) -> String {
        let m1 = self.read().await;
        assert_eq!(m1.get("Require"), Some("org.wfa.wfd1.0"));
        self.send(
            Message::reply(&m1, 200, "OK").header("Public", "org.wfa.wfd1.0, GET_PARAMETER, SET_PARAMETER"),
        )
        .await;
        let m2 = self.request("OPTIONS", "").await;
        assert!(m2.get("Public").unwrap().contains("SETUP"));
        let m3 = self.read().await;
        assert!(m3.body.contains("wfd_video_formats"));
        self.send(Message::reply(&m3, 200, "OK").with_body(caps)).await;
        let m4 = self.read().await;
        self.answer(&m4).await;
        let m5 = self.read().await;
        assert!(m5.body.contains("wfd_trigger_method: SETUP"));
        self.answer(&m5).await;
        self.cseq += 1;
        let setup = Message::request("SETUP", "rtsp://192.168.49.1/wfd1.0/streamid=0", self.cseq)
            .header("Transport", "RTP/AVP/UDP;unicast;client_port=1028");
        self.send(setup).await;
        let m6 = self.read().await;
        assert_eq!(m6.get("Session"), Some("6B8B4567"));
        assert!(m6.get("Transport").unwrap().contains("client_port=1028"));
        self.request("PLAY", "").await;
        m4.body
    }
}
