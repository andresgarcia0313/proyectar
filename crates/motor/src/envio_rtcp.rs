//! Listens for the TV's RTCP reports: loss and jitter go to the statistics and the log, and
//! steady loss lowers the bitrate through `Ritmo`, as repeated IDR requests already do.

use crate::medios_ritmo::Ritmo;
use crate::registro;
use crate::rtcp::{leer, Informe, SSRC};
use serde_json::json;
use std::io::ErrorKind;
use std::net::{IpAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The last report the TV sent; all zero until the first one arrives.
#[derive(Default)]
pub struct Calidad(Mutex<Informe>);

impl Calidad {
    pub fn ultimo(&self) -> Informe {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Everything the listening thread needs; `ajustar` hands a new bitrate to the encoder.
pub struct Escucha<A> {
    pub desde: IpAddr,
    pub calidad: Arc<Calidad>,
    pub ritmo: Arc<Ritmo>,
    pub ajustar: A,
}

/// Reads reports from `desde` until `parar` is set (checked every 200 ms).
pub fn escuchar<A: Fn(u32)>(sock: UdpSocket, e: Escucha<A>, parar: &AtomicBool) {
    let _ = sock.set_read_timeout(Some(Duration::from_millis(200)));
    let (mut buf, mut vistos) = ([0u8; 1500], 0u64);
    while !parar.load(Relaxed) {
        let (n, origen) = match sock.recv_from(&mut buf) {
            Ok(r) => r,
            Err(f) if matches!(f.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => continue,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
        };
        let Some(informe) = (origen.ip() == e.desde).then(|| leer(&buf[..n], SSRC)).flatten() else {
            continue;
        };
        *e.calidad.0.lock().unwrap_or_else(|p| p.into_inner()) = informe;
        vistos += 1;
        let kbps = e.ritmo.informe(informe.perdida_pct);
        // The first report proves RTCP gets through; after that only the ones that matter.
        if vistos == 1 || informe.perdida_pct > 0.0 || kbps.is_some() {
            registro::anotar(
                "rtcp",
                json!({"perdida_pct": informe.perdida_pct, "perdidos": informe.perdidos,
                       "jitter_ms": informe.jitter_ms, "informes": vistos, "kbps_nuevo": kbps}),
            );
        }
        if let Some(kbps) = kbps {
            (e.ajustar)(kbps);
        }
    }
}
