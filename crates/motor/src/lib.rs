//! Projection engine (Wi-Fi Direct, `wfd`, GPU encoder). The interface talks to it only through
//! [`Motor`] (commands) and [`Aviso`] (what happened), so it never blocks.

mod activa;
pub mod autodiagnostico;
mod autodiagnostico_sistema;
mod autodiagnostico_wpa;
mod busqueda;
pub mod captura;
mod ciclo;
mod ciclo_paso;
mod control;
pub mod diagnostico;
pub mod enlace;
pub mod envio;
mod envio_cubo;
mod envio_red;
mod envio_rtcp;
mod estadisticas;
pub mod fallo;
pub mod informe;
mod informe_metricas;
mod informe_sesion;
mod informe_texto;
pub mod medios;
mod medios_audio;
mod medios_cadena;
mod medios_codificador;
mod medios_estado;
mod medios_latencia;
mod medios_lectura;
mod medios_medidas;
mod medios_montaje;
mod medios_ritmo;
mod medios_tipos;
pub mod modo;
mod monitor;
mod nm;
mod pantalla;
mod perfil;
mod prioridad;
mod proyeccion;
pub mod recientes;
pub mod registro;
mod rtcp;
pub mod salida_audio;
mod salida_audio_cmd;
mod sesion;
mod transmision;
mod vigia;
pub mod wifi_direct;

pub use diagnostico::{Causa, Diagnostico};
pub use estadisticas::Estadisticas;
pub use fallo::{Accion, Fallo};
pub use modo::{Fuente, Latencia, Opciones};
pub use proyeccion::Motor;
pub use wifi_direct::Pantalla;

pub type Avisar = std::sync::Arc<dyn Fn(Aviso) + Send + Sync>; // called from any engine thread

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
