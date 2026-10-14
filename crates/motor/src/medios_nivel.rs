//! Whether the TV's sound carries anything: the `level` element of the audio branch posts the
//! RMS of every second and this counts the seconds measured and the ones above silence. It
//! only reads the samples on their way to the TV; nothing is played on this computer.

use gstreamer as gst;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// RMS above this (dBFS) is sound: room noise picked up digitally stays below -70, the
/// silence generator posts -inf (or -700), quiet speech and music sit between -40 and -20.
pub const SILENCIO_DB: f64 = -60.0;
/// What `level` posts: one message per second.
pub const INTERVALO_NS: u64 = 1_000_000_000;

#[derive(Debug, Default)]
pub struct Nivel {
    medidos: AtomicU32,
    con_senal: AtomicU32,
}

impl Nivel {
    /// A `level` message from the bus; anything else is ignored.
    pub fn mensaje(&self, s: &gst::StructureRef) {
        if s.name() != "level" {
            return;
        }
        let Ok(rms) = s.get::<gst::glib::ValueArray>("rms") else { return };
        let canales: Vec<f64> = rms.iter().filter_map(|v| v.get::<f64>().ok()).collect();
        self.anotar(&canales);
    }

    /// One second of audio with the RMS of each channel, in dBFS.
    pub fn anotar(&self, rms_db: &[f64]) {
        self.medidos.fetch_add(1, Relaxed);
        if con_senal(rms_db) {
            self.con_senal.fetch_add(1, Relaxed);
        }
    }

    /// (seconds measured, seconds with sound) since the pipeline started.
    pub fn segundos(&self) -> (u32, u32) {
        (self.medidos.load(Relaxed), self.con_senal.load(Relaxed))
    }
}

/// A second has sound when any channel is above silence.
pub fn con_senal(rms_db: &[f64]) -> bool {
    rms_db.iter().any(|v| v.is_finite() && *v > SILENCIO_DB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuenta_solo_los_segundos_con_sonido() {
        let n = Nivel::default();
        n.anotar(&[f64::NEG_INFINITY, f64::NEG_INFINITY]);
        n.anotar(&[-700.0, -65.0]);
        n.anotar(&[-80.0, -32.5]);
        assert_eq!(n.segundos(), (3, 1));
    }

    /// Real `level` messages from a pipeline that ends in fakesink: nothing is played.
    fn medir(onda: &str) -> (u32, u32) {
        use gstreamer::prelude::*;
        gst::init().unwrap();
        let texto = format!("audiotestsrc wave={onda} num-buffers=20 ! level interval=100000000 ! fakesink");
        let tuberia = gst::parse::launch(&texto).unwrap();
        tuberia.set_state(gst::State::Playing).unwrap();
        let n = Nivel::default();
        for msg in tuberia.bus().unwrap().iter_timed(gst::ClockTime::from_seconds(5)) {
            match msg.view() {
                gst::MessageView::Element(e) => n.mensaje(e.structure().unwrap()),
                gst::MessageView::Eos(_) | gst::MessageView::Error(_) => break,
                _ => {}
            }
        }
        tuberia.set_state(gst::State::Null).unwrap();
        n.segundos()
    }

    #[test]
    fn lee_los_mensajes_de_level() {
        let (medidos, con) = medir("sine");
        assert!(medidos >= 4 && con == medidos, "tono: {medidos} medidos, {con} con señal");
        let (medidos, con) = medir("silence");
        assert!(medidos >= 4 && con == 0, "silencio: {medidos} medidos, {con} con señal");
    }
}
