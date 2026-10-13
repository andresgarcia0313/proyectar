//! Every way a projection can fail, and what the engine does about each one.
//! Rust has no exceptions: each step returns `Result`, and `recovery()` is the "catch"
//! that decides how the projection carries on.

use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Fallo {
    #[error("NetworkManager: {0}")]
    Red(#[from] zbus::Error),
    #[error("Wi-Fi Direct: {0}")]
    WifiDirect(String),
    #[error("la pantalla no se conectó a tiempo")]
    SinRespuesta,
    #[error("{0}")]
    Protocolo(#[from] wfd::Error),
    #[error("captura de pantalla: {0}")]
    Captura(String),
    #[error("se canceló la elección de pantalla o ventana")]
    CapturaCancelada,
    #[error("codificador {codificador}: {detalle}")]
    Codificador { codificador: &'static str, detalle: String },
    #[error("transmisión: {0}")]
    Medios(String),
    #[error("conexión local: {0}")]
    Io(#[from] std::io::Error),
}

/// What the engine does after a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accion {
    /// Same step again after the pause (link hiccup, busy sink).
    Reintentar(Duration),
    /// The encoder failed: move to the next one in the chain and keep the session.
    OtroCodificador,
    /// The capture stream ended (portal closed it): ask for a new one.
    NuevaCaptura,
    /// The Wi-Fi Direct link or the RTSP session is gone: connect from scratch.
    Reconectar,
    /// Nothing sensible to retry; tell the user why.
    Detener,
}

impl Fallo {
    pub fn recovery(&self) -> Accion {
        match self {
            Fallo::Red(_) | Fallo::Io(_) => Accion::Reintentar(Duration::from_secs(2)),
            Fallo::WifiDirect(_) | Fallo::SinRespuesta => Accion::Reconectar,
            Fallo::Protocolo(e) if e.is_transient() => Accion::Reconectar,
            Fallo::Protocolo(_) | Fallo::CapturaCancelada => Accion::Detener,
            Fallo::Captura(_) => Accion::NuevaCaptura,
            Fallo::Codificador { .. } => Accion::OtroCodificador,
            // The link is torn down with the session: the TV needs a moment before a new invitation.
            Fallo::Medios(_) => Accion::Reintentar(Duration::from_secs(3)),
        }
    }
}

pub type Resultado<T> = Result<T, Fallo>;
