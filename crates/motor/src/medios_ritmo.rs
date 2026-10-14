//! Adaptive bitrate, as Windows does it: a display that keeps asking for full frames is
//! losing packets, so the picture gets lighter; after a calm spell it climbs back.

use crate::medios_tipos::Codificador;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Two requests this close mean steady loss, not a one-off glitch.
const RAFAGA: Duration = Duration::from_secs(5);
/// Calm needed before each step back up.
const CALMA: Duration = Duration::from_secs(15);
/// Never below this share of the mode's bitrate: text must stay readable.
const PISO: u32 = 40;
/// Loss in an RTCP report that counts; less is ordinary Wi-Fi noise the TV conceals.
const UMBRAL_PCT: f32 = 2.0;

pub struct Ritmo {
    maximo: u32,
    /// (current share in percent, last full-frame request, last change)
    estado: Mutex<Estado>,
    /// The previous RTCP report already showed loss.
    previo: AtomicBool,
}

type Estado = (u32, Option<Instant>, Instant);

impl Ritmo {
    pub fn nuevo(kbps: u32) -> Self {
        Self { maximo: kbps, estado: Mutex::new((100, None, Instant::now())), previo: AtomicBool::new(false) }
    }

    /// The display asked for a full frame; returns the lower bitrate when loss looks steady.
    pub fn perdida(&self) -> Option<u32> {
        let mut e = self.estado.lock().unwrap_or_else(|e| e.into_inner());
        let seguida = e.1.is_some_and(|t| t.elapsed() < RAFAGA);
        e.1 = Some(Instant::now());
        if !seguida {
            return None;
        }
        self.bajar(&mut e)
    }

    /// An RTCP report: two lossy ones in a row step the bitrate down, like steady IDR requests.
    pub fn informe(&self, perdida_pct: f32) -> Option<u32> {
        if perdida_pct < UMBRAL_PCT {
            self.previo.store(false, Relaxed);
            return None;
        }
        let mut e = self.estado.lock().unwrap_or_else(|e| e.into_inner());
        e.1 = Some(Instant::now());
        if !self.previo.swap(true, Relaxed) {
            return None;
        }
        self.previo.store(false, Relaxed);
        self.bajar(&mut e)
    }

    /// The bitrate in force now, in kbit/s.
    pub fn actual(&self) -> u32 {
        self.maximo * self.estado.lock().unwrap_or_else(|e| e.into_inner()).0 / 100
    }

    fn bajar(&self, e: &mut Estado) -> Option<u32> {
        if e.0 <= PISO {
            return None;
        }
        (e.0, e.2) = ((e.0 * 3 / 4).max(PISO), Instant::now());
        Some(self.maximo * e.0 / 100)
    }

    /// Called every second; returns a 10 % higher bitrate after a calm spell.
    pub fn calma(&self) -> Option<u32> {
        let mut e = self.estado.lock().unwrap_or_else(|e| e.into_inner());
        let tranquilo = e.1.is_none_or(|t| t.elapsed() > CALMA) && e.2.elapsed() > CALMA;
        if e.0 >= 100 || !tranquilo {
            return None;
        }
        (e.0, e.2) = ((e.0 + 10).min(100), Instant::now());
        Some(self.maximo * e.0 / 100)
    }
}

/// Sets the running encoder's bitrate, when that encoder accepts changes while playing.
pub fn aplicar(cod: &gst::Element, codificador: Codificador, kbps: u32) {
    let valor = match codificador {
        Codificador::OpenH264 => kbps * 1000,
        Codificador::V4l2 => return,
        _ => kbps,
    };
    let cambiable =
        cod.find_property("bitrate").is_some_and(|p| p.flags().contains(gst::PARAM_FLAG_MUTABLE_PLAYING));
    if cambiable {
        cod.set_property("bitrate", valor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baja_con_perdida_seguida_y_no_por_un_pedido_suelto() {
        let r = Ritmo::nuevo(10_000);
        assert_eq!(r.perdida(), None);
        assert_eq!(r.perdida(), Some(7_500));
        assert_eq!(r.perdida(), Some(5_600));
        for _ in 0..5 {
            r.perdida();
        }
        assert_eq!(r.perdida(), None, "nunca baja del piso");
        assert_eq!(r.calma(), None, "sin calma no sube");
    }

    #[test]
    fn la_perdida_rtcp_sostenida_baja_y_la_suelta_no() {
        let r = Ritmo::nuevo(10_000);
        assert_eq!(
            (r.informe(0.0), r.informe(5.0), r.informe(1.0), r.informe(5.0)),
            (None, None, None, None)
        );
        assert_eq!(r.informe(5.0), Some(7_500), "dos informes con pérdida seguidos");
        assert_eq!(r.informe(5.0), None, "cada escalón pide otro par");
        assert_eq!(r.informe(9.0), Some(5_600));
        assert_eq!(r.actual(), 5_600);
        assert_eq!(r.calma(), None, "la pérdida reciente impide subir");
    }
}
