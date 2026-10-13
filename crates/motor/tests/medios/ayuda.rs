//! Test helpers: a video mode, a started pipeline aimed at the stand-in receiver, and the
//! per-frame encoder delay probe.

use super::receptor::Receptor;
use gstreamer::prelude::*;
use motor::medios::{Codificador, Medios, Origen};
use motor::modo::Latencia;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wfd::{AudioCodec, VideoMode};

pub fn modo(ancho: u32, alto: u32, fps: u32, profile: u8) -> VideoMode {
    VideoMode { cea_bit: 7, width: ancho, height: alto, fps, profile, level: 0x04 }
}

pub fn iniciar(r: &Receptor, m: VideoMode, cod: Codificador) -> motor::fallo::Resultado<Medios> {
    let destino = format!("127.0.0.1:{}", r.puerto).parse().unwrap();
    let perfil = Latencia::Trabajo.perfil();
    Medios::iniciar(Origen::Prueba, destino, m, Some(AudioCodec::Aac), perfil, cod, false)
}

/// Encode delay per frame: time between entering and leaving the encoder, in milliseconds.
pub fn medir_latencia(m: &Medios) -> Arc<Mutex<Vec<f64>>> {
    let (entradas, salidas) = (Arc::new(Mutex::new(VecDeque::new())), Arc::new(Mutex::new(vec![])));
    let cod = m.tuberia().by_name("cod").unwrap();
    let (e, s) = (entradas.clone(), salidas.clone());
    cod.static_pad("sink").unwrap().add_probe(gstreamer::PadProbeType::BUFFER, move |_, i| {
        if let Some(gstreamer::PadProbeData::Buffer(b)) = &i.data {
            let _ = b;
            e.lock().unwrap().push_back(Instant::now());
        }
        gstreamer::PadProbeReturn::Ok
    });
    cod.static_pad("src").unwrap().add_probe(gstreamer::PadProbeType::BUFFER, move |_, i| {
        if let Some(gstreamer::PadProbeData::Buffer(b)) = &i.data {
            let _ = b;
            if let Some(t) = entradas.lock().unwrap().pop_front() {
                s.lock().unwrap().push(t.elapsed().as_secs_f64() * 1000.0);
            }
        }
        gstreamer::PadProbeReturn::Ok
    });
    salidas
}
