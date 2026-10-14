//! Own UDP sender in place of udpsink: the appsink `salida` hands over each RTP packet and a
//! thread paces it at 4x the bitrate with DSCP EF, from the port the RTSP SETUP announced
//! (`server_port=5000-5001`); the TV sends its RTCP reports to the next one, read by another thread.

use crate::envio_cubo::{repartir, Reparto};
use crate::envio_red::{avisar_fallo, enganchar, socket};
use crate::envio_rtcp::{escuchar, Calidad, Escucha};
use crate::fallo::{Fallo, Resultado};
use crate::medios_ritmo::{aplicar, Ritmo};
use crate::medios_tipos::Codificador;
use crate::{prioridad, registro};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

pub const PUERTO_RTP: u16 = 5000;
pub const PUERTO_RTCP: u16 = 5001;
/// Pace relative to the encoder bitrate: room for audio, TS overhead and bigger frames.
const FACTOR: f64 = 4.0;
/// A packet held this long leaves at once: pacing must not pile delay onto a big frame.
const ATRASO_MAX: Duration = Duration::from_millis(20);
/// Failed sends in a row before the projection gives up (one is a hiccup, twenty a dead link).
const FALLOS_MAX: u32 = 20;

/// Both threads; dropping it stops them within 200 ms and frees the ports.
pub struct Emisor {
    parar: Arc<AtomicBool>,
    hilos: Vec<JoinHandle<()>>,
    calidad: Arc<Calidad>,
    puerto_rtcp: u16,
}

fn medios(e: impl std::fmt::Display) -> Fallo {
    Fallo::Medios(format!("envío UDP: {e}"))
}

impl Emisor {
    pub fn instalar(
        tuberia: &gst::Pipeline,
        destino: SocketAddr,
        ritmo: Arc<Ritmo>,
        cod: Codificador,
    ) -> Resultado<Self> {
        let salida = tuberia.by_name("salida").ok_or_else(|| medios("falta salida"))?;
        let (rtp, rtcp) = (socket(PUERTO_RTP).map_err(medios)?, socket(PUERTO_RTCP).map_err(medios)?);
        let puerto_rtcp = rtcp.local_addr().map_err(medios)?.port();
        let rx = enganchar(&salida);
        let (parar, calidad) = (Arc::new(AtomicBool::new(false)), Arc::new(Calidad::default()));
        let (p, r, runtime) = (parar.clone(), ritmo.clone(), tokio::runtime::Handle::try_current().ok());
        let enviar = Builder::new().name("envio-rtp".into()).spawn(move || {
            prioridad::subir_este_hilo(&runtime);
            let bps = move || f64::from(r.actual()) * 1000.0 * FACTOR;
            let fallar = avisar_fallo(salida, destino);
            let reparto = Reparto { destino, bps, atraso_max: ATRASO_MAX, fallar, fallos_max: FALLOS_MAX };
            repartir(rx, rtp, reparto, &p);
        });
        let (p, codificador) = (parar.clone(), tuberia.by_name("cod"));
        let ajustar = move |kbps| {
            if let Some(c) = &codificador {
                aplicar(c, cod, kbps);
            }
        };
        let escucha = Escucha { desde: destino.ip(), calidad: calidad.clone(), ritmo, ajustar };
        let oir = Builder::new().name("rtcp".into()).spawn(move || escuchar(rtcp, escucha, &p));
        let hilos = [enviar, oir].into_iter().collect::<Result<Vec<_>, _>>().map_err(medios)?;
        let datos = serde_json::json!({"destino": destino.to_string(), "puerto_rtcp": puerto_rtcp});
        registro::anotar("envio", datos);
        Ok(Self { parar, hilos, calidad, puerto_rtcp })
    }

    /// The TV's last RTCP report on this stream.
    pub fn calidad(&self) -> crate::rtcp::Informe {
        self.calidad.ultimo()
    }

    /// Where this sender listens for RTCP (the fixed one unless it was taken).
    pub fn puerto_rtcp(&self) -> u16 {
        self.puerto_rtcp
    }
}

impl Drop for Emisor {
    fn drop(&mut self) {
        self.parar.store(true, Relaxed);
        self.hilos.drain(..).for_each(|h| drop(h.join()));
    }
}
