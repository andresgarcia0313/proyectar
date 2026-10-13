//! Adaptive bitrate, as Windows does it: a display that keeps asking for full frames is
//! losing packets, so the picture gets lighter; after a calm spell it climbs back.

use crate::medios_tipos::Codificador;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Two requests this close mean steady loss, not a one-off glitch.
const RAFAGA: Duration = Duration::from_secs(5);
/// Calm needed before each step back up.
const CALMA: Duration = Duration::from_secs(15);
/// Never below this share of the mode's bitrate: text must stay readable.
const PISO: u32 = 40;

pub struct Ritmo {
    maximo: u32,
    /// (current share in percent, last full-frame request, last change)
    estado: Mutex<(u32, Option<Instant>, Instant)>,
}

impl Ritmo {
    pub fn nuevo(kbps: u32) -> Self {
        Self { maximo: kbps, estado: Mutex::new((100, None, Instant::now())) }
    }

    /// The display asked for a full frame; returns the lower bitrate when loss looks steady.
    pub fn perdida(&self) -> Option<u32> {
        let mut e = self.estado.lock().unwrap_or_else(|e| e.into_inner());
        let seguida = e.1.is_some_and(|t| t.elapsed() < RAFAGA);
        e.1 = Some(Instant::now());
        if !seguida || e.0 <= PISO {
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
}
