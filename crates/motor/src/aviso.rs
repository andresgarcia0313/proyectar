//! What the engine tells the interface: where a projection stands and everything it reports.

use crate::{Diagnostico, Estadisticas, Pantalla};

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
    Reconectando {
        nombre: String,
        intento: u32,
        motivo: String, // the diagnosis title
    },
}

/// Everything the engine reports, from its own threads: queue it to the GUI thread.
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
