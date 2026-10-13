//! Projection engine: finds Miracast displays, links over Wi-Fi Direct, negotiates with
//! `wfd` and streams the screen with the GPU encoder. The interface talks to it only
//! through [`Motor`] (commands) and [`Aviso`] (what happened), so it never blocks.

mod activa;
mod busqueda;
pub mod captura;
mod ciclo;
mod control;
pub mod diagnostico;
pub mod enlace;
pub mod fallo;
pub mod medios;
mod medios_audio;
mod medios_cadena;
mod medios_estado;
mod medios_montaje;
mod medios_ritmo;
mod medios_tipos;
pub mod modo;
mod nm;
mod pantalla;
mod prioridad;
mod proyeccion;
pub mod recientes;
pub mod registro;
mod sesion;
pub mod wifi_direct;

pub use diagnostico::{Causa, Diagnostico};
pub use fallo::{Accion, Fallo};
pub use modo::{Fuente, Latencia, Opciones};
pub use proyeccion::Motor;
pub use wifi_direct::Pantalla;

/// How the engine reports, from any of its threads.
pub type Avisar = std::sync::Arc<dyn Fn(Aviso) + Send + Sync>;

/// Where a projection stands, as the Win+K panel shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Estado {
    Inactivo,
    Buscando,
    /// Inviting `nombre`: the TV may be asking the user to allow this laptop.
    Conectando {
        nombre: String,
    },
    /// Linked; waiting for the TV to open the video session.
    EsperandoPantalla {
        nombre: String,
    },
    /// The TV opened the session; agreeing on resolution and codecs (RTSP M1 to M7).
    Negociando {
        nombre: String,
    },
    Proyectando {
        nombre: String,
    },
    /// Something failed and the engine is trying again on its own.
    Reconectando {
        nombre: String,
        intento: u32,
    },
}

/// Live numbers for the status card.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Estadisticas {
    pub ancho: u32,
    pub alto: u32,
    pub fps: f32,
    pub kbps: u32,
    /// `true` with the GPU encoder, `false` with a software fallback.
    pub gpu: bool,
}

/// Everything the engine reports, delivered on its own threads: the interface must
/// queue it to the GUI thread.
#[derive(Debug, Clone)]
pub enum Aviso {
    Pantallas(Vec<Pantalla>),
    Recientes(Vec<Pantalla>),
    Estado(Estado),
    Estadisticas(Estadisticas),
    /// A failure the user should read, in plain Spanish.
    Error(String),
    /// Why the last projection failed and what to try.
    Diagnostico(Diagnostico),
}
