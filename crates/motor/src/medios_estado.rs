//! What the running pipeline reports: measured fps and bitrate, and which stage failed.

use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Counts the encoded frames and bytes that leave the parser, read as a rate between calls.
pub struct Contador {
    cuadros: AtomicU64,
    bytes: AtomicU64,
    ultima: Mutex<(Instant, u64, u64)>,
}

impl Contador {
    pub fn nuevo() -> Self {
        Self {
            cuadros: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            ultima: Mutex::new((Instant::now(), 0, 0)),
        }
    }

    pub fn sumar(&self, bytes: usize) {
        self.cuadros.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// (frames per second, kbit per second) since the previous call.
    pub fn tasa(&self) -> (f32, u32) {
        let (c, b) = (self.cuadros.load(Ordering::Relaxed), self.bytes.load(Ordering::Relaxed));
        let mut u = self.ultima.lock().unwrap_or_else(|e| e.into_inner());
        let dt = u.0.elapsed().as_secs_f32().max(0.001);
        let tasa = ((c - u.1) as f32 / dt, ((b - u.2) as f32 * 8.0 / dt / 1000.0) as u32);
        *u = (Instant::now(), c, b);
        tasa
    }
}

/// Which part of the pipeline raised an error; the engine reacts differently to each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    Codificador,
    Captura,
    Otra,
}

/// The first error seen on the bus, once there is one.
pub type Aviso = Option<(Clase, String)>;

/// Elements are named after their role: `cod` and `pre` belong to the encoder stage
/// (conversion plus encoder), `fuente` is the screen capture.
pub fn clasificar(msg: &gst::message::Error) -> (Clase, String) {
    let ruta = msg.src().map(|s| s.path_string().to_string()).unwrap_or_default();
    let es = |nombre: &str| ruta.split('/').any(|t| t.ends_with(&format!(":{nombre}")));
    let clase = if es("cod") || es("pre") {
        Clase::Codificador
    } else if es("fuente") {
        Clase::Captura
    } else {
        Clase::Otra
    };
    let detalle = match msg.debug() {
        Some(d) => format!("{} ({d})", msg.error()),
        None => msg.error().to_string(),
    };
    (clase, detalle)
}

/// Upstream request for an IDR frame with SPS/PPS, what `gst_video_event_new_upstream_force_key_unit`
/// builds (done by hand: the video crate is not a dependency).
pub fn evento_idr() -> gst::Event {
    gst::event::CustomUpstream::new(
        gst::Structure::builder("GstForceKeyUnit")
            .field("running-time", u64::MAX)
            .field("all-headers", true)
            .field("count", 0u32)
            .build(),
    )
}
