//! The slower inputs of a session summary: samples every ten seconds, IDR requests, falls,
//! and the counters that restart with each pipeline.

use crate::monitor::Radio;
use crate::monitor_carga::Carga;
use crate::resumen::Acumulador;
use std::time::Instant;

impl Acumulador {
    /// What is sampled every ten seconds: the radio, this process' CPU and the computer's load.
    pub fn muestra(&mut self, radio: Option<&Radio>, cpu: f32, carga: Carga) {
        self.m.cpu.poner(cpu);
        self.m.carga.poner_si(carga.carga_1m);
        self.m.psi.poner_si(carga.psi_cpu_pct);
        let Some(r) = radio else { return };
        self.m.senal.poner_si(r.senal_dbm.map(|v| v as f32));
        self.m.tx.poner_si(r.tx_mbps);
        self.ahorro = match (self.ahorro, r.ahorro) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (previo, actual) => actual.or(previo),
        };
        if let Some(n) = r.reintentos {
            self.reintentos_tramo = Some(self.reintentos_tramo.map_or((n, n), |(a, _)| (a, n)));
        }
    }

    pub fn idr(&mut self) {
        self.idr += 1;
    }

    /// A session failed and the cycle will try again; several failures in a row are one fall.
    pub fn caida(&mut self) {
        if self.caida.is_none() {
            (self.caida, self.reconexiones) = (Some(Instant::now()), self.reconexiones + 1);
        }
    }

    /// Folds the running pipeline's counters (they restart with each one) into the totals.
    pub fn cerrar_tramo(&mut self) {
        let (c, t) = (&mut self.cerrado, std::mem::take(&mut self.tramo));
        (c.descartes, c.tasa_descartes, c.tasa_duplicados) = (
            c.descartes + t.descartes,
            c.tasa_descartes + t.tasa_descartes,
            c.tasa_duplicados + t.tasa_duplicados,
        );
        (c.audio_medido_s, c.audio_senal_s) =
            (c.audio_medido_s + t.audio_medido_s, c.audio_senal_s + t.audio_senal_s);
        self.reintentos += self.reintentos_tramo.take().map_or(0, |(a, b)| b.saturating_sub(a));
        self.tramo_s = 0;
    }

    /// A released virtual screen: one left behind in any release of the session stays on record.
    pub fn quitada(&mut self, quitada: Option<bool>) {
        if quitada.is_some() && self.salida_quitada != Some(false) {
            self.salida_quitada = quitada;
        }
    }
}

/// The audio the pipeline sends: LPCM when agreed, AAC otherwise (silence included).
pub fn codec(audio: Option<wfd::AudioCodec>) -> String {
    match audio {
        Some(wfd::AudioCodec::Lpcm) => "lpcm".into(),
        _ => "aac".into(),
    }
}

/// After the session: the default output is no longer the TV's (`None` if pactl did not answer).
pub fn audio_fuera_del_tv() -> Option<bool> {
    use crate::salida_audio_cmd::{elegida, es_propio};
    elegida().ok().map(|actual| !es_propio(actual.as_deref()))
}
