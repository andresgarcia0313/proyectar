//! One session for the summary: the same options from the first attempt until it ends (closed
//! by the user or the TV, failed for good, or new options), across reconnections. Fed while
//! streaming; `resumen_informe` turns it into the `resumen_sesion` event.

use crate::modo::Opciones;
use crate::resumen_serie::{Muestras, Serie};
use crate::{registro, Estadisticas};
use std::time::Instant;

/// The first seconds of each pipeline are its warm-up, not its frame rate.
const ARRANQUE_S: u32 = 3;

#[derive(Debug, Default)]
pub struct Acumulador {
    pub pantalla: String,
    pub opciones: Option<Opciones>,
    pub captura: Option<&'static str>,
    pub codificadores: Vec<&'static str>,
    pub fps_objetivo: u32,
    pub audio_codec: Option<String>,
    pub inicio: Option<Instant>,
    pub hora_inicio: String,
    pub transmision_s: u32,
    pub m: Muestras,
    pub idr: u32,
    pub reconexiones: u32,
    pub recuperaciones: Serie,
    pub caida: Option<Instant>,
    pub salida_virtual: Option<bool>,
    pub salida_quitada: Option<bool>,
    pub audio_restaurada: Option<bool>,
    /// Power save as the radio reported it: `Some(true)` once it was seen on.
    pub ahorro: Option<bool>,
    /// Counters of the pipelines already closed, plus the running one's last reading.
    pub cerrado: Estadisticas,
    pub tramo: Estadisticas,
    pub(crate) tramo_s: u32,
    pub reintentos: u64,
    pub(crate) reintentos_tramo: Option<(u64, u64)>,
}

impl Acumulador {
    pub fn nuevo(pantalla: &str) -> Self {
        let hora_inicio = crate::resumen_informe::hora();
        Self { pantalla: pantalla.to_owned(), inicio: Some(Instant::now()), hora_inicio, ..Self::default() }
    }

    /// A new session after new options; a capture kept across them keeps its screen check.
    pub fn reiniciar(&mut self) {
        *self = Self { salida_virtual: self.salida_virtual, ..Self::nuevo(&self.pantalla) };
    }

    /// The picture reached the TV: a new pipeline starts, and a pending fall is recovered.
    pub fn transmitiendo(
        &mut self,
        opciones: &Opciones,
        cod: &'static str,
        fps: u32,
        audio: String,
        captura: &'static str,
    ) {
        self.cerrar_tramo();
        (self.opciones, self.fps_objetivo, self.audio_codec) = (Some(opciones.clone()), fps, Some(audio));
        self.captura = Some(captura);
        if !self.codificadores.contains(&cod) {
            self.codificadores.push(cod);
        }
        if let Some(caida) = self.caida.take() {
            let s = caida.elapsed().as_secs_f32();
            self.recuperaciones.poner(s);
            registro::anotar("recuperado", serde_json::json!({"s": s, "reconexiones": self.reconexiones}));
        }
    }

    /// The numbers of one streaming second.
    pub fn segundo(&mut self, e: &Estadisticas) {
        (self.transmision_s, self.tramo_s, self.tramo) = (self.transmision_s + 1, self.tramo_s + 1, *e);
        if self.tramo_s > ARRANQUE_S {
            self.m.fps.poner(e.fps);
            self.m.kbps.poner(e.kbps as f32);
        }
        self.m.retardo.poner_si((e.retardo_ms > 0.0).then_some(e.retardo_ms));
        // All zero means no RTCP report yet (a real one always carries some jitter).
        if e.jitter_ms > 0.0 || e.perdidos != 0 || e.perdida_pct > 0.0 {
            self.m.perdida.poner(e.perdida_pct);
            self.m.jitter.poner(e.jitter_ms);
        }
        self.m.rtt.poner_si((e.rtt_ms > 0.0).then_some(e.rtt_ms));
    }
}

#[cfg(test)]
#[path = "resumen_pruebas.rs"]
pub(crate) mod pruebas;
