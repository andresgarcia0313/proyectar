//! A stand-in for the TV: UDP -> RTP -> MPEG-TS demux, counting what arrives on each PID.

use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Arc;

#[derive(Default)]
pub struct Cuentas {
    pub video: AtomicU64,
    pub claves: AtomicU64,
    pub audio: AtomicU64,
    pub audio_bytes: AtomicU64,
    /// MPEG-TS level: PES starts seen on the audio PID (0x1100), and PMT packets (0x1000).
    pub ts_audio: AtomicU64,
    pub ts_pmt: AtomicU64,
    /// Names of the pads the demuxer created (they carry the PIDs).
    pub pads: std::sync::Mutex<Vec<String>>,
}

pub struct Receptor {
    pub puerto: u16,
    pub cuentas: Arc<Cuentas>,
    pipeline: gst::Element,
}

pub fn puerto_libre() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

impl Receptor {
    pub fn nuevo() -> Self {
        gst::init().unwrap();
        let puerto = puerto_libre();
        let texto = format!(
            "udpsrc port={puerto} address=127.0.0.1 buffer-size=4000000 caps=\"application/x-rtp,media=video,\
             clock-rate=90000,encoding-name=MP2T,payload=33\" ! rtpmp2tdepay name=dep ! tsdemux name=d \
             d. ! queue ! video/x-h264 ! h264parse ! fakesink name=v sync=false \
             d. ! queue ! audio/mpeg ! aacparse ! fakesink name=a sync=false"
        );
        let pipeline = gst::parse::launch(&texto).expect("receptor");
        let cuentas = Arc::new(Cuentas::default());
        let tuberia = pipeline.clone().downcast::<gst::Bin>().unwrap();
        let c = cuentas.clone();
        tuberia.by_name("d").unwrap().connect_pad_added(move |_, pad| {
            c.pads.lock().unwrap().push(pad.name().to_string());
        });
        let c = cuentas.clone();
        let dep = tuberia.by_name("dep").unwrap().static_pad("src").unwrap();
        dep.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if let Some(gst::PadProbeData::Buffer(b)) = &info.data {
                let mapa = b.map_readable().unwrap();
                for p in mapa.chunks_exact(188) {
                    let pid = (u16::from(p[1] & 0x1f) << 8) | u16::from(p[2]);
                    match pid {
                        0x1100 if p[1] & 0x40 != 0 => c.ts_audio.fetch_add(1, Relaxed),
                        0x1000 => c.ts_pmt.fetch_add(1, Relaxed),
                        _ => 0,
                    };
                }
            }
            gst::PadProbeReturn::Ok
        });
        for (nombre, es_video) in [("v", true), ("a", false)] {
            let c = cuentas.clone();
            let pad = tuberia.by_name(nombre).unwrap().static_pad("sink").unwrap();
            pad.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
                if let Some(gst::PadProbeData::Buffer(b)) = &info.data {
                    if es_video {
                        c.video.fetch_add(1, Relaxed);
                        if !b.flags().contains(gst::BufferFlags::DELTA_UNIT) {
                            c.claves.fetch_add(1, Relaxed);
                        }
                    } else {
                        c.audio.fetch_add(1, Relaxed);
                        c.audio_bytes.fetch_add(b.size() as u64, Relaxed);
                    }
                }
                gst::PadProbeReturn::Ok
            });
        }
        pipeline.set_state(gst::State::Playing).unwrap();
        Self { puerto, cuentas, pipeline }
    }
}

impl Drop for Receptor {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
