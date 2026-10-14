//! Token bucket that spreads the RTP packets at a steady rate: a frame leaves as an even stream
//! instead of one burst that overflows the Wi-Fi queue (udpsink with sync=false does not pace).

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// Bytes allowed back to back before pacing starts: four full RTP packets.
pub const RAFAGA_BYTES: f64 = 4.0 * 1328.0;
/// Below this pace the bucket would starve the audio and the transport overhead.
const MINIMO_BPS: f64 = 1_000_000.0;

pub struct Cubo {
    fichas: f64,
    ultimo: Instant,
}

impl Cubo {
    pub fn nuevo(ahora: Instant) -> Self {
        Self { fichas: RAFAGA_BYTES, ultimo: ahora }
    }

    /// Takes `bytes` at `bps` bit/s and says how long to wait before sending them.
    pub fn tomar(&mut self, bytes: usize, bps: f64, ahora: Instant) -> Duration {
        let bps = bps.max(MINIMO_BPS);
        let llenado = ahora.saturating_duration_since(self.ultimo).as_secs_f64() * bps / 8.0;
        self.fichas = (self.fichas + llenado).min(RAFAGA_BYTES) - bytes as f64;
        self.ultimo = ahora;
        Duration::from_secs_f64((-self.fichas).max(0.0) * 8.0 / bps)
    }

    /// Forgives the debt: a packet that is already late leaves at once.
    pub fn perdonar(&mut self) {
        self.fichas = self.fichas.max(0.0);
    }
}

/// How the sending thread behaves; `bps` is read for every packet so bitrate changes apply at once.
pub struct Reparto<B, F> {
    pub destino: SocketAddr,
    pub bps: B,
    /// A packet that waited this long since the pipeline handed it over is not held any longer.
    pub atraso_max: Duration,
    /// Called once per run of failed sends that reaches `fallos_max`.
    pub fallar: F,
    pub fallos_max: u32,
    /// Packets and bytes sent and the last RTP timestamp, for our RTCP sender reports.
    pub cuenta: std::sync::Arc<crate::rtcp_sr::Cuenta>,
}

/// Sends what arrives on `rx` until it closes or `parar` is set.
pub fn repartir<P, B, F>(rx: Receiver<(Instant, P)>, sock: UdpSocket, r: Reparto<B, F>, parar: &AtomicBool)
where
    P: AsRef<[u8]>,
    B: Fn() -> f64,
    F: Fn(io::Error),
{
    let (mut cubo, mut fallos) = (Cubo::nuevo(Instant::now()), 0);
    while !parar.load(Relaxed) {
        let (llegada, paquete) = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(p) => p,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let datos = paquete.as_ref();
        let espera = cubo.tomar(datos.len(), (r.bps)(), Instant::now());
        if llegada.elapsed() + espera > r.atraso_max {
            cubo.perdonar();
        } else if !espera.is_zero() {
            std::thread::sleep(espera);
        }
        match sock.send_to(datos, r.destino) {
            Ok(_) => {
                fallos = 0;
                r.cuenta.anotar(datos, Instant::now());
            }
            Err(e) => {
                fallos += 1;
                if fallos == r.fallos_max {
                    (r.fallar)(e);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "envio_cubo_pruebas.rs"]
mod pruebas;
