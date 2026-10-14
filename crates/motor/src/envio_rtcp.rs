//! Listens for the TV's RTCP reports: loss and jitter go to the statistics and the log, and
//! steady loss lowers the bitrate through `Ritmo`, as repeated IDR requests already do. Every
//! second it also sends our sender report, whose echo gives the round trip of the link.

use crate::medios_ritmo::Ritmo;
use crate::registro;
use crate::rtcp::{leer, Informe, SSRC};
use crate::rtcp_sr::{medio, ntp, rtt_ms, Cuenta};
use serde_json::json;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Sender reports this often, as RFC 3550 suggests for a single sender at our bitrate.
const CADA_SR: Duration = Duration::from_secs(1);

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
    /// The TV's RTCP port (its RTP port plus one) and what has been sent, for our reports.
    pub tv_rtcp: SocketAddr,
    pub cuenta: Arc<Cuenta>,
}

/// Reads reports from `desde` until `parar` is set (checked every 200 ms).
pub fn escuchar<A: Fn(u32)>(sock: UdpSocket, e: Escucha<A>, parar: &AtomicBool) {
    let _ = sock.set_read_timeout(Some(Duration::from_millis(200)));
    let (mut buf, mut vistos, mut ultimo_sr) = ([0u8; 1500], 0u64, None::<Instant>);
    while !parar.load(Relaxed) {
        if ultimo_sr.is_none_or(|t| t.elapsed() >= CADA_SR) {
            let informe = e.cuenta.informe(SSRC, Instant::now(), ntp(SystemTime::now()));
            if let Some(sr) = informe {
                if ultimo_sr.is_none() {
                    registro::anotar("rtcp_sr", json!({"destino": e.tv_rtcp.to_string()}));
                }
                let _ = sock.send_to(&sr, e.tv_rtcp);
                ultimo_sr = Some(Instant::now());
            }
        }
        let (n, origen) = match sock.recv_from(&mut buf) {
            Ok(r) => r,
            Err(f) if matches!(f.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => continue,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
        };
        let llegada = medio(ntp(SystemTime::now()));
        let Some(mut informe) = (origen.ip() == e.desde).then(|| leer(&buf[..n], SSRC)).flatten() else {
            continue;
        };
        let mut guardado = e.calidad.0.lock().unwrap_or_else(|p| p.into_inner());
        // A report that echoes no SR keeps the last round trip seen.
        informe.rtt_ms = rtt_ms(informe.lsr, informe.dlsr, llegada).unwrap_or(guardado.rtt_ms);
        *guardado = informe;
        drop(guardado);
        vistos += 1;
        let kbps = e.ritmo.informe(informe.perdida_pct);
        // The first report proves RTCP gets through; after that only the ones that matter.
        if vistos == 1 || informe.perdida_pct > 0.0 || kbps.is_some() {
            registro::anotar(
                "rtcp",
                json!({"perdida_pct": informe.perdida_pct, "perdidos": informe.perdidos,
                       "jitter_ms": informe.jitter_ms, "rtt_ms": informe.rtt_ms, "informes": vistos, "kbps_nuevo": kbps}),
            );
        }
        if let Some(kbps) = kbps {
            (e.ajustar)(kbps);
        }
    }
}
