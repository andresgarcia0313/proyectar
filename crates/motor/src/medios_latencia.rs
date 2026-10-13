//! Delay inside this laptop, frame by frame: from capture (the buffer timestamp is set when
//! the frame arrives) to the moment its packet leaves for the TV, on the pipeline clock.
//! The TV adds its own buffer on top; this is the part we control.

use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Arc;

const DESPLAZAMIENTO: gst::ClockTime = gst::ClockTime::from_seconds(1000 * 3600);

#[derive(Default)]
pub struct Retardo {
    suma_us: AtomicU64,
    cuenta: AtomicU64,
    peor_us: AtomicU64,
}

impl Retardo {
    /// Watches what crosses `pad` of the element named `elemento`.
    pub fn vigilar(tuberia: &gst::Pipeline, elemento: &str, pad: &str) -> Option<Arc<Self>> {
        let pad = tuberia.by_name(elemento)?.static_pad(pad)?;
        let retardo = Arc::new(Self::default());
        let r = retardo.clone();
        pad.add_probe(gst::PadProbeType::BUFFER | gst::PadProbeType::BUFFER_LIST, move |pad, info| {
            let pts = match &info.data {
                Some(gst::PadProbeData::Buffer(b)) => b.pts(),
                Some(gst::PadProbeData::BufferList(l)) => l.get(0).and_then(|b| b.pts()),
                _ => None,
            };
            if let (Some(pts), Some(ahora)) = (pts, reloj(pad)) {
                // Video encoders shift timestamps by 1000 h so DTS never goes negative.
                let pts = if pts > ahora { pts.saturating_sub(DESPLAZAMIENTO) } else { pts };
                r.anotar(ahora.saturating_sub(pts).useconds());
            }
            gst::PadProbeReturn::Ok
        });
        Some(retardo)
    }

    fn anotar(&self, us: u64) {
        // Anything over 10 s is a timestamp from another clock, not a delay.
        if us < 10_000_000 {
            self.suma_us.fetch_add(us, Relaxed);
            self.cuenta.fetch_add(1, Relaxed);
            self.peor_us.fetch_max(us, Relaxed);
        }
    }

    /// Average and worst delay in milliseconds since the last call; starts a new window.
    pub fn tomar(&self) -> (f32, f32) {
        let (suma, cuenta) = (self.suma_us.swap(0, Relaxed), self.cuenta.swap(0, Relaxed));
        let peor = self.peor_us.swap(0, Relaxed);
        if cuenta == 0 {
            return (0.0, 0.0);
        }
        (suma as f32 / cuenta as f32 / 1000.0, peor as f32 / 1000.0)
    }
}

/// The pipeline's running time now, the same timeline as buffer timestamps.
fn reloj(pad: &gst::Pad) -> Option<gst::ClockTime> {
    let elemento = pad.parent_element()?;
    Some(elemento.clock()?.time().saturating_sub(elemento.base_time()?))
}

#[cfg(test)]
mod tests {
    use super::Retardo;

    #[test]
    fn promedio_y_peor_por_ventana() {
        let r = Retardo::default();
        r.anotar(10_000);
        r.anotar(30_000);
        r.anotar(60_000_000);
        assert_eq!(r.tomar(), (20.0, 30.0));
        assert_eq!(r.tomar(), (0.0, 0.0));
    }
}
