//! Frames that never reach the TV. The leaky queue before the encoder (`fuga`) throws away the
//! oldest frame when the encoder is busy: those are the stutters this computer causes. The
//! frame-rate adapter (`tasa`) drops and duplicates to hold the negotiated rate; that is
//! expected (a 60 Hz screen sent at 30 fps drops half), so it is logged only as context.

use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Arc;

pub struct Descartes {
    cola: Option<gst::Element>,
    tasa: Option<gst::Element>,
    entran: Arc<AtomicU64>,
    salen: Arc<AtomicU64>,
}

impl Descartes {
    pub fn instalar(tuberia: &gst::Pipeline) -> Self {
        let (entran, salen) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
        let cola = tuberia.by_name("fuga");
        for (pad, cuenta) in [("sink", &entran), ("src", &salen)] {
            let (pad, c) = (cola.as_ref().and_then(|e| e.static_pad(pad)), cuenta.clone());
            pad.map(|p| {
                p.add_probe(gst::PadProbeType::BUFFER, move |_, _| {
                    c.fetch_add(1, Relaxed);
                    gst::PadProbeReturn::Ok
                })
            });
        }
        Self { cola, tasa: tuberia.by_name("tasa"), entran, salen }
    }

    /// Frames the leaky queue dropped since the start.
    pub fn cola(&self) -> u64 {
        let guardados =
            self.cola.as_ref().map_or(0, |c| u64::from(c.property::<u32>("current-level-buffers")));
        perdidos(self.entran.load(Relaxed), self.salen.load(Relaxed), guardados)
    }

    /// (dropped, duplicated) by videorate since the start.
    pub fn tasa(&self) -> (u64, u64) {
        self.tasa.as_ref().map_or((0, 0), |t| (t.property::<u64>("drop"), t.property::<u64>("duplicate")))
    }
}

/// What went in and neither came out nor is still waiting was dropped.
pub fn perdidos(entran: u64, salen: u64, guardados: u64) -> u64 {
    entran.saturating_sub(salen).saturating_sub(guardados)
}

#[cfg(test)]
mod tests {
    #[test]
    fn lo_que_no_sale_ni_espera_se_perdio() {
        assert_eq!(super::perdidos(100, 97, 1), 2);
        assert_eq!(super::perdidos(5, 5, 1), 0, "una lectura a destiempo no da negativos");
    }
}
