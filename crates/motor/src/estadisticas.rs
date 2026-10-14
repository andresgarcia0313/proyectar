//! Live numbers of a projection, for the status card and the event log.

/// Live numbers for the status card.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct Estadisticas {
    pub ancho: u32,
    pub alto: u32,
    pub fps: f32,
    pub kbps: u32,
    /// `true` with the GPU encoder, `false` with a software fallback.
    pub gpu: bool,
    /// Delay inside this laptop, capture to network, in ms: average and worst of the last second.
    pub retardo_ms: f32,
    pub retardo_peor_ms: f32,
    pub retardo_cod_ms: f32, // capture to encoded frame
    /// From the TV's RTCP reports: share lost lately (%), packets lost in total, jitter (ms).
    pub perdida_pct: f32,
    pub perdidos: i32,
    pub jitter_ms: f32,
}
