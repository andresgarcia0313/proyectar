//! The `resumen_sesion` event: figures of one session and a verdict for each manual test.
//! Measures that could not be taken stay `null`; the verdicts say why.

use crate::modo::{Fuente, Latencia};
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Resumen {
    pub pantalla: String,
    pub modo: Option<Latencia>,
    pub fuente: Option<Fuente>,
    pub captura: Option<&'static str>,
    pub codificadores: Vec<&'static str>,
    /// Why it ended: "fin" (user or TV), "fallo" (gave up), "opciones" (new options).
    pub motivo: &'static str,
    pub duracion_s: f32,
    pub transmision_s: u32,
    pub fps_objetivo: u32,
    pub fps_media: Option<f32>,
    pub fps_p5: Option<f32>,
    pub kbps_media: Option<f32>,
    pub retardo_equipo_media_ms: Option<f32>,
    pub retardo_equipo_p95_ms: Option<f32>,
    pub perdida_media_pct: Option<f32>,
    pub perdida_max_pct: Option<f32>,
    pub jitter_media_ms: Option<f32>,
    pub rtt_media_ms: Option<f32>,
    pub idr_pedidos: u32,
    pub descartes: u64,
    pub tasa_descartes: u64,
    pub tasa_duplicados: u64,
    pub reconexiones: u32,
    pub recuperadas: u32,
    pub recuperacion_media_s: Option<f32>,
    pub recuperacion_max_s: Option<f32>,
    pub senal_min_dbm: Option<f32>,
    pub senal_media_dbm: Option<f32>,
    pub tx_mbps_media: Option<f32>,
    pub reintentos_delta: Option<u64>,
    pub cpu_media_pct: Option<f32>,
    pub carga_media: Option<f32>,
    pub psi_cpu_media_pct: Option<f32>,
    /// Whether "Audio en el televisor" was on.
    pub audio_tv: Option<bool>,
    pub audio_medido_s: u32,
    pub audio_senal_s: u32,
    pub audio_codec: Option<String>,
    pub audio_restaurada: Option<bool>,
    pub salida_virtual_ok: Option<bool>,
    pub salida_virtual_quitada: Option<bool>,
    pub ahorro_apagado: Option<bool>,
    pub latencia_medida_ms: Option<f32>,
    pub veredictos: Vec<Veredicto>,
}

/// One manual test (1 to 11) judged from what the log saw.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Veredicto {
    pub prueba: u8,
    pub nombre: &'static str,
    pub resultado: Resultado,
    pub detalle: String,
}

/// Ordered from best to worst, so the worst of several checks is their maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Resultado {
    Bien,
    Aviso,
    Mal,
    SinDatos,
}

/// `bien` up to `bien`, `aviso` up to `aviso`, `mal` beyond: for measures where less is better.
pub fn hasta(valor: f32, bien: f32, aviso: f32) -> Resultado {
    if valor <= bien {
        Resultado::Bien
    } else if valor <= aviso {
        Resultado::Aviso
    } else {
        Resultado::Mal
    }
}

/// The same for measures where more is better.
pub fn desde(valor: f32, bien: f32, aviso: f32) -> Resultado {
    hasta(-valor, -bien, -aviso)
}
